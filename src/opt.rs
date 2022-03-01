use clap::{Parser, Subcommand};

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct EnvFmtOpts {
    // Mode to operate in. read or write
    #[clap(subcommand)]
    pub command: Command,
    #[clap(name = "format", long, short, global = true, help ="Format to use when printing results", possible_values = ["dot-env", "php-fpm"])]
    pub format: Option<Format>,
    #[clap(
        name = "region",
        long,
        short,
        help = "AWS region to query against. Defaults to us-east-1",
        global = true
    )]
    pub region: Option<String>,
    #[clap(
        name = "profile",
        long,
        short,
        help = "AWS profile to authenticate with",
        global = true
    )]
    pub profile: Option<String>,
    #[clap(name = "debug", long, help = "Display verbose debug information")]
    pub debug: bool,
    #[clap(
        name = "mfa",
        long,
        help = "Enables MFA authentication and token prompt",
        global = true,
        conflicts_with = "mfa-token"
    )]
    pub mfa: bool,
    #[clap(
        name = "mfa-token",
        long,
        help = "Enables MFA authentication and accepts token instead of prompting",
        global = true,
        conflicts_with = "mfa"
    )]
    pub mfa_token: Option<String>,
    #[clap(
        name = "out",
        short,
        long,
        help = "Output location for parameters instead of stdout",
        global = true
    )]
    pub out: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read parameters from AWS
    Read {
        /// Path prefix to select parameters for
        path: String,
    },
    /// Write parameters to AWS
    Write {
        #[clap(long, help = "Prefix to prepend tp each variable")]
        prefix: Option<String>,
        /// File path to a config file to read from
        file_path: String,
        /// Allow overwriting of existing values
        #[clap(short, long)]
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

impl std::error::Error for ArgError {}
