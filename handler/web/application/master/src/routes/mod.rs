//! Route definitions for the Master SPA.
//!
//! - `program_paths`: program code → URL path table
//! - `master_routes`: `MasterRoutes` component holding the `<Routes>` tree

pub mod master_routes;
pub mod program_paths;

pub use master_routes::MasterRoutes;
pub use program_paths::*;
