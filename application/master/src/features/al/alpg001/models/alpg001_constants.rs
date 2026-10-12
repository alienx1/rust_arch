//! ALPG001 values

/// `status` of a process (`rp_report_job.status`)
pub mod process_status {
    pub const QUEUED: &str = "queued";
    pub const RUNNING: &str = "running";
    pub const SUCCEEDED: &str = "succeeded";
    pub const FAILED: &str = "failed";
    pub const ALL: [&str; 4] = [QUEUED, RUNNING, SUCCEEDED, FAILED];
}

/// A `queued` / `running` process that has not moved for this long is treated as stuck (the
/// worker died or the request was lost) and may be reprocessed. `started_at` is the last time it
/// was queued or started.
pub const STUCK_AFTER_SECS: i64 = 15 * 60;
pub const DEFAULT_LIMIT: u64 = 25;
pub const MAX_LIMIT: u64 = 200;
/// Kinds of process the monitor knows (`rp_report_job.report_kind`)
pub const PROCESS_KINDS: [&str; 1] = [application_core::traits::report_kind::AUDIT_LOG_CSV];
