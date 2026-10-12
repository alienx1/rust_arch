//! What the signed-in account may do, per program (READ / WRITE / APPROVE), for hiding the
//! buttons it cannot use. The APIs enforce the permissions themselves; this is only the UI side.

pub mod access_context;
pub mod permission_gate;

pub use access_context::{Action, can, provide_access_context, use_access};
pub use permission_gate::PermissionGate;
