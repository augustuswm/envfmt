use rusoto_core::Region;
use structopt::StructOpt;

use std::fmt;
use std::str::FromStr;

#[derive(Debug, StructOpt)]
#[structopt(name = "envfmt", about = "Fetches env parameters from SSM")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct EnvFmtOpts {
    /// Path prefix to select parameters for
    pub path: String,
    /// Format to output results as
    #[structopt(raw(possible_values = "&[&\"dot-env\", &\"php-fpm\"]"))]
    pub format: Format,
    #[structopt(name = "region", long, short)]
    pub region: Option<Region>,
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
