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

use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use structopt::StructOpt;

use std::error::Error;

mod formatter;
mod opt;
mod params;

use crate::formatter::{DotEnv, PhpFpm};
use crate::opt::{EnvFmtOpts, Format};
use crate::params::get_all_params_for_path;

pub fn main() -> Result<(), Box<dyn Error>> {
    let opt = EnvFmtOpts::from_args();

    let client = SsmClient::new(opt.region.unwrap_or(Region::default()));
    let bag = get_all_params_for_path(&client, &opt.path)?;

    Ok(match opt.format.unwrap_or(Format::DotEnv) {
        Format::DotEnv => print!("{}", DotEnv::from(bag)),
        Format::PhpFpm => print!("{}", PhpFpm::from(bag)),
    })
}
