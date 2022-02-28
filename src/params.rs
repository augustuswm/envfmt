use async_trait::async_trait;
use dotenv::from_filename_iter;
use tracing::debug;

use std::error::Error;

#[async_trait]
pub trait ReadParamClient {
    async fn get_params(&self, mut bag: ParamBag) -> ParamResult;
}

#[async_trait]
impl ReadParamClient for aws_sdk_ssm::Client {
    async fn get_params(&self, mut bag: ParamBag) -> ParamResult {
        let resp = self
            .get_parameters_by_path()
            .path(&bag.prefix)
            .set_next_token(bag.next)
            .send()
            .await
            .map_err(Box::new)?;

        if let Some(parameters) = resp.parameters {
            for parameter in parameters {
                if let (Some(name), Some(value)) = (parameter.name, parameter.value) {
                    bag.params.push(Param {
                        key: to_env_name(name.as_str()).to_string(),
                        value,
                    });
                }
            }
        }

        bag.next = resp.next_token;

        Ok(bag)
    }
}

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
    pub async fn process<T>(self, client: &T) -> ParamResult
    where
        T: ReadParamClient,
    {
        client.get_params(self).await
    }
}

pub fn to_env_name(name: &str) -> String {
    name[name.rfind('/').unwrap_or(0) + 1..].to_uppercase()
}

#[tracing::instrument(skip(client))]
pub async fn get_all_params_for_path<T>(client: &T, path: &str) -> ParamResult
where
    T: ReadParamClient,
{
    let mut bag = ParamBag::new(path);

    debug!(?bag, "Created empty bag for storage bag");

    loop {
        bag = bag.process(client).await?;

        debug!(?bag, "Processed bag request. Checking for more.");

        if bag.next.is_none() {
            break;
        }
    }

    Ok(bag)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::RwLock};

    use async_trait::async_trait;
    use serde::Deserialize;
    use serde_json;
    use tracing::instrument;

    use super::*;

    #[derive(Deserialize)]
    struct TestParam {
        key: String,
        value: String,
    }

    #[derive(Deserialize)]
    struct Page {
        params: Vec<TestParam>,
        token: Option<String>,
    }

    #[derive(Deserialize)]
    struct MultiPageSsmClient {
        inner: RwLock<Inner>,
    }

    #[derive(Deserialize)]
    struct Inner {
        first_read: bool,
        pages: HashMap<String, Page>,
    }

    #[async_trait]
    impl ReadParamClient for MultiPageSsmClient {
        #[instrument(skip(self))]
        async fn get_params(&self, mut bag: ParamBag) -> ParamResult {
            let mut inner = self.inner.write().unwrap();

            let page = if inner.first_read {
                inner.pages.get("first")
            } else {
                bag.next.and_then(|token| inner.pages.get(&token))
            };

            if let Some(page) = page {
                for p in &page.params {
                    bag.params.push(Param {
                        key: to_env_name(&p.key).to_string(),
                        value: p.value.clone(),
                    });
                }

                bag.next = page.token.clone();
            } else {
                bag.next = None;
            }

            inner.first_read = false;

            Ok(bag)
        }
    }

    fn one_page_client() -> MultiPageSsmClient {
        MultiPageSsmClient {
            inner: RwLock::new(Inner {
                first_read: true,
                pages: serde_json::from_str::<HashMap<String, Page>>(include_str!(
                    "../test_data/one_page_client.json"
                ))
                .unwrap(),
            }),
        }
    }

    fn two_page_client() -> MultiPageSsmClient {
        MultiPageSsmClient {
            inner: RwLock::new(Inner {
                first_read: true,
                pages: serde_json::from_str::<HashMap<String, Page>>(include_str!(
                    "../test_data/two_page_client.json"
                ))
                .unwrap(),
            }),
        }
    }

    #[test]
    fn test_normalizes_path_on_construction() {
        let b1 = ParamBag::new("/path/to/the");
        let b2 = ParamBag::new("path/to/the");

        assert_eq!("/path/to/the", b1.prefix);
        assert_eq!("/path/to/the", b2.prefix);
    }

    #[tokio::test]
    async fn test_process_add_results_to_bag() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next: None,
        };

        bag = bag.process(&one_page_client()).await.unwrap();

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

    #[tokio::test]
    async fn test_process_updates_with_next_token() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next: None,
        };

        bag = bag.process(&two_page_client()).await.unwrap();

        assert!(bag.next.is_some());
        assert_eq!("second", bag.next.unwrap());
    }

    #[tokio::test]
    async fn test_process_updates_with_empty_token() {
        let mut bag = ParamBag {
            prefix: "/path/to/the/".to_string(),
            params: vec![],
            next: None,
        };

        let client = two_page_client();

        bag = bag.process(&client).await.unwrap();
        bag = bag.process(&client).await.unwrap();

        assert!(bag.next.is_none());
    }

    #[test]
    fn test_converts_to_env_var_name() {
        assert_eq!("PARAM_KEY", to_env_name("/path/to/the/param_key"));
    }

    #[tokio::test]
    async fn test_makes_initial_process_call() {
        let bag = get_all_params_for_path(&one_page_client(), "/path/to/the")
            .await
            .unwrap();

        assert_eq!(2, bag.params.len());
    }

    #[tokio::test]
    async fn test_calls_process_until_out_of_requests() {
        let bag = get_all_params_for_path(&two_page_client(), "/path/to/the")
            .await
            .unwrap();
        assert_eq!(4, bag.params.len());
        assert!(bag.next.is_none());
    }
}
