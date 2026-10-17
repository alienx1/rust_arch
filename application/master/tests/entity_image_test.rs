//! Logo images of organizations and companies: accepted formats and addressing.
//! Run: `cargo test -p application-master --test entity_image_test`

use application_master::features::db::dbpg001::{
    ImageOwner, MAX_IMAGE_BYTES, content_type_of_key, image_url, new_image_key, sniff_image,
};
use uuid::Uuid;

#[test]
fn test_images_are_recognised_by_their_bytes_not_by_a_claim() {
    assert_eq!(sniff_image(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0]), Some(("image/png", "png")));
    assert_eq!(sniff_image(&[0xFF, 0xD8, 0xFF, 0xE0, 0]), Some(("image/jpeg", "jpg")));
    assert_eq!(sniff_image(b"GIF89a....."), Some(("image/gif", "gif")));
    assert_eq!(sniff_image(b"RIFF\x10\0\0\0WEBPVP8 "), Some(("image/webp", "webp")));
}

#[test]
fn test_svg_scripts_and_other_files_are_refused() {
    assert_eq!(sniff_image(b"<svg xmlns='http://www.w3.org/2000/svg'><script>alert(1)</script></svg>"), None);
    assert_eq!(sniff_image(b"<html><script>alert(1)</script></html>"), None);
    assert_eq!(sniff_image(b"%PDF-1.7"), None);
    assert_eq!(sniff_image(b"RIFF\x10\0\0\0WAVEfmt "), None, "RIFF that is not WebP");
    assert_eq!(sniff_image(&[]), None);
}

#[test]
fn test_key_is_unique_per_upload_and_url_carries_it_as_cache_buster() {
    let id = Uuid::now_v7();
    let (a, b) = (new_image_key(ImageOwner::Company, id, "png"), new_image_key(ImageOwner::Company, id, "png"));
    assert_ne!(a, b);
    assert!(a.starts_with(&format!("images/companies/{id}/")) && a.ends_with(".png"));

    let stem = a.rsplit('/').next().unwrap().trim_end_matches(".png");
    assert_eq!(
        image_url(ImageOwner::Company, id, Some(&a)),
        Some(format!("/api/v1/db/dbpg001/companies/{id}/image?v={stem}"))
    );
    assert!(image_url(ImageOwner::Organization, id, Some(&new_image_key(ImageOwner::Organization, id, "jpg")))
        .unwrap()
        .starts_with(&format!("/api/v1/db/dbpg001/organizations/{id}/image?v=")));
    assert_eq!(image_url(ImageOwner::Organization, id, None), None);

    // Employees are served by DBPG003
    let photo = new_image_key(ImageOwner::Employee, id, "webp");
    assert!(photo.starts_with(&format!("images/employees/{id}/")));
    assert!(image_url(ImageOwner::Employee, id, Some(&photo))
        .unwrap()
        .starts_with(&format!("/api/v1/db/dbpg003/employees/{id}/image?v=")));
}

#[test]
fn test_content_type_follows_the_stored_extension() {
    assert_eq!(content_type_of_key("images/organizations/x/y.png"), "image/png");
    assert_eq!(content_type_of_key("images/organizations/x/y.jpg"), "image/jpeg");
    assert_eq!(content_type_of_key("images/organizations/x/y.webp"), "image/webp");
    assert_eq!(content_type_of_key("images/organizations/x/y"), "application/octet-stream");
    assert_eq!(MAX_IMAGE_BYTES, 2 * 1024 * 1024);
}
