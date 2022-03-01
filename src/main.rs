//! A small command line utility for reading parameters from a path in
//! the AWS Systems Manager Parameter Store and outputting them in a given
//! format.
//!
//! Parameters are expected to have keys stored in Parameter Store under an
//! AWS path format.
//!
//! `/path1/path2/path3/param`
//!
//! Two output formats are currently support: `.env` and `php-fpm.conf`
//!
//! `envfmt /path/to/ dot-env > .env`
//!
//! `envfmt /path/to/ php-fpm > env.conf`
//!
//! The region to use can be specified with the `region` flag.
//!
//! `envfmt /path/to/ dot-env --region us-west-1 > .env`
//!
//! If left unspecified the region will attempt to be read from the current
//! environment. In the case that it fails, it will fall back to us-east-1.

use aws_config::default_provider::region::DefaultRegionChain;
use aws_config::meta::credentials::LazyCachingCredentialsProvider;
use aws_types::credentials::SharedCredentialsProvider;
use clap::Parser;

use std::error::Error;
use std::fmt::Display;
use std::io::Write;
use std::time::Duration;

mod formatter;
mod mfa;
mod opt;
mod params;
mod writer;

use crate::formatter::{DotEnv, PhpFpm};
use crate::opt::{Command, EnvFmtOpts, Format};
use crate::params::{get_all_params_for_path, ParamBag};
use crate::writer::Writer;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let opts = EnvFmtOpts::parse();

    if opts.debug {
        tracing_subscriber::fmt::init();
    }

    let conf = if opts.mfa || opts.mfa_token.is_some() {
        let region = DefaultRegionChain::builder()
            .profile_name(
                opts.profile
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("default"),
            )
            .build()
            .region()
            .await;

        let mut mfa_provider = mfa::AssumeRoleWithMFATokenProvider::new();
        mfa_provider.set_profile(opts.profile);
        mfa_provider.set_token(opts.mfa_token);

        let lazy_provider = LazyCachingCredentialsProvider::builder()
            .load(mfa_provider)
            .load_timeout(Duration::from_secs(60))
            .build();

        let conf = aws_config::Config::builder()
            .region(region)
            .credentials_provider(SharedCredentialsProvider::new(lazy_provider))
            .build();

        conf
    } else {
        aws_config::load_from_env().await
    };

    let client = aws_sdk_ssm::Client::new(&conf);

    let result = match opts.command {
        Command::Read { ref path } => {
            let res = get_all_params_for_path(&client, &path).await;

            if let Ok(ref bag) = res {
                let formatted: Box<dyn Display> = match opts.format.unwrap_or(Format::DotEnv) {
                    Format::DotEnv => Box::new(DotEnv::from(bag)),
                    Format::PhpFpm => Box::new(PhpFpm::from(bag)),
                };

                if let Some(out_file) = opts.out {
                    let mut file = std::fs::File::create(out_file)?;
                    file.write_all(format!("{}", formatted).as_bytes())?;
                } else {
                    print!("{}", formatted);
                }
            }

            res.map(|_| ())
        }
        Command::Write {
            ref prefix,
            ref file_path,
            ref overwrite,
        } => {
            let writer = Writer::new(client, *overwrite);
            let bag = ParamBag::from_dotenv(file_path, &prefix.as_ref().unwrap_or(&"".to_string()));

            writer.write(&bag).await;

            Ok(())
        }
    };

    if result.is_err() {
        tracing::error!(?result, "Failed to get parameters from remote");
        println!("Failed to get paramaters");
    }

    Ok(())
}
