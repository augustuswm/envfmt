use rusoto_core::RusotoFuture;
use rusoto_ssm::{
    GetParametersByPathError, GetParametersByPathRequest, GetParametersByPathResult, Ssm, SsmClient,
};

use std::error::Error;

#[derive(Debug, PartialEq)]
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

impl ParamBag {
    pub fn new(path: &str) -> Self {
        let path_formatted = normalize_path(path);
        let req = make_path_req(path_formatted.as_str(), None);

        ParamBag {
            prefix: path_formatted,
            params: Vec::new(),
            next_req: Some(req),
        }
    }
}

pub fn normalize_path(path: &str) -> String {
    match path.chars().next() {
        Some('/') => path.to_string(),
        _ => "/".to_string() + path,
    }
}

type ParamResult = Result<ParamBag, Box<dyn Error>>;

pub trait Client {
    fn get_param_page(
        &self,
        input: GetParametersByPathRequest,
    ) -> RusotoFuture<GetParametersByPathResult, GetParametersByPathError>;
}

impl Client for SsmClient {
    fn get_param_page(
        &self,
        input: GetParametersByPathRequest,
    ) -> RusotoFuture<GetParametersByPathResult, GetParametersByPathError> {
        self.get_parameters_by_path(input)
    }
}

impl ParamBag {
    pub fn process<T>(mut self, client: &T) -> ParamResult
    where
        T: Client,
    {
        if let Some(req) = self.next_req.take() {
            let resp = client.get_param_page(req).sync().map_err(Box::new)?;

            if let Some(parameters) = resp.parameters {
                for parameter in parameters {
                    if let (Some(name), Some(value)) = (parameter.name, parameter.value) {
                        self.params.push(Param {
                            key: to_env_name(name.as_str()).to_string(),
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

pub fn to_env_name(name: &str) -> String {
    name[name.rfind('/').unwrap_or(0) + 1..].to_uppercase()
}

pub fn get_all_params_for_path<T>(client: &T, path: &str) -> ParamResult
where
    T: Client,
{
    let mut bag = ParamBag::new(path);

    while bag.next_req.is_some() {
        bag = bag.process(client)?;
    }

    Ok(bag)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
    };

    use std::cell::RefCell;

    struct MultiPageSsmClient {
        page: RefCell<u8>,
    }

    impl Client for MultiPageSsmClient {
        fn get_param_page(
            &self,
            input: GetParametersByPathRequest,
        ) -> RusotoFuture<GetParametersByPathResult, GetParametersByPathError> {
            let res = match *self.page.borrow() {
                1 => client_with_next().get_param_page(input),
                2 => client_with_next_ending().get_param_page(input),
                _ => panic!("Too many pages!"),
            };

            let new_page = { *self.page.borrow() + 1 };

            self.page.replace(new_page);

            res
        }
    }

    fn client_with_next() -> SsmClient {
        rusoto_ssm::SsmClient::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "test_data",
                "path_with_next_resp.json",
            )),
            MockCredentialsProvider,
            Default::default(),
        )
    }

    fn client_with_next_ending() -> SsmClient {
        rusoto_ssm::SsmClient::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "test_data",
                "path_with_next_ending_resp.json",
            )),
            MockCredentialsProvider,
            Default::default(),
        )
    }

    fn client_without_next() -> SsmClient {
        rusoto_ssm::SsmClient::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "test_data",
                "path_resp.json",
            )),
            MockCredentialsProvider,
            Default::default(),
        )
    }

    fn client_with_multiple_pages() -> MultiPageSsmClient {
        MultiPageSsmClient {
            page: RefCell::new(1),
        }
    }

    #[test]
    fn test_normalizes_path_on_construction() {
        let b1 = ParamBag::new("/path/to/the");
        let b2 = ParamBag::new("path/to/the");

        assert_eq!("/path/to/the", b1.prefix);
        assert_eq!("/path/to/the", b2.prefix);
    }

    #[test]
    fn test_process_add_results_to_bag() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next_req: Some(make_path_req("/path/to/the/", None)),
        };

        bag = bag.process(&client_without_next()).unwrap();

        assert_eq!(
            Param {
                key: "FIRST_PARAM".into(),
                value: "first_param_value".into()
            },
            bag.params[0]
        );

        assert_eq!(
            Param {
                key: "SECOND_PARAM".into(),
                value: "second_param_value".into()
            },
            bag.params[1]
        );
    }

    #[test]
    fn test_process_creates_next_request_with_token() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next_req: Some(make_path_req("/path/to/the/", None)),
        };

        bag = bag.process(&client_with_next()).unwrap();

        assert!(bag.next_req.is_some());
        assert!(bag.next_req.clone().unwrap().next_token.is_some());
        assert_eq!(
            "this-is-the-next-token",
            bag.next_req.unwrap().next_token.unwrap()
        );
    }

    #[test]
    fn test_process_does_not_create_next_request_without_token() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next_req: Some(make_path_req("/path/to/the/", None)),
        };

        bag = bag.process(&client_without_next()).unwrap();

        assert!(bag.next_req.is_none());
    }

    #[test]
    fn test_converts_to_env_var_name() {
        assert_eq!("PARAM_KEY", to_env_name("/path/to/the/param_key"));
    }

    #[test]
    fn test_makes_initial_process_call() {
        let bag = get_all_params_for_path(&client_without_next(), "/path/to/the").unwrap();

        assert_eq!(2, bag.params.len());
    }

    #[test]
    fn test_calls_process_until_out_of_requests() {
        let bag = get_all_params_for_path(&client_with_multiple_pages(), "/path/to/the").unwrap();
        assert_eq!(4, bag.params.len());
        assert!(bag.next_req.is_none());
    }
}
