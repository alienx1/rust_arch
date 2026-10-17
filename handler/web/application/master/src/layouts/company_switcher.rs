//! Company and branch switcher under the organization name in the sidebar brand: the company the whole app is looking at.
//!
//! A headquarters account picks any company of its organization or "all companies" (the
//! overview); any other account has its own company only, and sees only its own branch. The server
//! decides what each account may see; the pick is sent as `X-Company-Id` (see
//! `web_package::core::company_scope`) and the page reloads so every list is read again with it.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_package::core::company_scope::{save_selected_branch, save_selected_company};
use web_package::core::i18n::{localized, t_or};

use crate::access::use_access;

/// Option value of "all companies" / "all branches"
const ALL: &str = "";

const SELECT_CLASS: &str = "block w-full truncate rounded bg-transparent -ml-0.5 py-0 pl-0.5 pr-1 text-[10px] font-mono font-normal text-muted-foreground hover:bg-muted focus:outline-none focus:ring-1 focus:ring-ring cursor-pointer disabled:cursor-default disabled:hover:bg-transparent";

#[component]
pub fn CompanySwitcher() -> impl IntoView {
    let access = use_access();
    let options = move || {
        let mut options = Vec::new();
        // "All companies" when there is more than one to look at (or the whole organization)
        if access.unlimited.get() || access.companies.with(|c| c.len() > 1) {
            options.push((ALL.to_string(), t_or("common.company.all", "ทุกบริษัท (ภาพรวม)")));
        }
        for c in access.companies.get() {
            let name = localized(&c.name_th, &c.name_en);
            options.push((c.id.clone(), if name.is_empty() { c.code.clone() } else { name }));
        }
        options
    };
    // A switcher is for choosing only when there is a choice
    let can_pick_company = move || access.unlimited.get() || access.companies.with(|c| c.len() > 1);
    let can_pick_branch = move || {
        let company = access.selected_company_id.get();
        access.unlimited.get()
            || access.branches.with(|b| b.iter().filter(|b| company.as_ref().is_none_or(|c| *c == b.company_id)).count() > 1)
    };
    let selected = move || access.selected_company_id.get().unwrap_or_default();
    // Branches of the company being looked at (all of them in the overview); a headquarters
    // account may also pick "all branches"
    let branch_options = move || {
        let company = access.selected_company_id.get();
        let shown: Vec<_> = access
            .branches
            .get()
            .into_iter()
            .filter(|b| company.as_ref().is_none_or(|c| *c == b.company_id))
            .collect();
        let mut options = Vec::new();
        if access.unlimited.get() || shown.len() > 1 {
            options.push((ALL.to_string(), t_or("common.branch.all", "ทุกสาขา")));
        }
        for b in shown {
            {
                let name = localized(&b.name_th, &b.name_en);
                let label = if name.is_empty() { b.code.clone() } else { name };
                options.push((b.id.clone(), label));
            }
        }
        options
    };
    let selected_branch = move || access.selected_branch_id.get().unwrap_or_default();
    let on_branch_change = move |ev: leptos::ev::Event| {
        let Some(select) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) else {
            return;
        };
        let value = select.value();
        save_selected_branch((!value.is_empty()).then_some(value.as_str()));
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    };
    let on_change = move |ev: leptos::ev::Event| {
        let Some(select) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) else {
            return;
        };
        let value = select.value();
        save_selected_company((!value.is_empty()).then_some(value.as_str()));
        // A branch belongs to one company: the pick under the old company no longer applies
        save_selected_branch(None);
        // Every list reads its data again with the new company
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    };

    view! {
        <Show when=move || !access.companies.get().is_empty()>
            <label class="block min-w-0" title=move || {
                if can_pick_company() {
                    t_or("common.company.switcher", "บริษัทที่กำลังดู")
                } else {
                    t_or("common.company.own_branch_only", "แสดงข้อมูลเฉพาะสาขาของคุณ")
                }
            }>
                <select
                    class=SELECT_CLASS
                    aria-label=move || t_or("common.company.switcher", "บริษัทที่กำลังดู")
                    disabled=move || !can_pick_company()
                    on:change=on_change
                    prop:value=selected
                >
                    {move || {
                        let current = selected();
                        options()
                            .into_iter()
                            .map(|(value, label)| {
                                let is_selected = value == current;
                                view! { <option value=value selected=is_selected>{label}</option> }
                            })
                            .collect_view()
                    }}
                </select>
            </label>
            <label class="block min-w-0" title=move || {
                if can_pick_branch() {
                    t_or("common.branch.switcher", "สาขาที่กำลังดู")
                } else {
                    t_or("common.company.own_branch_only", "แสดงข้อมูลเฉพาะสาขาของคุณ")
                }
            }>
                <select
                    class=SELECT_CLASS
                    aria-label=move || t_or("common.branch.switcher", "สาขาที่กำลังดู")
                    disabled=move || !can_pick_branch()
                    on:change=on_branch_change
                    prop:value=selected_branch
                >
                    {move || {
                        let current = selected_branch();
                        branch_options()
                            .into_iter()
                            .map(|(value, label)| {
                                let is_selected = value == current;
                                view! { <option value=value selected=is_selected>{label}</option> }
                            })
                            .collect_view()
                    }}
                </select>
            </label>
        </Show>
    }
}
