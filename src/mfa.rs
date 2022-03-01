use std::io::Write;

use aws_config::{
    default_provider::region::DefaultRegionChain,
    profile::{Profile, ProfileSet},
};
use aws_types::credentials::{CredentialsError, ProvideCredentials, SharedCredentialsProvider};
use tracing::instrument;

#[derive(Debug)]
pub struct AssumeRoleWithMFATokenProvider {
    profile: Option<String>,
    token: Option<String>,
}

impl AssumeRoleWithMFATokenProvider {
    pub fn new() -> Self {
        Self {
            profile: None,
            token: None,
        }
    }

    pub fn set_profile(&mut self, profile: Option<impl Into<String>>) -> &mut Self {
        self.profile = profile.map(|p| p.into());
        self
    }

    pub fn set_token(&mut self, token: Option<impl Into<String>>) -> &mut Self {
        self.token = token.map(|t| t.into());
        self
    }
}

struct AssumeRoleWithMFATokenProviderRequest {
    role: String,
    key: String,
    secret: String,
    mfa_serial: String,
}

impl AssumeRoleWithMFATokenProviderRequest {
    pub fn new(
        role: impl Into<String>,
        key: impl Into<String>,
        secret: impl Into<String>,
        mfa_serial: impl Into<String>,
    ) -> Self {
        Self {
            role: role.into(),
            key: key.into(),
            secret: secret.into(),
            mfa_serial: mfa_serial.into(),
        }
    }

    pub async fn from_profile_set(
        profile_name: &str,
        set: &ProfileSet,
    ) -> Result<Self, &'static str> {
        let profile = set.get_profile(profile_name);
        let source = Self::get_source_profile(profile_name, set);

        if let (Some(profile), Some(source)) = (profile, source) {
            let profiles = [profile, source];

            let role = profile
                .get("role_arn")
                .ok_or("Failed to find a role to assume in selected profile")?;
            let key = Self::extract_field("aws_access_key_id", &profiles)
                .ok_or("Failed to find an access key in source profile for selected profile")?;
            let secret = Self::extract_field("aws_secret_access_key", &profiles)
                .ok_or("Failed to find a secret key in source profile for selected profile")?;
            let mfa_serial = profile
                .get("mfa_serial")
                .ok_or("Failed to find a mfa serial to use in selected profile")?;

            Ok(AssumeRoleWithMFATokenProviderRequest::new(
                role, key, secret, mfa_serial,
            ))
        } else {
            Err("Failed to find a profile or source")
        }
    }

    fn get_source_profile<'a>(profile: &str, set: &'a ProfileSet) -> Option<&'a Profile> {
        set.get_profile(profile).and_then(|p| {
            if let Some(source) = p.get("source_profile") {
                Self::get_source_profile(source, set)
            } else {
                Some(p)
            }
        })
    }

    fn extract_field<'a, const N: usize>(
        field: &str,
        profiles: &[&'a Profile; N],
    ) -> Option<&'a str> {
        for profile in profiles {
            if let Some(value) = profile.get(field) {
                return Some(value);
            }
        }

        None
    }
}

impl ProvideCredentials for AssumeRoleWithMFATokenProvider {
    #[instrument]
    fn provide_credentials<'a>(&'a self) -> aws_types::credentials::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        aws_types::credentials::future::ProvideCredentials::new(async move {
            let profiles = aws_config::profile::load(
                &aws_types::os_shim_internal::Fs::default(),
                &aws_types::os_shim_internal::Env::default(),
            )
            .await
            .map_err(|err| CredentialsError::not_loaded(err))?;

            let profile_name = self
                .profile
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("default");

            let request =
                AssumeRoleWithMFATokenProviderRequest::from_profile_set(profile_name, &profiles)
                    .await
                    .map_err(|err| CredentialsError::not_loaded(err))?;

            let region = DefaultRegionChain::builder()
                .profile_name(
                    self.profile
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("default"),
                )
                .build()
                .region()
                .await;

            let credentials_provider =
                SharedCredentialsProvider::new(aws_types::credentials::Credentials::new(
                    request.key,
                    request.secret,
                    None,
                    None,
                    "assumed-role-credentials",
                ));

            let config = aws_config::Config::builder()
                .region(region.clone())
                .credentials_provider(SharedCredentialsProvider::new(credentials_provider))
                .build();

            let sts_client = aws_sdk_sts::Client::new(&config);

            // TODO: Academically how do we rewrite this block to prevent creating a copy of the
            // token when it has already been supplied
            let mfa_token = if let Some(mfa_token) = &self.token {
                mfa_token.to_string()
            } else {
                let handle = tokio::task::spawn_blocking(|| -> Result<String, CredentialsError> {
                    print!("MFA token is required: ");
                    std::io::stdout()
                        .flush()
                        .map_err(|err| CredentialsError::not_loaded(err))?;

                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .map_err(|err| CredentialsError::not_loaded(err))?;
                    Ok(input.trim().to_string())
                })
                .await
                .map_err(|err| CredentialsError::not_loaded(err))?;

                handle?
            };

            let role = sts_client
                .assume_role()
                .role_session_name("envfmt")
                .role_arn(request.role)
                .serial_number(request.mfa_serial)
                .token_code(mfa_token)
                .send()
                .await
                .map_err(|err| CredentialsError::not_loaded(err))?;

            role.credentials()
                .map(|credentials| {
                    aws_types::credentials::Credentials::new(
                        credentials.access_key_id.as_ref().unwrap(),
                        credentials.secret_access_key.as_ref().unwrap(),
                        credentials.session_token().map(|s| s.into()),
                        None,
                        // credentials.expiration().map(|t| t.into()),
                        "AssumeRoleWithMFAToken",
                    )
                })
                .ok_or(CredentialsError::not_loaded(
                    "Successfully assume role, but not credentials were returned",
                ))
        })
    }
}
