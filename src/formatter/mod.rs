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

        write!(f, "{}", out)
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

        write!(f, "{}", out)
    }
}
