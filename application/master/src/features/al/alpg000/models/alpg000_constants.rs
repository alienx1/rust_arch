//! ALPG000 limits

pub const DEFAULT_PAGE_SIZE: u64 = 25;
pub const MAX_PAGE_SIZE: u64 = 200;
/// Longest free-text search accepted
pub const MAX_TEXT_LEN: usize = 200;
/// Events accepted in one `POST /audit/events`
pub const MAX_CLIENT_EVENTS: usize = 50;
pub const MAX_PAGE_FIELD_LEN: usize = 300;
pub const MAX_ELEMENT_LEN: usize = 200;
pub const MAX_LABEL_LEN: usize = 300;
pub const MAX_DETAIL_LEN: usize = 1000;
