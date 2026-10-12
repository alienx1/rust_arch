//! SUPG002 pure menu tree builder (permission-filtered, folders restored above permitted children)

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::features::su::supg002::models::{MenuNodeDto, db_program};

/// Builds the visible tree: a program is shown when it is permitted or has a shown descendant
/// (folders are restored above a permitted child). Siblings sort by `sort_order`, then code.
///
/// Switching a folder off hides its whole subtree: a program whose parent is not in `programs`
/// (inactive, or in an inactive system / module) is not shown either, even if it is permitted.
pub fn build_menu_tree(
    programs: &[db_program::Model],
    permitted: &HashSet<Uuid>,
    names: &HashMap<Uuid, (String, String)>,
) -> Vec<MenuNodeDto> {
    let ids: HashSet<Uuid> = programs.iter().map(|p| p.id).collect();
    let mut children_of: HashMap<Option<Uuid>, Vec<&db_program::Model>> = HashMap::new();
    for p in programs {
        // Roots are programs without a parent; children of a hidden parent never get attached
        // and so are not reachable from a root
        if p.program_id.is_none_or(|parent| ids.contains(&parent)) {
            children_of.entry(p.program_id).or_default().push(p);
        }
    }

    fn build(
        parent: Option<Uuid>,
        children_of: &HashMap<Option<Uuid>, Vec<&db_program::Model>>,
        permitted: &HashSet<Uuid>,
        names: &HashMap<Uuid, (String, String)>,
    ) -> Vec<MenuNodeDto> {
        let mut nodes: Vec<MenuNodeDto> = children_of
            .get(&parent)
            .into_iter()
            .flatten()
            .filter_map(|p| {
                let children = build(Some(p.id), children_of, permitted, names);
                if !permitted.contains(&p.id) && children.is_empty() {
                    return None;
                }
                let (label_th, label_en) = names.get(&p.id).cloned().unwrap_or_default();
                Some(MenuNodeDto {
                    id: p.id,
                    code: p.code.clone(),
                    path: p.path.clone().filter(|path| !path.trim().is_empty()),
                    icon: p.icon.clone(),
                    label_th: if label_th.is_empty() { p.code.clone() } else { label_th },
                    label_en: if label_en.is_empty() { p.code.clone() } else { label_en },
                    sort_order: p.sort_order,
                    children,
                })
            })
            .collect();
        nodes.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then_with(|| a.code.cmp(&b.code)));
        nodes
    }

    build(None, &children_of, permitted, names)
}
