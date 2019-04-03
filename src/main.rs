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

use rusoto_core::{credential::ProfileProvider, HttpClient, Region};
use rusoto_ssm::SsmClient;
use structopt::StructOpt;

use std::error::Error;

mod formatter;
mod opt;
mod params;

use crate::formatter::{DotEnv, PhpFpm};
use crate::opt::{EnvFmtOpts, Format};
use crate::params::get_all_params_for_path;

fn make_profile_provider(profile: &str) -> ProfileProvider {
    let mut provider = ProfileProvider::new().expect("Failed to find AWS credentials");
    provider.set_profile(profile);
    provider
}

fn make_client(opts: &EnvFmtOpts) -> SsmClient {
    match &opts.profile {
        Some(profile) => SsmClient::new_with(
            HttpClient::new().expect("Failed to create HTTP client"),
            make_profile_provider(profile),
            opts.region.clone().unwrap_or(Region::default()),
        ),
        None => SsmClient::new(opts.region.clone().unwrap_or(Region::default())),
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let opts = EnvFmtOpts::from_args();

    let client = make_client(&opts);
    let bag = get_all_params_for_path(&client, &opts.path)?;

    Ok(match opts.format.unwrap_or(Format::DotEnv) {
        Format::DotEnv => print!("{}", DotEnv::from(bag)),
        Format::PhpFpm => print!("{}", PhpFpm::from(bag)),
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_makes_profile_provider() {
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", "/tmp/not_a_file");

        let mut opts = opt::EnvFmtOpts::default();
        opts.profile = Some("not_a_profile".to_string());

        let provider = make_profile_provider(&opts.profile.unwrap());

        assert_eq!("not_a_profile", provider.profile())
    }
}
