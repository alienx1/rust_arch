//! DBPG006 constants: page sizes and field limits

pub const DEFAULT_PAGE_SIZE: u64 = 25;
pub const MAX_PAGE_SIZE: u64 = 200;

/// `db_label.category` is `varchar(100)`
pub const MAX_CATEGORY_LEN: usize = 100;
pub const MAX_KEY_LEN: usize = 300;
pub const MAX_VALUE_LEN: usize = 2000;
