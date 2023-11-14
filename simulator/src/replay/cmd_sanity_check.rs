use super::Error;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::profile::ProfileFileRegionProvider;
use clap::Parser;
use radix_engine::types::*;

#[derive(Parser, Debug)]
pub struct SanityCheck {}

impl SanityCheck {
    pub fn run(&self) -> Result<(), Error> {
        async fn scan() {
            let config = aws_config::from_env()
                .region(
                    ProfileFileRegionProvider::builder()
                        .profile_name("sandbox-cli")
                        .build(),
                )
                .credentials_provider(
                    ProfileFileCredentialsProvider::builder()
                        .profile_name("sandbox-cli")
                        .build(),
                )
                .load()
                .await;
            let client = aws_sdk_s3::Client::new(&config);

            let objects = client
                .list_objects_v2()
                .bucket("yulongtest")
                .prefix("transaction-json")
                .send()
                .await
                .unwrap();
            println!("Objects in bucket:");
            for obj in objects.contents() {
                println!("{:?}", obj.key().unwrap());

                let object = client
                    .get_object()
                    .bucket("yulongtest")
                    .key(obj.key().unwrap())
                    .send()
                    .await
                    .unwrap();
                let json1 = object.body.collect().await.unwrap().into_bytes();
                let value = serde_json::from_slice::<serde_json::Value>(&json1).unwrap();

                let json2 = serde_json::to_string(&value).unwrap();
                assert_eq!(json1.as_ref().len(), json2.len());
            }
        }

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(scan());

        Ok(())
    }
}
