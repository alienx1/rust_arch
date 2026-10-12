//! DBPG005 constants: paging limits and geography level tags

pub const DEFAULT_PAGE_SIZE: u64 = 25;
/// Large enough for a whole level under one parent (77 provinces, ~50 districts per province)
pub const MAX_PAGE_SIZE: u64 = 200;

/// Level of a [`GeoNodeDto`](super::GeoNodeDto)
pub const LEVEL_PROVINCE: &str = "province";
pub const LEVEL_DISTRICT: &str = "district";
pub const LEVEL_SUB_DISTRICT: &str = "sub_district";
