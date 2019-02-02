use rusoto_ssm::{GetParametersByPathRequest, Ssm, SsmClient};

use std::error::Error;

#[derive(Debug)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct ParamBag {
    pub prefix: String,
    pub params: Vec<Param>,
    pub next_req: Option<GetParametersByPathRequest>,
}

type ParamResult = Result<ParamBag, Box<dyn Error>>;

impl ParamBag {
    pub fn process(mut self, client: &SsmClient) -> ParamResult {
        if let Some(req) = self.next_req.take() {
            let resp = client
                .get_parameters_by_path(req)
                .sync()
                .map_err(Box::new)?;

            if let Some(parameters) = resp.parameters {
                for parameter in parameters {
                    if let (Some(name), Some(value)) = (parameter.name, parameter.value) {
                        self.params.push(Param {
                            key: to_env_name(self.prefix.as_str(), name.as_str()),
                            value,
                        });
                    }
                }
            }

            if resp.next_token.is_some() {
                self.next_req = Some(make_path_req(self.prefix.as_str(), resp.next_token))
            }
        }

        Ok(self)
    }
}

pub fn make_path_req(path: &str, next_token: Option<String>) -> GetParametersByPathRequest {
    GetParametersByPathRequest {
        max_results: None,
        next_token: next_token,
        parameter_filters: None,
        path: path.to_string(),
        recursive: Some(false),
        with_decryption: Some(true),
    }
}

pub fn to_env_name(prefix: &str, name: &str) -> String {
    name.trim_start_matches(prefix).to_uppercase()
}

pub fn get_all_params_for_path(client: &SsmClient, path: &str) -> ParamResult {
    let mut bag = ParamBag {
        prefix: path.to_string(),
        params: Vec::new(),
        next_req: Some(make_path_req(path, None)),
    };

    while bag.next_req.is_some() {
        bag = bag.process(&client)?;
    }

    Ok(bag)
}
