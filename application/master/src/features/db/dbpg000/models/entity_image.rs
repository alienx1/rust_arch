//! Logo images of organizations and companies: which files are accepted and how they are addressed

use uuid::Uuid;

/// Largest accepted image (the router allows a little more so this check gives the clear error)
pub const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

/// An image read back from storage
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredImage {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

/// Who owns the image: part of the storage key and of the URL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageOwner {
    Organization,
    Company,
    Employee,
}

impl ImageOwner {
    fn segment(self) -> &'static str {
        match self {
            Self::Organization => "organizations",
            Self::Company => "companies",
            Self::Employee => "employees",
        }
    }
}

/// Format of an image judged by its first bytes (never by the client's claim): PNG, JPEG, GIF and
/// WebP only. SVG is refused on purpose: it can carry scripts.
/// Returns `(content type, file extension)`.
pub fn sniff_image(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(("image/png", "png"))
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(("image/jpeg", "jpg"))
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(("image/gif", "gif"))
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(("image/webp", "webp"))
    } else {
        None
    }
}

/// Content type of a stored key (by its extension)
pub fn content_type_of_key(key: &str) -> &'static str {
    match key.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

/// A fresh storage key: `images/<owners>/<id>/<uuidv7>.<ext>` (a new name per upload, so browsers
/// never show a stale cached image)
pub fn new_image_key(owner: ImageOwner, id: Uuid, ext: &str) -> String {
    format!("images/{}/{id}/{}.{ext}", owner.segment(), Uuid::now_v7())
}

/// Program path the owner's image routes live under
fn api_base(owner: ImageOwner) -> &'static str {
    match owner {
        ImageOwner::Organization | ImageOwner::Company => "/api/v1/db/dbpg001",
        ImageOwner::Employee => "/api/v1/db/dbpg003",
    }
}

/// URL the web app puts in `<img src>`: the API path plus the key's unique name as cache buster.
/// `None` when there is no image.
pub fn image_url(owner: ImageOwner, id: Uuid, key: Option<&str>) -> Option<String> {
    let stem = key?.rsplit('/').next()?.split('.').next()?;
    Some(format!("{}/{}/{id}/image?v={stem}", api_base(owner), owner.segment()))
}
