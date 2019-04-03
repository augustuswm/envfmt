use rusoto_core::Region;
use structopt::StructOpt;

use std::fmt;
use std::str::FromStr;

#[derive(Debug, StructOpt, Default)]
#[structopt(
    name = "envfmt",
    author = "",
    about = "Fetches env parameters from SSM"
)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::ArgRequiredElseHelp")
)]
pub struct EnvFmtOpts {
    /// Path prefix to select parameters for
    pub path: String,
    /// Format to output results as
    #[structopt(
        raw(possible_values = "&[&\"dot-env\", &\"php-fpm\"]"),
        name = "format",
        long,
        short
    )]
    pub format: Option<Format>,
    #[structopt(
        name = "region",
        long,
        short,
        help = "AWS region to query against. Defaults to us-east-1"
    )]
    pub region: Option<Region>,
    #[structopt(
        name = "profile",
        long,
        short,
        help = "AWS profile to authenticate with"
    )]
    pub profile: Option<String>,
}

#[derive(Debug)]
pub enum Format {
    DotEnv,
    PhpFpm,
}

#[derive(Debug)]
pub enum FormatError {
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
