use std::fmt;

use crate::params::{Param, ParamBag};

pub struct DotEnv<'a> {
    params: &'a Vec<Param>,
}

impl<'a> From<&'a ParamBag> for DotEnv<'a> {
    fn from(bag: &'a ParamBag) -> Self {
        DotEnv {
            params: &bag.params,
        }
    }
}

impl<'a> fmt::Display for DotEnv<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = self
            .params
            .iter()
            .map(|param: &Param| param.key.clone() + "=" + "\"" + &param.value + "\"\n")
            .collect::<String>();

        write!(f, "{}", out.trim())
    }
}

pub struct PhpFpm<'a> {
    params: &'a Vec<Param>,
}

impl<'a> From<&'a ParamBag> for PhpFpm<'a> {
    fn from(bag: &'a ParamBag) -> Self {
        PhpFpm {
            params: &bag.params,
        }
    }
}

impl<'a> fmt::Display for PhpFpm<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "env[";

        let out = self
            .params
            .iter()
            .map(|param: &Param| {
                prefix.to_string() + &param.key + "]=" + "\"" + &param.value + "\"\n"
            })
            .collect::<String>();

        write!(f, "{}", out.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_as_dot_env() {
        let params = vec![
            Param {
                key: "ALPHA".to_string(),
                value: "the".to_string(),
            },
            Param {
                key: "BETA".to_string(),
                value: "four".to_string(),
            },
            Param {
                key: "DELTA".to_string(),
                value: "test".to_string(),
            },
            Param {
                key: "GAMMA".to_string(),
                value: "strings".to_string(),
            },
        ];

        let output = "ALPHA=\"the\"\nBETA=\"four\"\nDELTA=\"test\"\nGAMMA=\"strings\"";

        assert_eq!(output, format!("{}", DotEnv { params: &params }));
    }

    #[test]
    fn formats_as_php_fpm() {
        let params = vec![
            Param {
                key: "ALPHA".to_string(),
                value: "the".to_string(),
            },
            Param {
                key: "BETA".to_string(),
                value: "four".to_string(),
            },
            Param {
                key: "DELTA".to_string(),
                value: "test".to_string(),
            },
            Param {
                key: "GAMMA".to_string(),
                value: "strings".to_string(),
            },
        ];

        let output =
            "env[ALPHA]=\"the\"\nenv[BETA]=\"four\"\nenv[DELTA]=\"test\"\nenv[GAMMA]=\"strings\"";

        assert_eq!(output, format!("{}", PhpFpm { params: &params }));
    }
}
