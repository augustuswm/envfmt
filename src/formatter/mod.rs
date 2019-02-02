use std::fmt;

use crate::params::{Param, ParamBag};

pub struct DotEnv {
    params: Vec<Param>,
}

impl From<ParamBag> for DotEnv {
    fn from(bag: ParamBag) -> Self {
        DotEnv { params: bag.params }
    }
}

impl fmt::Display for DotEnv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = self
            .params
            .iter()
            .map(|param: &Param| param.key.clone() + "=" + "\"" + &param.value + "\"\n")
            .collect::<String>();

        write!(f, "{}", out.trim())
    }
}

pub struct PhpFpm {
    params: Vec<Param>,
}

impl From<ParamBag> for PhpFpm {
    fn from(bag: ParamBag) -> Self {
        PhpFpm { params: bag.params }
    }
}

impl fmt::Display for PhpFpm {
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

        assert_eq!(output, format!("{}", DotEnv { params }));
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

        assert_eq!(output, format!("{}", PhpFpm { params }));
    }
}
