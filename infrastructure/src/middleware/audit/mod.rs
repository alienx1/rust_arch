//! Audit trail: one event per API request (body capture and masking, program/action derivation).

pub mod audit_body;
pub mod audit_target;
pub mod audit_trail;

pub use audit_target::{derive_audit_target, should_audit};
pub use audit_trail::{AuditTrailState, audit_trail};
