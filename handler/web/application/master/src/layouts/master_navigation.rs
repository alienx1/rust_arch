//! Master Portal sidebar menu: maps the menu tree from the database (SUPG002) to the
//! `NavSection`s rendered by `web_package::layout::MainLayout`.
//!
//! Nothing here lists pages. A program shows up in the sidebar because its `db_program` row is
//! active and the signed-in account has READ on it; switch it off with `db_program.active` or by
//! revoking the permission.

use web_package::core::i18n::t_or;
use web_package::layout::{NavItem, NavSection, UserMenuItem};

use crate::layouts::menu_service::MenuNodeDto;
use crate::routes::{DEMO, SYPG000};

/// Icon of the "main menu" group (the programs without a folder)
const MAIN_MENU_ICON: &str = "HomeOutlined";

/// Section for dev-only demo page (not in the database)
fn dev_section_label() -> String {
    t_or("common.nav.dev_demo", "ห้องทดลอง & ตัวอย่าง (Dev Demo)")
}

/// SPA route for a catalogue path.
fn route_for(path: &str) -> String {
    path.trim().to_lowercase()
}

/// The program's own canonical route, `/{system}/{code}` in lower case.
fn own_route(code: &str) -> Option<String> {
    let system = code.get(..2)?;
    code.chars().all(|c| c.is_ascii_alphanumeric()).then(|| {
        format!("/{}/{}", system.to_lowercase(), code.to_lowercase())
    })
}

/// Every leaf program under `node`, in order.
fn pages_under(node: &MenuNodeDto, lang: &str, items: &mut Vec<NavItem>) {
    if node.children.is_empty() {
        if let Some(path) = &node.path {
            let mut item = NavItem::new(route_for(path), node.label_for(lang)).with_icon(node.icon.clone());
            item.also_active.extend(own_route(&node.code).filter(|own| *own != item.href));
            items.push(item);
        }
    }
    for child in &node.children {
        pages_under(child, lang, items);
    }
}

/// Programs of the signed-in user's own (reports, notifications): they open from the user dropdown
/// in the topbar, not from the sidebar. `(code, icon path)`; the icon is a 24x24 stroke path.
const USER_MENU_PROGRAMS: [(&str, &str); 2] = [
    ("RPPG000", "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M9 15l2 2 4-4"),
    ("NTPG000", "M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9 M10.3 21a1.94 1.94 0 0 0 3.4 0"),
];

fn is_user_menu_program(code: &str) -> bool {
    USER_MENU_PROGRAMS.iter().any(|(c, _)| *c == code)
}

/// Entries of the user dropdown: the user-menu programs the account has in its menu tree
/// (a program it has no READ on is not in the tree, so it is not offered), in catalogue order.
pub fn user_menu_items(tree: &[MenuNodeDto], lang: &str) -> Vec<UserMenuItem> {
    let mut items = Vec::new();
    for node in tree {
        if let (Some((_, icon)), Some(path)) =
            (USER_MENU_PROGRAMS.iter().find(|(c, _)| *c == node.code), &node.path)
        {
            items.push(UserMenuItem::new(route_for(path), node.label_for(lang), *icon));
        }
    }
    items
}

/// Builds the sidebar from the menu tree using the specified language.
pub fn menu_to_sections(tree: &[MenuNodeDto], lang: &str, show_dev_demo: bool) -> Vec<NavSection> {
    let main_label = t_or("common.nav.main_menu", "เมนูหลัก");
    let mut main_items = Vec::new();
    let mut folders = Vec::new();

    for root in tree {
        if is_user_menu_program(&root.code) {
            continue;
        }
        if root.children.is_empty() {
            pages_under(root, lang, &mut main_items);
        } else {
            let mut items = Vec::new();
            pages_under(root, lang, &mut items);
            if !items.is_empty() {
                folders.push(NavSection::new(root.label_for(lang), items).with_icon(root.icon.clone()));
            }
        }
    }

    let mut sections = Vec::new();
    if !main_items.is_empty() {
        sections.push(NavSection::new(main_label, main_items).with_icon(Some(MAIN_MENU_ICON.to_string())));
    }
    sections.extend(folders);
    if show_dev_demo {
        sections.push(dev_demo_section());
    }
    sections
}

/// Menu shown when the tree cannot be loaded: just the dashboard, so the shell stays usable.
pub fn fallback_sections(lang: &str, show_dev_demo: bool) -> Vec<NavSection> {
    let main_label = t_or("common.nav.main_menu", "เมนูหลัก");
    let dash_label = if lang.trim().eq_ignore_ascii_case("en") {
        t_or("sypg000.title", "Dashboard")
    } else {
        t_or("sypg000.title", "ภาพรวมระบบ")
    };
    let mut sections = vec![
        NavSection::new(main_label, vec![NavItem::new(SYPG000, dash_label).with_icon(Some("DashboardOutlined".to_string()))])
            .with_icon(Some(MAIN_MENU_ICON.to_string())),
    ];
    if show_dev_demo {
        sections.push(dev_demo_section());
    }
    sections
}

fn dev_demo_section() -> NavSection {
    let mut demo = NavItem::new(DEMO, "UI Components Demo").with_icon(Some("ExperimentOutlined".to_string()));
    demo.badge = Some("Static".to_string());
    NavSection::new(dev_section_label(), vec![demo]).with_icon(Some("ExperimentOutlined".to_string()))
}
