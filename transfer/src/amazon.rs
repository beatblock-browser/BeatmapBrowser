use anyhow::Error;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_sts as sts;

pub const MAPS_TABLE_NAME: &str = "beatmapbrowser-maps";
pub const USERS_TABLE_NAME: &str = "beatmapbrowser-users";
pub const BUCKET_REGION: &str = "us-east-2";

#[derive(Clone)]
pub struct Amazon {
    pub db_client: DynamoClient,
}

pub async fn setup() -> Result<Amazon, Error> {
    // Use the default credential chain from environment/profile/IMDS, and force region
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(BUCKET_REGION))
        .load()
        .await;

    // Diagnostics: print resolved region and caller identity to confirm creds work
    let resolved_region = shared_config
        .region()
        .map(|r| r.as_ref().to_string())
        .unwrap_or_else(|| "<none>".to_string());
    println!("AWS resolved region: {}", resolved_region);

    let sts_client = sts::Client::new(&shared_config);
    match sts_client.get_caller_identity().send().await {
        Ok(id) => {
            let account = id.account().unwrap_or("<unknown>");
            let arn = id.arn().unwrap_or("<unknown>");
            println!("STS GetCallerIdentity OK. Account={} ARN={}", account, arn);
        }
        Err(e) => {
            println!(
                "STS GetCallerIdentity failed: {}\nHint: check AWS_* env vars for hidden newlines/CR, AWS_REGION, and system clock.",
                e
            );
        }
    }
    Ok(Amazon { db_client: DynamoClient::new(&shared_config) })
}
// Minimal Amazon surface for the transfer binary.