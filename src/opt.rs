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
    // Mode to operate in. read or write
    #[structopt(subcommand)]
    pub command: Command,
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
    pub region: Option<String>,
    #[structopt(
        name = "profile",
        long,
        short,
        help = "AWS profile to authenticate with"
    )]
    pub profile: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::ArgRequiredElseHelp")
)]
pub enum Command {
    /// Read parameters from AWS
    #[structopt(name = "read")]
    Read {
        /// Path prefix to select parameters for
        path: String,
    },
    /// Write parameters to AWS
    #[structopt(name = "write")]
    Write {
        #[structopt(name = "prefix", long, help = "Prefix to prepend tp each variable")]
        prefix: Option<String>,
        /// File path to a config file to read from
        file_path: String,
        /// Allow overwriting of existing values
        #[structopt(short, long)]
        overwrite: bool,
    },
}

impl Default for Command {
    fn default() -> Command {
        Command::Read {
            path: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Format {
    DotEnv,
    PhpFpm,
}

#[derive(Debug)]
pub enum ArgError {
    InvalidFormat,
    InvalidCommand,
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is not a valid output format", self)
    }
}

impl FromStr for Format {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dot-env" => Ok(Format::DotEnv),
            "php-fpm" => Ok(Format::PhpFpm),
            _ => Err(ArgError::InvalidFormat),
        }
    }
}
