//! ALPG000 program option DTO: a catalogue program the audit log can be filtered by

use serde::{Deserialize, Serialize};

/// A program page of the catalogue (`db_program` with a route) for the program filter
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramOptionDto {
    /// Program code, e.g. `DBPG004`
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}
