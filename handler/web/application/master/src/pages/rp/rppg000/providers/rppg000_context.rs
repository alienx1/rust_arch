//! RPPG000 context: my report jobs, and the preview of one file. The list reloads when the
//! server pushes a notification.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::pages::rp::rppg000::models::{ReportJobDto, ReportPreviewDto};
use crate::pages::nt::ntpg000::providers::use_notifications;
use crate::pages::rp::rppg000::services::Rppg000Service;

/// Rows the preview asks for
const PREVIEW_ROWS: usize = 100;

/// The preview dialog
#[derive(Clone, Debug, PartialEq)]
pub enum PreviewState {
    Closed,
    Loading { job: ReportJobDto },
    Ready { job: ReportJobDto, preview: ReportPreviewDto },
    Failed { job: ReportJobDto, message: String },
}

#[derive(Clone, Copy)]
pub struct Rppg000Context {
    pub jobs: ReadSignal<Vec<ReportJobDto>>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub preview: ReadSignal<PreviewState>,
    pub refresh: Callback<()>,
    pub open_preview: Callback<ReportJobDto>,
    pub close_preview: Callback<()>,
    /// Starts the browser download of a finished report
    pub download: Callback<ReportJobDto>,
}

pub fn provide_rppg000_context() -> Rppg000Context {
    let service = Rppg000Service::new();
    let notifications = use_notifications();

    let (jobs, set_jobs) = signal(Vec::<ReportJobDto>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (preview, set_preview) = signal(PreviewState::Closed);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new(move |_| {
        refresh_trigger.track();
        // A report finished (or failed) elsewhere: show it
        notifications.changes.track();
        set_is_loading.set(true);
        let s = service.clone();
        spawn_local(async move {
            match s.fetch_jobs().await {
                Ok(list) => {
                    set_jobs.set(list);
                    set_error.set(None);
                }
                Err(e) => set_error.set(Some(e.message())),
            }
            set_is_loading.set(false);
        });
    });

    let open_preview = {
        let service = Rppg000Service::new();
        Callback::new(move |job: ReportJobDto| {
            set_preview.set(PreviewState::Loading { job: job.clone() });
            let s = service.clone();
            spawn_local(async move {
                match s.fetch_preview(&job.id, PREVIEW_ROWS).await {
                    Ok(preview) => set_preview.set(PreviewState::Ready { job, preview }),
                    Err(e) => set_preview.set(PreviewState::Failed { job, message: e.message() }),
                }
            });
        })
    };

    let download_service = Rppg000Service::new();
    let ctx = Rppg000Context {
        jobs,
        is_loading,
        error,
        preview,
        refresh: Callback::new(move |_| set_refresh_trigger.update(|n| *n += 1)),
        open_preview,
        close_preview: Callback::new(move |_| set_preview.set(PreviewState::Closed)),
        download: Callback::new(move |job: ReportJobDto| {
            let s = download_service.clone();
            spawn_local(async move {
                match s.fetch_download_url(&job.id).await {
                    // The storage answers with an attachment, so the page stays where it is
                    Ok(url) => {
                        if let Some(win) = web_sys::window() {
                            let _ = win.location().set_href(&url);
                        }
                    }
                    Err(e) => set_error.set(Some(e.message())),
                }
            });
        }),
    };
    provide_context(ctx);
    ctx
}

pub fn use_rppg000() -> Rppg000Context {
    expect_context::<Rppg000Context>()
}
