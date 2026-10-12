//! Models and Data Types for Static UI Component Showcase

use serde::{Deserialize, Serialize};

/// Mock user record for table and avatar showcase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockUserItem {
    pub id: &'static str,
    pub name: &'static str,
    pub email: &'static str,
    pub role: &'static str,
    pub department: &'static str,
    pub status: &'static str,
}
