//! Builds the S3 client from the environment (`AWS_ENDPOINT_URL`, `AWS_REGION`,
//! `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`). Path-style addressing is forced: Garage (and
//! MinIO) serve `endpoint/bucket/key`, not `bucket.endpoint`.

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

pub async fn s3_client_from_env() -> Client {
    let shared = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let config = aws_sdk_s3::config::Builder::from(&shared).force_path_style(true).build();
    Client::from_conf(config)
}
