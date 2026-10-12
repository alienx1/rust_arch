//! Generic paginated result DTO

use serde::{Deserialize, Serialize};

/// โครงสร้างผลลัพธ์แบบแบ่งหน้า (Paginated Result)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}
