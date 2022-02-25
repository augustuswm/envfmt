use dotenv::from_filename_iter;
use tracing::debug;

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
    pub next: Option<String>,
}

impl ParamBag {
    pub fn new(path: &str) -> Self {
        let path_formatted = normalize_path(path);
        // let req = make_path_req(path_formatted.as_str(), None);

        ParamBag {
            prefix: path_formatted,
            params: Vec::new(),
            next: None,
        }
    }

    pub fn from_dotenv(file: &str, prefix: &str) -> Self {
        let params = from_filename_iter(&file)
            .unwrap()
            .filter_map(|item| item.ok())
            .map(|(key, value)| Param { key, value })
            .collect::<Vec<Param>>();

        ParamBag {
            prefix: prefix.to_string(),
            params,
            next: None,
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

impl ParamBag {
    #[tracing::instrument(skip(client))]
    pub async fn process(mut self, client: &aws_sdk_ssm::Client) -> ParamResult {
        let resp = client
            .get_parameters_by_path()
            .path(&self.prefix)
            .set_next_token(self.next)
            .send()
            .await
            .map_err(Box::new)?;

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

        self.next = resp.next_token;

        Ok(self)
    }
}

pub fn to_env_name(name: &str) -> String {
    name[name.rfind('/').unwrap_or(0) + 1..].to_uppercase()
}

#[tracing::instrument(skip(client))]
pub async fn get_all_params_for_path(client: &aws_sdk_ssm::Client, path: &str) -> ParamResult {
    let mut bag = ParamBag::new(path);

    debug!("Created bag");

    loop {
        debug!("Process bag");

        bag = bag.process(client).await?;

        if bag.next.is_none() {
            break;
        }
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
            input: GetParametersByPath,
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
            next: Some(make_path_req("/path/to/the/", None)),
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
    fn test_process_creates_nextuest_with_token() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next: Some(make_path_req("/path/to/the/", None)),
        };

        bag = bag.process(&client_with_next()).unwrap();

        assert!(bag.next.is_some());
        assert!(bag.next.clone().unwrap().next_token.is_some());
        assert_eq!(
            "this-is-the-next-token",
            bag.next.unwrap().next_token.unwrap()
        );
    }

    #[test]
    fn test_process_does_not_create_nextuest_without_token() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next: Some(make_path_req("/path/to/the/", None)),
        };

        bag = bag.process(&client_without_next()).unwrap();

        assert!(bag.next.is_none());
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
        assert!(bag.next.is_none());
    }
}
