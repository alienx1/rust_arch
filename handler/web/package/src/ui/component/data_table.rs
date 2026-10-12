//! `DataTable`: the shared list-table shell used across the apps.
//!
//! Same look as the user table of `/su/supg005`: a card with a title/description header, an
//! optional search box, refresh button and action slot, a filter-chip row, loading / empty
//! states, a bordered table with a loading overlay, and a numbered pagination footer.
//!
//! The component owns the chrome; the caller owns the rows:
//!
//! ```ignore
//! <DataTable
//!     title="ตารางบริษัท"
//!     columns=vec![DataColumn::new("รหัส").class("w-28"), DataColumn::new("ชื่อ")]
//!     loading=is_loading
//!     empty=Signal::derive(move || rows.get().is_empty())
//! >
//!     {move || rows.get().into_iter().map(|r| view! { <TableRow>...</TableRow> }).collect_view()}
//! </DataTable>
//! ```

use leptos::prelude::*;
use std::sync::Arc;

use crate::core::i18n::{t_fmt, t_or};
use super::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle,
    SearchInput, Select, SelectOption, Spinner, Table, TableBody, TableHead, TableHeader, TableRow,
};

/// Optional content slot of [`DataTable`]: pass any `move || view! { ... }` closure.
#[derive(Clone, Default)]
pub struct Slot(Option<Arc<dyn Fn() -> AnyView + Send + Sync>>);

impl Slot {
    /// The slot content, `None` when the slot was not provided
    pub fn render(&self) -> Option<AnyView> {
        self.0.as_ref().map(|f| f())
    }
}

impl<F, V> From<F> for Slot
where
    F: Fn() -> V + Send + Sync + 'static,
    V: IntoAny + 'static,
{
    fn from(f: F) -> Self {
        Slot(Some(Arc::new(move || f().into_any())))
    }
}

/// One table column: header label plus optional width/alignment classes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataColumn {
    pub label: String,
    pub class: String,
}

impl DataColumn {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), class: String::new() }
    }

    /// Adds header classes, e.g. `"w-28 text-center"`
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }
}

/// Client-side paging for lists the page already holds in memory (server-paged lists keep their
/// own page signal). Pass it to [`DataTable`] as `page` / `limit` / `total` / `on_page` and render
/// `slice(...)` instead of the full list.
#[derive(Clone, Copy)]
pub struct ClientPaging {
    page: RwSignal<u64>,
    size: RwSignal<u64>,
}

impl ClientPaging {
    pub fn new(size: u64) -> Self {
        Self { page: RwSignal::new(1), size: RwSignal::new(size.max(1)) }
    }

    /// Rows per page (reactive when read inside a view closure)
    pub fn size(&self) -> u64 {
        self.size.get()
    }

    /// Rows per page as a signal, for the `limit` prop of [`DataTable`]
    pub fn size_signal(&self) -> Signal<u64> {
        let size = self.size;
        Signal::derive(move || size.get())
    }

    /// Changes the rows per page and returns to page 1
    pub fn set_size(&self, size: u64) {
        self.size.set(size.max(1));
        self.page.set(1);
    }

    /// `on_limit` callback of [`DataTable`] (the rows-per-page selector)
    pub fn on_limit(&self) -> Callback<u64> {
        let paging = *self;
        Callback::new(move |size| paging.set_size(size))
    }

    pub fn set_page(&self, page: u64) {
        self.page.set(page.max(1));
    }

    /// Current page, clamped to the pages that exist for `total` rows (filters can shrink the list)
    pub fn page_for(&self, total: u64) -> u64 {
        let pages = total.div_ceil(self.size.get()).max(1);
        self.page.get().clamp(1, pages)
    }

    /// Rows of the current page plus the 1-based number of its first row
    pub fn slice<T>(&self, items: Vec<T>) -> (Vec<T>, u64) {
        let page = self.page_for(items.len() as u64);
        let size = self.size.get();
        let start = (page - 1) * size;
        let rows = items.into_iter().skip(start as usize).take(size as usize).collect();
        (rows, start + 1)
    }
}

/// 1-based row number of the `index`-th row (0-based) on `page` of size `limit`
pub fn data_row_number(page: u64, limit: u64, index: usize) -> u64 {
    page.max(1).saturating_sub(1) * limit + index as u64 + 1
}

/// Page buttons to show: everything up to 7 pages, otherwise first, last and the current page
/// with its neighbours, `None` marking a gap.
fn page_window(current: u64, total: u64) -> Vec<Option<u64>> {
    if total <= 7 {
        return (1..=total).map(Some).collect();
    }
    let mut pages: Vec<u64> = vec![1, total];
    pages.extend(current.saturating_sub(1).max(1)..=(current + 1).min(total));
    pages.sort_unstable();
    pages.dedup();

    let mut out = Vec::new();
    let mut previous = 0;
    for p in pages {
        if p > previous + 1 {
            out.push(None);
        }
        out.push(Some(p));
        previous = p;
    }
    out
}

const CHIP_ON: &str = "px-2.5 py-1 text-xs font-medium whitespace-nowrap rounded-full bg-primary text-primary-foreground shadow-sm cursor-pointer transition-colors";
const CHIP_OFF: &str = "px-2.5 py-1 text-xs font-medium whitespace-nowrap rounded-full bg-muted text-muted-foreground hover:bg-muted/80 cursor-pointer transition-colors";

/// Row of selectable filter chips for the `filters` slot of [`DataTable`].
/// `options` are `(value, label)` pairs.
#[component]
pub fn FilterChips(
    #[prop(into)] label: String,
    options: Vec<(String, String)>,
    #[prop(into)] value: Signal<String>,
    on_change: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-2 pt-2">
            <span class="text-xs text-muted-foreground whitespace-nowrap">{label}</span>
            {options.into_iter().map(|(key, text)| {
                let selected = {
                    let key = key.clone();
                    move || value.get() == key
                };
                view! {
                    <button
                        type="button"
                        class=move || if selected() { CHIP_ON } else { CHIP_OFF }
                        on:click=move |_| on_change.run(key.clone())
                    >
                        {text}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn DataTable(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: String,
    /// SVG path data (24x24, stroked) for the icon before the title
    #[prop(optional, into)] icon_path: String,
    columns: Vec<DataColumn>,
    /// Data is being fetched: spinner when there are no rows yet, overlay when refreshing
    #[prop(optional, into)] loading: Signal<bool>,
    /// There are no rows to show
    #[prop(optional, into)] empty: Signal<bool>,
    #[prop(optional, into)] loading_text: String,
    #[prop(optional, into)] empty_title: String,
    #[prop(optional, into)] empty_text: String,
    /// Content under the empty text, e.g. an "add" button
    #[prop(optional, into)] empty_action: Slot,
    #[prop(optional)] search_value: Option<Signal<String>>,
    #[prop(optional, into)] search_placeholder: String,
    #[prop(optional)] on_search: Option<Callback<String>>,
    #[prop(optional)] on_refresh: Option<Callback<()>>,
    /// Header buttons/badges next to search and refresh
    #[prop(optional, into)] actions: Slot,
    /// Filter row under the header, typically [`FilterChips`]
    #[prop(optional, into)] filters: Slot,
    /// Banner above the table (notifications, errors)
    #[prop(optional, into)] notice: Slot,
    #[prop(optional, into)] page: Signal<u64>,
    #[prop(optional, into)] limit: Signal<u64>,
    #[prop(optional, into)] total: Signal<u64>,
    /// Shows the pagination footer; receives the 1-based page to go to
    #[prop(optional)] on_page: Option<Callback<u64>>,
    /// Shows a rows-per-page selector in the footer; receives the chosen size (the caller goes
    /// back to page 1 and reloads)
    #[prop(optional)] on_limit: Option<Callback<u64>>,
    /// Sizes offered by the selector (default 10 / 25 / 50 / 100; the current size is added)
    #[prop(optional)] limit_options: Option<Vec<u64>>,
    /// Table rows (`<TableRow>` elements)
    children: ChildrenFn,
) -> impl IntoView {
    let has_description = !description.is_empty();
    let has_icon = !icon_path.is_empty();
    let loading_text = if loading_text.is_empty() { t_or("common.state.loading", "กำลังโหลดข้อมูล...") } else { loading_text };
    let empty_title = if empty_title.is_empty() { t_or("common.state.empty", "ไม่พบข้อมูล") } else { empty_title };
    let search_placeholder =
        if search_placeholder.is_empty() { t_or("common.action.search", "ค้นหา...") } else { search_placeholder };

    let columns = StoredValue::new(columns);

    let total_pages = Signal::derive(move || {
        let l = limit.get();
        if l == 0 { 1 } else { total.get().div_ceil(l).max(1) }
    });

    view! {
        <Card class="border-border/60 shadow-sm bg-card">
            <CardHeader class="pb-4 border-b border-border/40">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <CardTitle class="text-lg font-semibold flex items-center gap-2">
                            {has_icon.then(|| view! {
                                <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path d=icon_path.clone() />
                                </svg>
                            })}
                            {title}
                        </CardTitle>
                        {has_description.then(|| view! { <CardDescription>{description.clone()}</CardDescription> })}
                    </div>

                    <div class="flex items-center gap-2">
                        {on_search.map(|on_search| view! {
                            <div class="w-56 sm:w-72">
                                <SearchInput
                                    placeholder=search_placeholder.clone()
                                    value=search_value.unwrap_or_default()
                                    on_input=on_search
                                />
                            </div>
                        })}
                        {on_refresh.map(|on_refresh| view! {
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                class="h-9 px-2.5 text-xs gap-1 shrink-0"
                                on_click=Callback::new(move |_| on_refresh.run(()))
                            >
                                <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                                    <path d="M3 3v5h5" />
                                    <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                                    <path d="M16 21h5v-5" />
                                </svg>
                                <span class="hidden sm:inline">{move || t_or("common.action.refresh", "รีเฟรช")}</span>
                            </Button>
                        })}
                        {actions.render()}
                    </div>
                </div>

                {filters.render()}
            </CardHeader>

            <CardContent class="p-4 space-y-4">
                {notice.render()}

                {move || {
                    let is_loading = loading.get();
                    let rows = children.clone();
                    if is_loading && empty.get() {
                        view! {
                            <div class="flex flex-col items-center justify-center py-12 gap-2 text-muted-foreground text-sm">
                                <Spinner class="h-6 w-6 text-primary" />
                                <span>{loading_text.clone()}</span>
                            </div>
                        }.into_any()
                    } else if empty.get() {
                        view! {
                            <div class="text-center py-12 text-muted-foreground text-sm space-y-2">
                                <svg class="w-10 h-10 mx-auto mb-2 text-muted-foreground/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <circle cx="11" cy="11" r="8" />
                                    <line x1="21" y1="21" x2="16.65" y2="16.65" />
                                </svg>
                                <h3 class="text-base font-semibold text-foreground">{empty_title.clone()}</h3>
                                {(!empty_text.is_empty()).then(|| view! {
                                    <p class="text-xs max-w-sm mx-auto">{empty_text.clone()}</p>
                                })}
                                {empty_action.render().map(|action| view! { <div class="pt-1">{action}</div> })}
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="rounded-md border border-border/70 overflow-hidden relative">
                                {is_loading.then(|| view! {
                                    <div class="absolute inset-0 bg-background/50 backdrop-blur-[1px] flex items-center justify-center z-10">
                                        <Spinner class="h-6 w-6 text-primary" />
                                    </div>
                                })}
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            {columns.get_value().into_iter().map(|c| view! {
                                                <TableHead class=c.class>{c.label}</TableHead>
                                            }).collect_view()}
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>{rows()}</TableBody>
                                </Table>
                            </div>
                        }.into_any()
                    }
                }}

                {on_page.map(|on_page| view! {
                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-border/40">
                        // Range text and rows-per-page selector sit together on the left
                        <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
                        <div class="text-xs text-muted-foreground">
                            {move || {
                                let total = total.get();
                                if total == 0 {
                                    t_or("common.table.no_records", "ไม่พบรายการข้อมูล")
                                } else {
                                    let pages = total_pages.get();
                                    let current = page.get().clamp(1, pages);
                                    let size = limit.get();
                                    let start = (current - 1) * size + 1;
                                    let end = (start + size - 1).min(total);
                                    t_fmt(
                                        "common.table.range",
                                        "แสดงรายการที่ {start} - {end} จากทั้งหมด {total} รายการ (หน้า {current} จาก {pages})",
                                        &[
                                            ("start", &start.to_string()),
                                            ("end", &end.to_string()),
                                            ("total", &total.to_string()),
                                            ("current", &current.to_string()),
                                            ("pages", &pages.to_string()),
                                        ],
                                    )
                                }
                            }}
                        </div>

                        {on_limit.map(|on_limit| {
                            let base = limit_options.clone().unwrap_or_else(|| vec![10, 25, 50, 100]);
                            view! {
                                <label class="flex items-center gap-2 text-xs text-muted-foreground">
                                    <span class="whitespace-nowrap">{move || t_or("common.table.page_size", "รายการต่อหน้า:")}</span>
                                    <div class="w-20">
                                        <Select
                                            value=Signal::derive(move || limit.get().to_string())
                                            on_change=Callback::new(move |v: String| {
                                                if let Ok(n) = v.parse::<u64>() {
                                                    on_limit.run(n);
                                                }
                                            })
                                            class="bg-background text-xs h-8 px-2"
                                        >
                                            {move || {
                                                let mut sizes = base.clone();
                                                let current = limit.get();
                                                if current > 0 && !sizes.contains(&current) {
                                                    sizes.push(current);
                                                }
                                                sizes.sort_unstable();
                                                sizes.into_iter().map(|n| view! {
                                                    <SelectOption value=n.to_string()>{n.to_string()}</SelectOption>
                                                }).collect_view()
                                            }}
                                        </Select>
                                    </div>
                                </label>
                            }
                        })}
                        </div>

                        <div class="flex items-center gap-1.5 self-end sm:self-auto">
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                class="h-8 px-2.5 text-xs gap-1"
                                disabled=Signal::derive(move || page.get() <= 1 || loading.get())
                                on_click=Callback::new(move |_| on_page.run(page.get().saturating_sub(1).max(1)))
                            >
                                <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <polyline points="15 18 9 12 15 6"></polyline>
                                </svg>
                                <span>{move || t_or("common.action.prev", "ก่อนหน้า")}</span>
                            </Button>

                            {move || {
                                let current = page.get();
                                let busy = loading.get();
                                page_window(current, total_pages.get()).into_iter().map(|p| match p {
                                    Some(p) => view! {
                                        <Button
                                            variant=if p == current { ButtonVariant::Default } else { ButtonVariant::Outline }
                                            size=ButtonSize::Sm
                                            class="h-8 min-w-8 px-2 text-xs font-medium"
                                            disabled=Signal::derive(move || busy)
                                            on_click=Callback::new(move |_| on_page.run(p))
                                        >
                                            {p.to_string()}
                                        </Button>
                                    }.into_any(),
                                    None => view! { <span class="px-1 text-xs text-muted-foreground">"…"</span> }.into_any(),
                                }).collect_view()
                            }}

                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                class="h-8 px-2.5 text-xs gap-1"
                                disabled=Signal::derive(move || page.get() >= total_pages.get() || loading.get())
                                on_click=Callback::new(move |_| on_page.run(page.get() + 1))
                            >
                                <span>{move || t_or("common.action.next", "ถัดไป")}</span>
                                <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <polyline points="9 18 15 12 9 6"></polyline>
                                </svg>
                            </Button>
                        </div>
                    </div>
                })}
            </CardContent>
        </Card>
    }
}
