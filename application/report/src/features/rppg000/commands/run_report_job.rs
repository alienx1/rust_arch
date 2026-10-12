//! RPPG000 worker action: build one report

use application_core::traits::{
    Notification, ReportJobMessage, notification_kind, report_kind,
};
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

use super::audit_csv_rows::{BOM, event_line, header_line};
use super::csv_object_writer::CsvObjectWriter;
use super::rppg000_executor::Rppg000CommandExecutor;
use crate::features::rppg000::models::{MAX_EXPORT_ROWS, job_status};

/// What a finished report is
struct BuiltReport {
    rows: u64,
    bytes: u64,
    file_name: String,
    object_key: String,
}

impl Rppg000CommandExecutor {
    pub(super) async fn run_job_action(&self, message: ReportJobMessage) {
        let Ok(id) = Uuid::parse_str(&message.job_id) else {
            tracing::error!(job_id = %message.job_id, "report request with a bad job id skipped");
            return;
        };
        // A redelivered request whose job already finished is not built twice. A reprocess request
        // (ALPG001) is the opposite: it runs only while its row is waiting (`queued`) or running,
        // so a redelivery after the re-run finished is skipped the same way.
        match self.jobs.find_job(id).await {
            Ok(Some(job)) if message.reprocess => {
                if job.status != job_status::QUEUED && job.status != job_status::RUNNING {
                    tracing::info!(%id, status = %job.status, "reprocess request already handled");
                    return;
                }
            }
            Ok(None) if message.reprocess => {
                tracing::warn!(%id, "reprocess request for a job without a row skipped");
                return;
            }
            Ok(Some(job)) if job.status == job_status::SUCCEEDED || job.status == job_status::FAILED => {
                tracing::info!(%id, status = %job.status, "report request already handled");
                return;
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!(%id, error = %e, "cannot read the job row; request skipped");
                return;
            }
        }
        let requested_at = DateTime::parse_from_rfc3339(&message.requested_at)
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let filter_json = serde_json::to_value(&message.filter).unwrap_or_default();
        if let Err(e) = self
            .jobs
            .start_job(id, &message.report_kind, &message.requested_by, filter_json, requested_at)
            .await
        {
            tracing::error!(%id, error = %e, "cannot record the job start; request skipped");
            return;
        }
        tracing::info!(%id, kind = %message.report_kind, user = %message.requested_by, "report started");

        let outcome = match message.report_kind.as_str() {
            report_kind::AUDIT_LOG_CSV => self.build_audit_log_csv(id, &message).await,
            other => Err(format!("Unknown report kind `{other}`")),
        };
        match outcome {
            Ok(built) => {
                if let Err(e) = self
                    .jobs
                    .finish_job_ok(id, built.rows as i64, built.bytes as i64, &built.file_name, &built.object_key)
                    .await
                {
                    tracing::error!(%id, error = %e, "cannot record the job result");
                }
                tracing::info!(%id, rows = built.rows, bytes = built.bytes, "report ready");
                self.notify(
                    &message.requested_by,
                    notification_kind::REPORT_READY,
                    "รายงานพร้อมให้ดาวน์โหลด",
                    format!("{} ({} แถว)", built.file_name, built.rows),
                    id,
                )
                .await;
            }
            Err(reason) => {
                tracing::error!(%id, %reason, "report failed");
                if let Err(e) = self.jobs.finish_job_failed(id, &reason).await {
                    tracing::error!(%id, error = %e, "cannot record the job failure");
                }
                self.notify(
                    &message.requested_by,
                    notification_kind::REPORT_FAILED,
                    "สร้างรายงานไม่สำเร็จ",
                    reason,
                    id,
                )
                .await;
            }
        }
    }

    async fn notify(&self, username: &str, kind: &str, title: &str, body: String, job_id: Uuid) {
        self.notifier
            .notify(
                username,
                Notification {
                    id: Uuid::now_v7().to_string(),
                    kind: kind.to_string(),
                    title: title.to_string(),
                    body,
                    data: serde_json::json!({ "job_id": job_id.to_string() }),
                    link: Some("/rp/rppg000".to_string()),
                    created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                },
            )
            .await;
    }

    /// Streams every event the filter matches into `reports/<job id>.csv`
    async fn build_audit_log_csv(&self, id: Uuid, message: &ReportJobMessage) -> Result<BuiltReport, String> {
        let object_key = format!("reports/{id}.csv");
        let thai = FixedOffset::east_opt(7 * 3600).ok_or("bad offset")?;
        let file_name = format!("audit-log-{}.csv", Utc::now().with_timezone(&thai).format("%Y%m%d-%H%M%S"));
        let mut writer = CsvObjectWriter::new(self.content.clone(), &self.bucket, &object_key);

        let result = async {
            writer.write(BOM).await.map_err(|e| e.to_string())?;
            writer.write(&header_line()).await.map_err(|e| e.to_string())?;
            let (mut rows, mut cursor) = (0u64, None);
            loop {
                let chunk = self
                    .audit
                    .export_chunk(message.filter.clone(), cursor)
                    .await
                    .map_err(|e| e.to_string())?;
                let mut text = String::new();
                for event in &chunk.items {
                    text.push_str(&event_line(event));
                }
                rows += chunk.items.len() as u64;
                writer.write(&text).await.map_err(|e| e.to_string())?;
                cursor = chunk.cursor;
                if cursor.is_none() || rows >= MAX_EXPORT_ROWS {
                    break;
                }
            }
            Ok::<u64, String>(rows)
        }
        .await;

        match result {
            Ok(rows) => {
                let bytes = writer.written;
                writer.finish().await.map_err(|e| e.to_string())?;
                Ok(BuiltReport { rows, bytes, file_name, object_key })
            }
            Err(reason) => {
                writer.abort().await;
                Err(reason)
            }
        }
    }
}
