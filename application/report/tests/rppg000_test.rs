//! Tests for RPPG000 (report worker): the job runner over in-memory ports (queries, audit store,
//! object storage, notifications), preview, download, ownership and the CSV rows. Located at
//! `application/report/tests/rppg000_test.rs`.

use application_core::errors::ContentError;
use application_core::traits::{
    AuditError, AuditEvent, AuditExportChunk, AuditLogPort, AuditPage, AuditSearch, ContentService, EventHandler,
    Notification, NotificationService, ReportJobMessage, report_kind,
};
use application_report::features::rppg000::commands::audit_csv_rows::{
    BOM, HEADER, changes_text, escape_field, event_line, header_line,
};
use application_report::features::rppg000::{
    ReportJobHandler, ReportJobModel, Rppg000CommandExecutor, Rppg000Commands, Rppg000Error, Rppg000Queries,
    UPLOAD_PART_BYTES, preview_rows,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ---- in-memory ports ----

#[derive(Default)]
struct MemoryJobs {
    rows: Mutex<HashMap<Uuid, ReportJobModel>>,
}

#[async_trait]
impl Rppg000Queries for MemoryJobs {
    async fn start_job(
        &self,
        id: Uuid,
        kind: &str,
        by: &str,
        filter: Value,
        at: DateTime<Utc>,
    ) -> Result<(), Rppg000Error> {
        let mut rows = self.rows.lock().unwrap();
        let row = rows.entry(id).or_insert_with(|| ReportJobModel {
            id,
            report_kind: kind.into(),
            requested_by: by.into(),
            filter,
            status: "running".into(),
            row_count: 0,
            size_bytes: 0,
            file_name: None,
            object_key: None,
            error: None,
            requested_at: at.fixed_offset(),
            started_at: None,
            finished_at: None,
            row_version: 1,
        });
        row.status = "running".into();
        Ok(())
    }
    async fn find_job(&self, id: Uuid) -> Result<Option<ReportJobModel>, Rppg000Error> {
        Ok(self.rows.lock().unwrap().get(&id).cloned())
    }
    async fn finish_job_ok(&self, id: Uuid, rows: i64, size: i64, name: &str, key: &str) -> Result<(), Rppg000Error> {
        let mut all = self.rows.lock().unwrap();
        let row = all.get_mut(&id).unwrap();
        row.status = "succeeded".into();
        row.row_count = rows;
        row.size_bytes = size;
        row.file_name = Some(name.into());
        row.object_key = Some(key.into());
        Ok(())
    }
    async fn finish_job_failed(&self, id: Uuid, error: &str) -> Result<(), Rppg000Error> {
        let mut all = self.rows.lock().unwrap();
        let row = all.get_mut(&id).unwrap();
        row.status = "failed".into();
        row.error = Some(error.into());
        Ok(())
    }
    async fn list_jobs(&self, by: &str, limit: u64) -> Result<Vec<ReportJobModel>, Rppg000Error> {
        let mut rows: Vec<_> =
            self.rows.lock().unwrap().values().filter(|r| r.requested_by == by).cloned().collect();
        rows.sort_by(|a, b| b.requested_at.cmp(&a.requested_at));
        rows.truncate(limit as usize);
        Ok(rows)
    }
}

/// Serves `pages` slices of `per_page` events; fails on slice `fail_at` when set
struct PagedAudit {
    pages: usize,
    per_page: usize,
    fail_at: Option<usize>,
}

fn audit(pages: usize, per_page: usize, fail_at: Option<usize>) -> PagedAudit {
    PagedAudit { pages, per_page, fail_at }
}

#[async_trait]
impl AuditLogPort for PagedAudit {
    fn record(&self, _events: Vec<AuditEvent>) {}
    async fn search(&self, f: AuditSearch) -> Result<AuditPage, AuditError> {
        Ok(AuditPage { items: vec![], total: 0, page: f.page, limit: f.limit })
    }
    async fn get(&self, _id: &str) -> Result<Option<AuditEvent>, AuditError> {
        Ok(None)
    }
    async fn is_available(&self) -> bool {
        true
    }
    async fn export_chunk(&self, _f: AuditSearch, cursor: Option<String>) -> Result<AuditExportChunk, AuditError> {
        let index: usize = cursor.as_deref().map(|c| c.parse().unwrap()).unwrap_or(0);
        if self.fail_at == Some(index) {
            return Err(AuditError::Unavailable("quickwit".into()));
        }
        let items = (0..self.per_page)
            .map(|i| AuditEvent {
                event_id: format!("e{index}-{i}"),
                timestamp: "2026-10-11T17:30:05.250Z".into(),
                kind: "api".into(),
                username: Some("admin".into()),
                path: Some("/api/v1/x".into()),
                ..AuditEvent::default()
            })
            .collect();
        let next = (index + 1 < self.pages).then(|| (index + 1).to_string());
        Ok(AuditExportChunk { items, cursor: next })
    }
}

#[derive(Default)]
struct MemoryStorage {
    objects: Mutex<HashMap<String, Vec<u8>>>,
    uploads: Mutex<HashMap<String, Vec<(i32, Vec<u8>)>>>,
    aborted: Mutex<Vec<String>>,
    parts_sent: Mutex<usize>,
}

#[async_trait]
impl ContentService for MemoryStorage {
    async fn put(&self, _b: &str, key: &str, data: Vec<u8>, _ct: Option<&str>) -> Result<(), ContentError> {
        self.objects.lock().unwrap().insert(key.into(), data);
        Ok(())
    }
    async fn get(&self, _b: &str, key: &str) -> Result<Vec<u8>, ContentError> {
        self.objects.lock().unwrap().get(key).cloned().ok_or(ContentError::NotFound(key.into()))
    }
    async fn delete(&self, _b: &str, _k: &str) -> Result<(), ContentError> {
        Ok(())
    }
    async fn presign_get_url(&self, _b: &str, key: &str, _e: u64) -> Result<String, ContentError> {
        Ok(format!("http://storage/{key}"))
    }
    async fn get_range(&self, b: &str, key: &str, from: u64, to: u64) -> Result<Vec<u8>, ContentError> {
        let all = self.get(b, key).await?;
        let end = (to as usize + 1).min(all.len());
        Ok(all[(from as usize).min(end)..end].to_vec())
    }
    async fn size(&self, b: &str, key: &str) -> Result<u64, ContentError> {
        Ok(self.get(b, key).await?.len() as u64)
    }
    async fn presign_download_url(&self, _b: &str, key: &str, name: &str, _e: u64) -> Result<String, ContentError> {
        Ok(format!("http://storage/{key}?name={name}"))
    }
    async fn begin_multipart(&self, _b: &str, key: &str, _ct: Option<&str>) -> Result<String, ContentError> {
        self.uploads.lock().unwrap().insert(key.into(), Vec::new());
        Ok(format!("upload-{key}"))
    }
    async fn upload_part(&self, _b: &str, key: &str, _u: &str, n: i32, data: Vec<u8>) -> Result<String, ContentError> {
        *self.parts_sent.lock().unwrap() += 1;
        self.uploads.lock().unwrap().get_mut(key).unwrap().push((n, data));
        Ok(format!("etag-{n}"))
    }
    async fn complete_multipart(
        &self,
        _b: &str,
        key: &str,
        _u: &str,
        parts: Vec<(i32, String)>,
    ) -> Result<(), ContentError> {
        let uploaded = self.uploads.lock().unwrap().remove(key).unwrap();
        let numbers: Vec<i32> = parts.iter().map(|(n, _)| *n).collect();
        assert_eq!(numbers, (1..=uploaded.len() as i32).collect::<Vec<_>>(), "parts completed in order");
        self.objects.lock().unwrap().insert(key.into(), uploaded.into_iter().flat_map(|(_, d)| d).collect());
        Ok(())
    }
    async fn abort_multipart(&self, _b: &str, key: &str, _u: &str) -> Result<(), ContentError> {
        self.uploads.lock().unwrap().remove(key);
        self.aborted.lock().unwrap().push(key.into());
        Ok(())
    }
}

#[derive(Default)]
struct RecordingNotifier {
    sent: Mutex<Vec<(String, Notification)>>,
}

#[async_trait]
impl NotificationService for RecordingNotifier {
    async fn notify(&self, username: &str, n: Notification) {
        self.sent.lock().unwrap().push((username.into(), n));
    }
}

struct Rig {
    exec: Arc<Rppg000CommandExecutor>,
    jobs: Arc<MemoryJobs>,
    storage: Arc<MemoryStorage>,
    notifier: Arc<RecordingNotifier>,
}

fn rig(audit: PagedAudit) -> Rig {
    let jobs = Arc::new(MemoryJobs::default());
    let storage = Arc::new(MemoryStorage::default());
    let notifier = Arc::new(RecordingNotifier::default());
    let exec = Arc::new(Rppg000CommandExecutor::new(
        jobs.clone(),
        Arc::new(audit),
        storage.clone(),
        notifier.clone(),
        "bucket",
    ));
    Rig { exec, jobs, storage, notifier }
}

fn message(job_id: Uuid, by: &str) -> ReportJobMessage {
    ReportJobMessage {
        job_id: job_id.to_string(),
        report_kind: report_kind::AUDIT_LOG_CSV.into(),
        requested_by: by.into(),
        requested_at: "2026-10-11T17:30:00Z".into(),
        filter: AuditSearch::default(),
        reprocess: false,
    }
}

// ---- the worker ----

#[tokio::test]
async fn test_job_streams_every_slice_into_one_file_and_notifies_the_requester() {
    let r = rig(audit(3, 4, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;

    let job = r.jobs.find_job(id).await.unwrap().unwrap();
    assert_eq!((job.status.as_str(), job.row_count), ("succeeded", 12));
    let key = job.object_key.clone().unwrap();
    assert_eq!(key, format!("reports/{id}.csv"));
    let file = String::from_utf8(r.storage.objects.lock().unwrap()[&key].clone()).unwrap();
    assert!(file.starts_with(BOM) && file[BOM.len()..].starts_with(&header_line()));
    assert_eq!(file.matches("\r\n").count(), 1 + 12, "header + 12 events");
    assert_eq!(job.size_bytes as usize, file.len());
    assert!(job.file_name.unwrap().starts_with("audit-log-"));

    let sent = r.notifier.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!((sent[0].0.as_str(), sent[0].1.kind.as_str()), ("somchai", "report_ready"));
    assert_eq!(sent[0].1.data["job_id"], id.to_string());
}

#[tokio::test]
async fn test_a_file_bigger_than_one_part_is_uploaded_in_ordered_parts_without_loss() {
    // ~90 bytes a row: 120k rows is ~10 MB, more than one part
    let r = rig(audit(24, 5_000, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    let job = r.jobs.find_job(id).await.unwrap().unwrap();
    assert_eq!((job.status.as_str(), job.row_count), ("succeeded", 120_000));
    assert!(*r.storage.parts_sent.lock().unwrap() >= 2, "multipart was used");
    let file = r.storage.objects.lock().unwrap()[&format!("reports/{id}.csv")].clone();
    assert_eq!(file.len() as i64, job.size_bytes, "the assembled object is exactly what was written");
    assert_eq!(String::from_utf8(file).unwrap().matches("\r\n").count(), 120_001);
    assert!(UPLOAD_PART_BYTES >= 5 * 1024 * 1024, "parts meet the object-store minimum");
}

#[tokio::test]
async fn test_a_store_failure_midway_fails_the_job_aborts_the_upload_and_notifies() {
    let r = rig(audit(24, 5_000, Some(22)));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    let job = r.jobs.find_job(id).await.unwrap().unwrap();
    assert_eq!(job.status, "failed");
    assert!(job.error.unwrap().contains("quickwit"));
    assert_eq!(r.storage.aborted.lock().unwrap().len(), 1, "half a file is not left behind");
    assert!(r.storage.objects.lock().unwrap().is_empty());
    assert_eq!(r.notifier.sent.lock().unwrap()[0].1.kind, "report_failed");
}

#[tokio::test]
async fn test_a_redelivered_request_of_a_finished_job_is_not_built_twice() {
    let r = rig(audit(1, 2, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    r.exec.run_job(message(id, "somchai")).await;
    assert_eq!(r.notifier.sent.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_a_reprocess_request_reruns_a_queued_job_once() {
    let r = rig(audit(1, 2, Some(0)));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    assert_eq!(r.jobs.find_job(id).await.unwrap().unwrap().status, "failed");

    // ALPG001 puts the row back to `queued` and publishes the request again
    r.jobs.rows.lock().unwrap().get_mut(&id).unwrap().status = "queued".to_string();
    let again = ReportJobMessage { reprocess: true, ..message(id, "somchai") };
    // (the store still fails here, so the job fails again: what matters is that it ran)
    r.exec.run_job(again.clone()).await;
    assert_eq!(r.notifier.sent.lock().unwrap().len(), 2, "the re-run notified the requester again");

    // A redelivery of the same reprocess request after it finished is a no-op
    r.exec.run_job(again).await;
    assert_eq!(r.notifier.sent.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn test_a_reprocess_request_for_a_finished_or_unknown_job_is_skipped() {
    let r = rig(audit(1, 2, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    let again = ReportJobMessage { reprocess: true, ..message(id, "somchai") };
    r.exec.run_job(again).await;
    r.exec.run_job(ReportJobMessage { reprocess: true, ..message(Uuid::now_v7(), "somchai") }).await;
    assert_eq!(r.notifier.sent.lock().unwrap().len(), 1, "nothing ran twice, nothing ran for a missing row");
}

#[tokio::test]
async fn test_an_unknown_kind_fails_the_job_and_a_bad_id_is_skipped() {
    let r = rig(audit(1, 1, None));
    let id = Uuid::now_v7();
    let mut m = message(id, "somchai");
    m.report_kind = "mystery".into();
    r.exec.run_job(m).await;
    assert_eq!(r.jobs.find_job(id).await.unwrap().unwrap().status, "failed");

    let mut bad = message(id, "somchai");
    bad.job_id = "not-a-uuid".into();
    r.exec.run_job(bad).await; // must not panic
}

#[tokio::test]
async fn test_handler_runs_readable_messages_and_skips_garbage() {
    let r = rig(audit(1, 3, None));
    let handler = ReportJobHandler::new(r.exec.clone());
    handler.handle(None, b"not json".to_vec()).await;
    assert!(r.notifier.sent.lock().unwrap().is_empty());
    let id = Uuid::now_v7();
    handler.handle(Some(id.to_string()), serde_json::to_vec(&message(id, "somchai")).unwrap()).await;
    assert_eq!(r.jobs.find_job(id).await.unwrap().unwrap().status, "succeeded");
}

// ---- reading: list, preview, download, ownership ----

#[tokio::test]
async fn test_only_the_requester_sees_a_job() {
    let r = rig(audit(1, 3, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    assert_eq!(r.exec.list_jobs("somchai").await.unwrap().len(), 1);
    assert!(r.exec.list_jobs("somying").await.unwrap().is_empty());
    assert!(r.exec.get_job("somchai", &id.to_string()).await.is_ok());
    for result in [
        r.exec.get_job("somying", &id.to_string()).await.map(|_| ()),
        r.exec.preview_job("somying", &id.to_string(), None).await.map(|_| ()),
        r.exec.download_url("somying", &id.to_string()).await.map(|_| ()),
        r.exec.get_job("somchai", "not-a-uuid").await.map(|_| ()),
    ] {
        assert!(matches!(result, Err(Rppg000Error::NotFound(_))), "{result:?}");
    }
}

#[tokio::test]
async fn test_preview_shows_columns_and_the_first_rows() {
    let r = rig(audit(1, 30, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    let p = r.exec.preview_job("somchai", &id.to_string(), Some(10)).await.unwrap();
    assert_eq!(p.columns.len(), HEADER.len());
    assert_eq!(p.columns[0], "time_th");
    assert_eq!((p.rows.len(), p.total_rows, p.truncated), (10, 30, true));
    assert_eq!(p.rows[0][0], "2026-10-12 00:30:05");
    let all = r.exec.preview_job("somchai", &id.to_string(), Some(9_999)).await.unwrap();
    assert_eq!((all.rows.len(), all.truncated), (30, false), "capped at the file, not at the request");
}

#[tokio::test]
async fn test_download_is_a_named_signed_link_and_a_failed_job_has_none() {
    let r = rig(audit(1, 1, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    let url = r.exec.download_url("somchai", &id.to_string()).await.unwrap();
    assert!(url.contains(&format!("reports/{id}.csv")) && url.contains("name=audit-log-"), "{url}");

    let failing = rig(audit(2, 1, Some(1)));
    let bad = Uuid::now_v7();
    failing.exec.run_job(message(bad, "somchai")).await;
    assert!(matches!(failing.exec.download_url("somchai", &bad.to_string()).await, Err(Rppg000Error::NotReady(_))));
}

#[test]
fn test_preview_rows_drops_a_record_cut_in_half() {
    let csv = "\u{feff}a,b\r\n1,2\r\n3,\"x\r\ny\"\r\n5,6";
    let (cols, rows) = preview_rows(csv.as_bytes(), 10, false);
    assert_eq!((cols, rows.len()), (vec!["a".to_string(), "b".to_string()], 3));
    assert_eq!(rows[1], vec!["3".to_string(), "x\r\ny".to_string()], "quoted line breaks stay in the cell");
    let (_, cut) = preview_rows(csv.as_bytes(), 10, true);
    assert_eq!(cut.len(), 2, "the last record of a cut read may be incomplete");
    assert_eq!(preview_rows(csv.as_bytes(), 1, false).1.len(), 1);
}

// ---- CSV rows ----

#[test]
fn test_csv_line_has_one_cell_per_header_and_two_clocks() {
    let e = AuditEvent {
        event_id: "e1".into(),
        timestamp: "2026-10-11T17:30:05.250Z".into(),
        kind: "api".into(),
        username: Some("admin".into()),
        method: Some("POST".into()),
        path: Some("/api/v1/db/dbpg002".into()),
        status: Some(200),
        duration_ms: Some(12),
        ..AuditEvent::default()
    };
    let line = event_line(&e);
    assert!(line.ends_with("\r\n"));
    assert!(line.starts_with("2026-10-12 00:30:05,2026-10-11T17:30:05.250Z,api,"), "{line}");
    assert_eq!(line.trim_end().split(',').count(), HEADER.len());
}

#[test]
fn test_csv_escapes_and_defuses_formulas() {
    assert_eq!(escape_field("a,b"), "\"a,b\"");
    assert_eq!(escape_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(escape_field("line\nbreak"), "\"line\nbreak\"");
    let e = AuditEvent {
        kind: "api".into(),
        username: Some("=HYPERLINK(\"http://x\")".into()),
        path: Some("-1+1".into()),
        ..AuditEvent::default()
    };
    let line = event_line(&e);
    assert!(line.contains("\"'=HYPERLINK(\"\"http://x\"\")\""), "{line}");
    assert!(line.contains(",'-1+1,"), "{line}");
}

#[test]
fn test_changes_are_one_readable_line_per_row() {
    let json = serde_json::json!([
        {"table":"db_branch","op":"UPDATE","row_id":"b1","columns":["name"],
         "old":{"name":"A","row_version":1},"new":{"name":"B","row_version":2}},
        {"table":"db_branch_label","op":"INSERT","row_id":"l1","columns":[],
         "old":null,"new":{"lid":"th","text":"สาขา","row_version":1,"note":null}},
        {"table":"db_x","op":"DELETE","row_id":"x1","columns":[],"old":{"id":"x1"},"new":null}
    ])
    .to_string();
    assert_eq!(
        changes_text(&json),
        "db_branch UPDATE b1: name: A -> B | db_branch_label INSERT l1: lid=th, text=สาขา | db_x DELETE x1: id=x1"
    );
    let no_key = serde_json::json!([{"table":"db_x","op":"UPDATE","columns":["label"],"old":{"label":"a"},"new":{"label":"b"}}]).to_string();
    assert_eq!(changes_text(&no_key), "db_x UPDATE: label: a -> b");
    assert_eq!(changes_text("not json"), "");
    let long = serde_json::json!([{"table":"t","op":"INSERT","row_id":"1","new":{"v":"x".repeat(5000)}}]).to_string();
    assert!(changes_text(&long).chars().count() <= 1501);
}

#[tokio::test]
async fn test_a_file_that_is_gone_from_storage_is_gone_not_unavailable() {
    let r = rig(audit(1, 3, None));
    let id = Uuid::now_v7();
    r.exec.run_job(message(id, "somchai")).await;
    r.storage.objects.lock().unwrap().clear(); // storage wiped
    assert!(matches!(r.exec.preview_job("somchai", &id.to_string(), None).await, Err(Rppg000Error::FileGone(_))));
    assert!(matches!(r.exec.download_url("somchai", &id.to_string()).await, Err(Rppg000Error::FileGone(_))));
}
