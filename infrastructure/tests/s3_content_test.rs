//! Object storage adapter against a live S3 / Garage (`AWS_*` and `S3_BUCKET_NAME` from `.env`):
//! multipart upload, ranged read, size and the signed download link.
//! `cargo test -p infrastructure --test s3_content_test -- --ignored`

use application_core::traits::ContentService;
use infrastructure::services::{S3ContentService, s3_client_from_env};

#[tokio::test]
#[ignore]
async fn live_multipart_upload_range_read_and_signed_download() {
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env")).ok();
    let bucket = std::env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "rust-arch-bucket".into());
    let s3 = S3ContentService::new(s3_client_from_env().await);
    let key = format!("tests/multipart-{}.bin", uuid::Uuid::now_v7());

    // Two parts: all but the last must be >= 5 MiB
    let part1 = vec![b'a'; 5 * 1024 * 1024];
    let part2 = b"tail-of-the-file".to_vec();
    let upload = s3.begin_multipart(&bucket, &key, Some("text/csv")).await.expect("begin");
    let e1 = s3.upload_part(&bucket, &key, &upload, 1, part1.clone()).await.expect("part 1");
    let e2 = s3.upload_part(&bucket, &key, &upload, 2, part2.clone()).await.expect("part 2");
    s3.complete_multipart(&bucket, &key, &upload, vec![(1, e1), (2, e2)]).await.expect("complete");

    assert_eq!(s3.size(&bucket, &key).await.unwrap(), (part1.len() + part2.len()) as u64);
    assert_eq!(s3.get_range(&bucket, &key, 0, 9).await.unwrap(), vec![b'a'; 10]);
    let tail_from = part1.len() as u64;
    assert_eq!(s3.get_range(&bucket, &key, tail_from, tail_from + 1000).await.unwrap(), part2, "range past the end is cut");

    let url = s3.presign_download_url(&bucket, &key, "my report.csv", 60).await.unwrap();
    assert!(url.contains("response-content-disposition"), "{url}");

    // An upload that is aborted leaves no object
    let key2 = format!("tests/aborted-{}.bin", uuid::Uuid::now_v7());
    let up2 = s3.begin_multipart(&bucket, &key2, None).await.unwrap();
    s3.upload_part(&bucket, &key2, &up2, 1, part1).await.unwrap();
    s3.abort_multipart(&bucket, &key2, &up2).await.unwrap();
    assert!(s3.size(&bucket, &key2).await.is_err());

    s3.delete(&bucket, &key).await.unwrap();
}
