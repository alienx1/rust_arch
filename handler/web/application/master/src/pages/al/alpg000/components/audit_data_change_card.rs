//! Data change (`kind=data_change`): everything one API request changed, row by row with the
//! changed columns and the row before / after.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle};

use crate::pages::al::alpg000::components::json_block::JsonBlock;
use crate::pages::al::alpg000::models::{AuditLogDto, DataChangeEntryDto};

fn op_variant(op: &str) -> BadgeVariant {
    match op {
        "INSERT" => BadgeVariant::Success,
        "DELETE" => BadgeVariant::Destructive,
        _ => BadgeVariant::Warning,
    }
}

fn parse_entries(event: &AuditLogDto) -> Vec<DataChangeEntryDto> {
    event
        .changes
        .as_deref()
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or_default()
}

fn nonblank(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[component]
fn ChangeEntry(entry: DataChangeEntryDto) -> impl IntoView {
    let before = t_or("alpg000.detail.data_before", "ก่อนแก้ไข (Before)");
    let after = t_or("alpg000.detail.data_after", "หลังแก้ไข (After)");
    let both = entry.old.is_some() && entry.new.is_some();
    let columns = entry.columns.join(", ");
    view! {
        <div class="rounded-lg border border-border/60 p-3 space-y-3">
            <div class="flex flex-wrap items-center gap-2 text-sm">
                <Badge variant=op_variant(&entry.op) class="text-xs">{entry.op.clone()}</Badge>
                <span class="font-mono font-semibold">{entry.table.clone()}</span>
                {entry.row_id.clone().map(|r| view! { <span class="font-mono text-xs text-muted-foreground">{format!("#{r}")}</span> })}
                {(!columns.is_empty()).then(|| view! {
                    <span class="text-xs text-muted-foreground">{format!("· {columns}")}</span>
                })}
            </div>
            <div class=if both { "grid gap-3 xl:grid-cols-2" } else { "grid gap-3" }>
                {entry.old.clone().filter(|v| !v.is_null()).map(|old| view! {
                    <div class="space-y-1.5">
                        <div class="text-xs font-medium text-muted-foreground">{before}</div>
                        <JsonBlock body=Some(old) />
                    </div>
                })}
                {entry.new.clone().filter(|v| !v.is_null()).map(|new| view! {
                    <div class="space-y-1.5">
                        <div class="text-xs font-medium text-muted-foreground">{after}</div>
                        <JsonBlock body=Some(new) />
                    </div>
                })}
            </div>
        </div>
    }
}

#[component]
pub fn AuditDataChangeCard(event: AuditLogDto) -> impl IntoView {
    let entries = parse_entries(&event);
    let count = entries.len();
    let request_id = nonblank(&event.request_id);
    view! {
        <Card>
            <CardHeader class="pb-3">
                <CardTitle class="text-base">
                    {t_or("alpg000.detail.data_change", "การแก้ไขข้อมูล")}
                    <span class="ml-2 text-xs font-normal text-muted-foreground">{format!("({count})")}</span>
                </CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
                {request_id.map(|r| view! {
                    <div class="text-xs text-muted-foreground">
                        {t_or("alpg000.field.request_id", "คำขอ API (event id)")}": "
                        <span class="font-mono">{r}</span>
                    </div>
                })}
                {entries.into_iter().map(|entry| view! { <ChangeEntry entry=entry /> }).collect_view()}
            </CardContent>
        </Card>
    }
}
