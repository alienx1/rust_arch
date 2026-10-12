//! SUPG003 matrix action: the permission actions in matrix column order

use serde::{Deserialize, Serialize};

/// Permission actions, in matrix column order
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatrixAction {
    Read,
    Write,
    Approve,
}

impl MatrixAction {
    pub const ALL: [MatrixAction; 3] = [MatrixAction::Read, MatrixAction::Write, MatrixAction::Approve];

    /// `db_master_lookup.code` (category `permission_type`)
    pub fn code(self) -> &'static str {
        match self {
            MatrixAction::Read => "READ",
            MatrixAction::Write => "WRITE",
            MatrixAction::Approve => "APPROVE",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.code().eq_ignore_ascii_case(code))
    }
}
