use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use structopt::StructOpt;

use std::error::Error;
use std::fmt;
use std::str::FromStr;

mod formatter;
mod params;

use crate::formatter::{DotEnv, PhpFpm};
use crate::params::get_all_params_for_path;

#[derive(Debug, StructOpt)]
#[structopt(name = "envfmt", about = "Fetches env parameters from SSM")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct EnvFmtOpts {
    /// Path prefix to select parameters for
    path: String,
    /// Format to output results as
    #[structopt(raw(possible_values = "&[&\"dot-env\", &\"php-fpm\"]"))]
    format: Format,
}

#[derive(Debug)]
enum Format {
    DotEnv,
    PhpFpm,
}

#[derive(Debug)]
enum FormatError {
    InvalidFormat,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is not a valid output format", self)
    }
}

impl FromStr for Format {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dot-env" => Ok(Format::DotEnv),
            "php-fpm" => Ok(Format::PhpFpm),
            _ => Err(FormatError::InvalidFormat),
        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let opt = EnvFmtOpts::from_args();

    let client = SsmClient::new(Region::UsEast1);
    let bag = get_all_params_for_path(&client, &opt.path)?;

    Ok(match opt.format {
        Format::DotEnv => print!("{}", DotEnv::from(bag)),
        Format::PhpFpm => print!("{}", PhpFpm::from(bag)),
    })
}
