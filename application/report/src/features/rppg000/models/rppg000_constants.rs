//! RPPG000 limits

/// Most events one audit CSV holds (a runaway filter must not run for hours)
pub const MAX_EXPORT_ROWS: u64 = 5_000_000;
/// Size of one multipart upload part (object storage wants >= 5 MiB for all but the last)
pub const UPLOAD_PART_BYTES: usize = 8 * 1024 * 1024;
/// How much of the file the preview reads
pub const PREVIEW_READ_BYTES: u64 = 512 * 1024;
pub const DEFAULT_PREVIEW_ROWS: usize = 100;
pub const MAX_PREVIEW_ROWS: usize = 500;
/// Jobs the list shows (newest first)
pub const LIST_LIMIT: u64 = 50;
/// How long a download link works
pub const DOWNLOAD_LINK_SECS: u64 = 300;
