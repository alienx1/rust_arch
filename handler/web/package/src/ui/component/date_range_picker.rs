//! Date range picker: a button showing the range, opening a panel with quick presets (today,
//! last 7 days ...) and a month calendar. Click a first day, move over the days to preview, click
//! a second day to apply. Dates are `YYYY-MM-DD` strings (empty = open end), labels follow the
//! active language (Buddhist year in Thai).

use crate::core::i18n::localized;
use crate::ui::component::dismiss_on_outside;
use chrono::{Datelike, Duration, NaiveDate};
use leptos::prelude::*;

const ISO: &str = "%Y-%m-%d";
const MONTHS_TH: [&str; 12] = ["ม.ค.", "ก.พ.", "มี.ค.", "เม.ย.", "พ.ค.", "มิ.ย.", "ก.ค.", "ส.ค.", "ก.ย.", "ต.ค.", "พ.ย.", "ธ.ค."];
const MONTHS_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const WEEKDAYS_TH: [&str; 7] = ["อา", "จ", "อ", "พ", "พฤ", "ศ", "ส"];
const WEEKDAYS_EN: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

const BAND: &str = "bg-[color-mix(in_oklch,var(--primary)_14%,transparent)]";

fn is_thai() -> bool {
    localized("th", "en") == "th"
}

fn month_name(month0: u32) -> &'static str {
    let i = (month0 % 12) as usize;
    if is_thai() { MONTHS_TH[i] } else { MONTHS_EN[i] }
}

fn display_year(year: i32) -> i32 {
    if is_thai() { year + 543 } else { year }
}

fn parse(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date.trim(), ISO).ok()
}

fn iso(date: NaiveDate) -> String {
    date.format(ISO).to_string()
}

fn today() -> NaiveDate {
    let now = js_sys::Date::new_0();
    NaiveDate::from_ymd_opt(now.get_full_year() as i32, now.get_month() + 1, now.get_date()).unwrap_or_default()
}

fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

fn shift_month(first: NaiveDate, delta: i32) -> NaiveDate {
    let total = first.year() * 12 + first.month0() as i32 + delta;
    NaiveDate::from_ymd_opt(total.div_euclid(12), total.rem_euclid(12) as u32 + 1, 1).unwrap_or(first)
}

fn day_label(date: NaiveDate) -> String {
    format!("{} {} {}", date.day(), month_name(date.month0()), display_year(date.year()))
}

/// Text of the trigger button for the chosen range
fn range_label(from: &str, to: &str) -> String {
    match (parse(from), parse(to)) {
        (Some(a), Some(b)) if a == b => day_label(a),
        (Some(a), Some(b)) => format!("{} – {}", day_label(a), day_label(b)),
        (Some(a), None) => format!("{} – …", day_label(a)),
        (None, Some(b)) => format!("… – {}", day_label(b)),
        (None, None) => localized("ทุกช่วงเวลา", "All time"),
    }
}

/// The quick presets as `(label, from, to)`; an empty pair means "no limit"
fn presets() -> Vec<(String, String, String)> {
    let t = today();
    let this_month = first_of_month(t);
    let last_month_end = this_month - Duration::days(1);
    vec![
        (localized("วันนี้", "Today"), iso(t), iso(t)),
        (localized("เมื่อวาน", "Yesterday"), iso(t - Duration::days(1)), iso(t - Duration::days(1))),
        (localized("7 วันล่าสุด", "Last 7 days"), iso(t - Duration::days(6)), iso(t)),
        (localized("30 วันล่าสุด", "Last 30 days"), iso(t - Duration::days(29)), iso(t)),
        (localized("เดือนนี้", "This month"), iso(this_month), iso(t)),
        (localized("เดือนที่แล้ว", "Last month"), iso(first_of_month(last_month_end)), iso(last_month_end)),
        (localized("ทุกช่วงเวลา", "All time"), String::new(), String::new()),
    ]
}

#[component]
pub fn DateRangePicker(
    /// First day, `YYYY-MM-DD` (empty = no lower bound)
    #[prop(into)]
    from: Signal<String>,
    /// Last day, inclusive (empty = no upper bound)
    #[prop(into)]
    to: Signal<String>,
    /// Called with `(from, to)` when a preset or a second day is chosen
    #[prop(into)]
    on_change: Callback<(String, String)>,
    #[prop(optional, into)] aria_label: String,
) -> impl IntoView {
    let (open, set_open) = signal(false);
    // First day of the month shown
    let (view_month, set_view_month) = signal(first_of_month(today()));
    // First click of a range being picked, and the day under the pointer
    let (pending, set_pending) = signal(Option::<String>::None);
    let (hover, set_hover) = signal(Option::<String>::None);

    let toggle = move |_| {
        if open.get_untracked() {
            set_open.set(false);
        } else {
            // Open on the month of the chosen range
            let anchor = parse(&from.get_untracked()).or_else(|| parse(&to.get_untracked())).unwrap_or_else(today);
            set_view_month.set(first_of_month(anchor));
            set_pending.set(None);
            set_hover.set(None);
            set_open.set(true);
        }
    };
    let close = move || {
        set_open.set(false);
        set_pending.set(None);
        set_hover.set(None);
    };
    let root = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside(root, move || open.get_untracked(), close);
    let apply = move |a: String, b: String| {
        on_change.run((a, b));
        close();
    };

    // Range painted in the calendar: the one being picked, else the chosen one
    let painted = move || -> (String, String) {
        match pending.get() {
            Some(p) => {
                let h = hover.get().unwrap_or_else(|| p.clone());
                if h < p { (h, p) } else { (p, h) }
            }
            None => {
                let (a, b) = (from.get(), to.get());
                match (a.is_empty(), b.is_empty()) {
                    (false, true) => (a.clone(), a),
                    (true, false) => (b.clone(), b),
                    _ => (a, b),
                }
            }
        }
    };

    let pick_day = move |day: String| match pending.get_untracked() {
        None => {
            set_pending.set(Some(day.clone()));
            set_hover.set(Some(day));
        }
        Some(first) => {
            let (a, b) = if day < first { (day, first) } else { (first, day) };
            apply(a, b);
        }
    };

    let weekday_row = move || {
        let names = if is_thai() { WEEKDAYS_TH } else { WEEKDAYS_EN };
        names
            .iter()
            .map(|n| view! { <div class="h-8 w-9 flex items-center justify-center text-[11px] font-medium text-muted-foreground">{*n}</div> })
            .collect_view()
    };

    let grid = move || {
        let first = view_month.get();
        let (lo, hi) = painted();
        let today_iso = iso(today());
        let lead = first.weekday().num_days_from_sunday() as usize;
        let days = (shift_month(first, 1) - first).num_days() as u32;
        let mut cells = Vec::with_capacity(lead + days as usize);
        for _ in 0..lead {
            cells.push(view! { <div class="h-8 w-9"></div> }.into_any());
        }
        for d in 1..=days {
            let date = first.with_day(d).unwrap_or(first);
            let key = iso(date);
            let is_start = !lo.is_empty() && key == lo;
            let is_end = !hi.is_empty() && key == hi;
            let in_range = !lo.is_empty() && !hi.is_empty() && key >= lo && key <= hi;
            let is_today = key == today_iso;
            let col = (lead as u32 + d - 1) % 7;
            // The band runs across the week and is rounded at its ends
            let band = if in_range && !(is_start && is_end) {
                format!(
                    "{BAND} {} {}",
                    if is_start || col == 0 { "rounded-l-md" } else { "" },
                    if is_end || col == 6 { "rounded-r-md" } else { "" }
                )
            } else {
                String::new()
            };
            let face = if is_start || is_end {
                "bg-primary text-primary-foreground font-semibold"
            } else if is_today {
                "ring-1 ring-primary text-primary font-semibold hover:bg-accent"
            } else {
                "hover:bg-accent"
            };
            let click_key = key.clone();
            let hover_key = key.clone();
            cells.push(
                view! {
                    <div class=format!("h-8 w-9 flex items-center justify-center {band}")>
                        <button
                            type="button"
                            class=format!("h-8 w-8 rounded-md text-xs transition-colors {face}")
                            on:click=move |_| pick_day(click_key.clone())
                            on:mouseenter=move |_| {
                                if pending.get_untracked().is_some() {
                                    set_hover.set(Some(hover_key.clone()));
                                }
                            }
                        >
                            {d}
                        </button>
                    </div>
                }
                .into_any(),
            );
        }
        cells.collect_view()
    };

    view! {
        <div class="relative inline-block text-left" node_ref=root>
            <button
                type="button"
                class="flex h-8 items-center gap-2 rounded-md border border-input bg-background px-2.5 text-xs shadow-sm hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                aria-label=aria_label
                aria-haspopup="dialog"
                aria-expanded=move || open.get().to_string()
                on:click=toggle
            >
                <svg class="h-3.5 w-3.5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <rect x="3" y="4" width="18" height="18" rx="2" />
                    <path d="M16 2v4M8 2v4M3 10h18" />
                </svg>
                <span class="whitespace-nowrap font-medium">{move || range_label(&from.get(), &to.get())}</span>
            </button>
            <Show when=move || open.get()>
                <div
                    class="absolute left-0 z-50 mt-2 flex w-max max-w-[calc(100vw-2rem)] max-sm:fixed max-sm:inset-x-4 max-sm:w-auto max-sm:max-w-none flex-col overflow-hidden rounded-lg border border-border bg-popover text-popover-foreground shadow-lg sm:flex-row"
                    role="dialog"
                >
                    <div class="flex flex-row flex-wrap gap-1 border-b border-border p-2 sm:w-36 sm:flex-col sm:flex-nowrap sm:border-b-0 sm:border-r">
                        {presets()
                            .into_iter()
                            .map(|(label, a, b)| {
                                let (a2, b2) = (a.clone(), b.clone());
                                let active = move || from.get() == a && to.get() == b;
                                view! {
                                    <button
                                        type="button"
                                        class=move || format!(
                                            "rounded-md px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-accent {}",
                                            if active() { "bg-accent font-semibold text-primary" } else { "" }
                                        )
                                        on:click=move |_| apply(a2.clone(), b2.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                    <div class="mx-auto w-[17.25rem] shrink-0 p-3">
                        <div class="mb-2 flex items-center justify-between">
                            <button
                                type="button"
                                class="flex h-7 w-7 items-center justify-center rounded-md hover:bg-accent"
                                aria-label="Previous month"
                                on:click=move |_| set_view_month.update(|m| *m = shift_month(*m, -1))
                            >
                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path d="M15 18l-6-6 6-6" /></svg>
                            </button>
                            <div class="text-sm font-semibold">
                                {move || {
                                    let m = view_month.get();
                                    format!("{} {}", month_name(m.month0()), display_year(m.year()))
                                }}
                            </div>
                            <button
                                type="button"
                                class="flex h-7 w-7 items-center justify-center rounded-md hover:bg-accent"
                                aria-label="Next month"
                                on:click=move |_| set_view_month.update(|m| *m = shift_month(*m, 1))
                            >
                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path d="M9 18l6-6-6-6" /></svg>
                            </button>
                        </div>
                        <div class="grid grid-cols-7">{weekday_row}</div>
                        <div class="grid grid-cols-7" on:mouseleave=move |_| {
                            if let Some(p) = pending.get_untracked() {
                                set_hover.set(Some(p));
                            }
                        }>
                            {grid}
                        </div>
                        <div class="mt-2 text-center text-[11px] text-muted-foreground">
                            {move || if pending.get().is_some() {
                                localized("เลือกวันสุดท้ายของช่วง", "Pick the last day")
                            } else {
                                localized("เลือกวันแรกของช่วง", "Pick the first day")
                            }}
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
