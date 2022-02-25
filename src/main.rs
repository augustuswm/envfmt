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

use structopt::StructOpt;
use tracing::debug;

use std::error::Error;

mod formatter;
mod opt;
mod params;
mod writer;

use crate::formatter::{DotEnv, PhpFpm};
use crate::opt::{Command, EnvFmtOpts, Format};
use crate::params::{get_all_params_for_path, ParamBag};
use crate::writer::Writer;

async fn ssm_client() -> aws_sdk_ssm::Client {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_ssm::Client::new(&shared_config);
    client
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // tracing_subscriber::fmt::init();

    let opts = EnvFmtOpts::from_args();

    let client = ssm_client().await;

    debug!("Prep complete");

    match opts.command {
        Command::Read { ref path } => {
            let bag = get_all_params_for_path(&client, &path).await?;

            match opts.format.unwrap_or(Format::DotEnv) {
                Format::DotEnv => print!("{}", DotEnv::from(bag)),
                Format::PhpFpm => print!("{}", PhpFpm::from(bag)),
            }
        }
        Command::Write {
            ref prefix,
            ref file_path,
            ref overwrite,
        } => {
            let writer = Writer::new(client, *overwrite);
            let bag = ParamBag::from_dotenv(file_path, &prefix.as_ref().unwrap_or(&"".to_string()));

            writer.write(&bag).await;
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_makes_profile_provider() {
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", "/tmp/not_a_file");

        let mut opts = crate::opt::EnvFmtOpts::default();
        opts.profile = Some("not_a_profile".to_string());

        let provider = make_profile_provider(&opts.profile.unwrap());

        assert_eq!("not_a_profile", provider.profile())
    }
}
