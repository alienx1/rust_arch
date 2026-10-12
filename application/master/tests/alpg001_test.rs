//! Tests for ALPG001 (Process Monitor): filter rules, DTO flags and the reprocess flow over an
//! in-memory job store. Located at `application/master/tests/alpg001_test.rs`.

use application_core::errors::EventError;
use application_core::traits::{EventService, REPORT_REQUESTS_TOPIC, ReportJobMessage};
use application_master::features::al::alpg001::{
    Alpg001CommandExecutor, Alpg001Commands, Alpg001Error, Alpg001Queries, ProcessCountsDto, ProcessFilterQuery,
    ProcessModel, ProcessSearch, normalize_process_filter,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Job rows in memory; `mark_queued` follows the same rule as the SQL (finished, or waiting /
/// running since before `stuck_before`)
#[derive(Default)]
struct MemoryJobs {
    rows: Mutex<HashMap<Uuid, ProcessModel>>,
    searched: Mutex<Vec<ProcessSearch>>,
}

#[async_trait]
impl Alpg001Queries for MemoryJobs {
    async fn search(&self, f: &ProcessSearch) -> Result<(Vec<ProcessModel>, u64, ProcessCountsDto), Alpg001Error> {
        self.searched.lock().unwrap().push(f.clone());
        let rows: Vec<ProcessModel> = self.rows.lock().unwrap().values().cloned().collect();
        let total = rows.len() as u64;
        let mut counts = ProcessCountsDto::default();
        for r in &rows {
            match r.status.as_str() {
                "queued" => counts.queued += 1,
                "running" => counts.running += 1,
                "succeeded" => counts.succeeded += 1,
                _ => counts.failed += 1,
            }
        }
        Ok((rows, total, counts))
    }

    async fn find(&self, id: Uuid) -> Result<Option<ProcessModel>, Alpg001Error> {
        Ok(self.rows.lock().unwrap().get(&id).cloned())
    }

    async fn mark_queued(&self, id: Uuid, now: DateTime<Utc>, stuck_before: DateTime<Utc>) -> Result<bool, Alpg001Error> {
        let mut rows = self.rows.lock().unwrap();
        let Some(row) = rows.get_mut(&id) else { return Ok(false) };
        let started = row.started_at.map(|t| t.with_timezone(&Utc));
        let allowed = matches!(row.status.as_str(), "succeeded" | "failed")
            || (matches!(row.status.as_str(), "queued" | "running") && started.is_some_and(|s| s <= stuck_before));
        if allowed {
            row.status = "queued".into();
            row.error = None;
            row.started_at = Some(now.fixed_offset());
            row.finished_at = None;
        }
        Ok(allowed)
    }

    async fn restore(&self, row: ProcessModel) -> Result<(), Alpg001Error> {
        self.rows.lock().unwrap().insert(row.id, row);
        Ok(())
    }
}

#[derive(Default)]
struct MockEvents {
    published: Mutex<Vec<(String, Option<String>, Vec<u8>)>>,
    fail: bool,
}

#[async_trait]
impl EventService for MockEvents {
    async fn publish(&self, topic: &str, key: Option<&str>, payload: &[u8]) -> Result<(), EventError> {
        if self.fail {
            return Err(EventError::PublishFailed { topic: topic.into(), reason: "broker down".into() });
        }
        self.published.lock().unwrap().push((topic.into(), key.map(str::to_string), payload.to_vec()));
        Ok(())
    }
}

fn job(status: &str, started_mins_ago: Option<i64>, error: Option<&str>) -> ProcessModel {
    let now = Utc::now();
    ProcessModel {
        id: Uuid::now_v7(),
        report_kind: "audit_log_csv".into(),
        requested_by: "somchai".into(),
        filter: serde_json::json!({ "kind": "api", "page": 1, "limit": 25 }),
        status: status.into(),
        row_count: 0,
        size_bytes: 0,
        file_name: None,
        object_key: None,
        error: error.map(str::to_string),
        requested_at: (now - Duration::hours(2)).fixed_offset(),
        started_at: started_mins_ago.map(|m| (now - Duration::minutes(m)).fixed_offset()),
        finished_at: matches!(status, "succeeded" | "failed").then(|| now.fixed_offset()),
        row_version: 1,
    }
}

fn rig(rows: Vec<ProcessModel>, fail_publish: bool) -> (Alpg001CommandExecutor, Arc<MemoryJobs>, Arc<MockEvents>) {
    let jobs = Arc::new(MemoryJobs::default());
    for r in rows {
        jobs.rows.lock().unwrap().insert(r.id, r);
    }
    let events = Arc::new(MockEvents { fail: fail_publish, ..Default::default() });
    let exec = Alpg001CommandExecutor::new(jobs.clone()).with_events(events.clone());
    (exec, jobs, events)
}

// ---- filter ----

#[test]
fn test_filter_defaults_and_clamps_paging() {
    let f = normalize_process_filter(ProcessFilterQuery::default()).unwrap();
    assert_eq!((f.page, f.limit), (1, 25));
    let f = normalize_process_filter(ProcessFilterQuery { page: Some(0), limit: Some(9999), ..Default::default() }).unwrap();
    assert_eq!((f.page, f.limit), (1, 200));
}

#[test]
fn test_filter_normalizes_known_values_and_rejects_unknown_ones() {
    let f = normalize_process_filter(ProcessFilterQuery {
        status: Some(" FAILED ".into()),
        kind: Some("audit_log_csv".into()),
        user: Some("  ".into()),
        from: Some("2026-10-11T17:00:00Z".into()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(f.status.as_deref(), Some("failed"));
    assert_eq!(f.user, None, "blank values are dropped");
    assert!(f.from.is_some());

    for bad in [
        ProcessFilterQuery { status: Some("done".into()), ..Default::default() },
        ProcessFilterQuery { kind: Some("nope".into()), ..Default::default() },
        ProcessFilterQuery { to: Some("yesterday".into()), ..Default::default() },
    ] {
        assert!(matches!(normalize_process_filter(bad), Err(Alpg001Error::Validation(_))));
    }
}

// ---- list / get ----

#[tokio::test]
async fn test_list_flags_what_can_be_reprocessed() {
    let (exec, _, _) = rig(
        vec![
            job("failed", Some(5), Some("quickwit down")),
            job("succeeded", Some(5), None),
            job("running", Some(2), None),
            job("running", Some(40), None),
            job("queued", Some(1), None),
        ],
        false,
    );
    let page = exec.list_processes(ProcessFilterQuery::default()).await.unwrap();
    assert_eq!(page.total, 5);
    assert_eq!((page.counts.failed, page.counts.succeeded, page.counts.running, page.counts.queued), (1, 1, 2, 1));
    let by = |status: &str, stuck: bool| page.items.iter().find(|p| p.status == status && p.stuck == stuck).unwrap();
    assert!(by("failed", false).can_reprocess);
    assert_eq!(by("failed", false).error.as_deref(), Some("quickwit down"));
    assert!(by("succeeded", false).can_reprocess);
    assert!(!by("running", false).can_reprocess, "a fresh run must finish first");
    assert!(by("running", true).can_reprocess, "a run that has not moved for 15 minutes is stuck");
    assert!(!by("queued", false).can_reprocess);
    assert!(by("failed", false).duration_ms.is_some());
    assert!(by("running", false).duration_ms.is_none());
}

#[tokio::test]
async fn test_get_process_needs_a_uuid_and_may_find_nothing() {
    let row = job("failed", Some(5), Some("x"));
    let id = row.id;
    let (exec, _, _) = rig(vec![row], false);
    assert_eq!(exec.get_process(&id.to_string()).await.unwrap().unwrap().id, id.to_string());
    assert!(exec.get_process(&Uuid::now_v7().to_string()).await.unwrap().is_none());
    assert!(matches!(exec.get_process("nope").await, Err(Alpg001Error::Validation(_))));
}

// ---- reprocess ----

#[tokio::test]
async fn test_reprocess_queues_the_row_and_publishes_the_original_request_flagged() {
    let row = job("failed", Some(5), Some("quickwit down"));
    let id = row.id;
    let (exec, jobs, events) = rig(vec![row], false);

    let done = exec.reprocess(&id.to_string()).await.unwrap();
    assert_eq!(done.id, id.to_string());

    let stored = jobs.rows.lock().unwrap()[&id].clone();
    assert_eq!(stored.status, "queued");
    assert_eq!(stored.error, None, "the old failure is cleared");

    let published = events.published.lock().unwrap();
    assert_eq!(published.len(), 1);
    let (topic, key, payload) = &published[0];
    assert_eq!(topic, REPORT_REQUESTS_TOPIC);
    assert_eq!(key.as_deref(), Some(id.to_string().as_str()));
    let message: ReportJobMessage = serde_json::from_slice(payload).unwrap();
    assert!(message.reprocess);
    assert_eq!((message.job_id.as_str(), message.requested_by.as_str()), (id.to_string().as_str(), "somchai"));
    assert_eq!(message.filter.kind.as_deref(), Some("api"), "the stored request is what runs again");
}

#[tokio::test]
async fn test_a_second_reprocess_while_queued_is_a_conflict_and_publishes_nothing() {
    let row = job("failed", Some(5), Some("x"));
    let id = row.id.to_string();
    let (exec, _, events) = rig(vec![row], false);
    exec.reprocess(&id).await.unwrap();
    assert!(matches!(exec.reprocess(&id).await, Err(Alpg001Error::Conflict(_))));
    assert_eq!(events.published.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_a_running_process_cannot_be_reprocessed_but_a_stuck_one_can() {
    let fresh = job("running", Some(1), None);
    let stuck = job("running", Some(60), None);
    let (fresh_id, stuck_id) = (fresh.id.to_string(), stuck.id.to_string());
    let (exec, _, events) = rig(vec![fresh, stuck], false);
    assert!(matches!(exec.reprocess(&fresh_id).await, Err(Alpg001Error::Conflict(_))));
    exec.reprocess(&stuck_id).await.unwrap();
    assert_eq!(events.published.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_reprocess_of_a_missing_or_unreadable_process() {
    let mut broken = job("failed", Some(5), Some("x"));
    broken.filter = serde_json::json!({ "limit": "many" });
    let broken_id = broken.id.to_string();
    let (exec, _, events) = rig(vec![broken], false);
    assert!(matches!(exec.reprocess(&Uuid::now_v7().to_string()).await, Err(Alpg001Error::NotFound(_))));
    assert!(matches!(exec.reprocess(&broken_id).await, Err(Alpg001Error::Validation(_))));
    assert!(matches!(exec.reprocess("nope").await, Err(Alpg001Error::Validation(_))));
    assert!(events.published.lock().unwrap().is_empty());
}

#[tokio::test]
async fn test_a_broker_that_is_down_leaves_the_row_as_it_was() {
    let row = job("succeeded", Some(5), None);
    let id = row.id;
    let before = row.clone();
    let (exec, jobs, _) = rig(vec![row], true);
    assert!(matches!(exec.reprocess(&id.to_string()).await, Err(Alpg001Error::Unavailable(_))));
    let stored = jobs.rows.lock().unwrap()[&id].clone();
    assert_eq!((stored.status.as_str(), stored.finished_at), ("succeeded", before.finished_at));
}

#[tokio::test]
async fn test_without_an_event_stream_reprocess_is_unavailable() {
    let row = job("failed", Some(5), Some("x"));
    let id = row.id.to_string();
    let jobs = Arc::new(MemoryJobs::default());
    jobs.rows.lock().unwrap().insert(row.id, row);
    let exec = Alpg001CommandExecutor::new(jobs);
    assert!(matches!(exec.reprocess(&id).await, Err(Alpg001Error::Unavailable(_))));
}
