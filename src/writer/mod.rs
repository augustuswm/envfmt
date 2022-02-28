use aws_sdk_ssm::model::ParameterType;

use crate::params::ParamBag;

pub struct Writer {
    client: aws_sdk_ssm::Client,
    force: bool,
}

impl Writer {
    pub fn new(client: aws_sdk_ssm::Client, force: bool) -> Self {
        Writer { client, force }
    }

    pub async fn write(&self, bag: &ParamBag) -> Option<()> {
        for param in bag.params.iter() {
            match self
                .client
                .put_parameter()
                .name(format!("{}/{}", bag.prefix, param.key.to_lowercase()))
                .overwrite(self.force)
                .set_type(Some(ParameterType::String))
                .value(param.value.to_string())
                .send()
                .await
            {
                Ok(_) => println!("Wrote {}/{}", bag.prefix, param.key.to_lowercase()),
                Err(err) => println!(
                    "Failed to write {}/{} due to {} {:?}",
                    bag.prefix,
                    param.key.to_lowercase(),
                    err,
                    err
                ),
            };

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        Some(())
    }
}
