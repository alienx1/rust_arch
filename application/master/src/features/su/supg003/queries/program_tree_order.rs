//! SUPG003 program tree order: programs as a depth-annotated tree for the matrix rows.

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use domain::entity::db_program;

/// Programs in tree order (folders before their children, siblings by `sort_order` then code)
/// within each system and module, with their depth. Orphans (parent not in the list) are roots.
pub fn order_programs(programs: &[db_program::Model], systems: &HashMap<Uuid, String>, modules: &HashMap<Uuid, String>) -> Vec<(db_program::Model, u32)> {
    let ids: HashSet<Uuid> = programs.iter().map(|p| p.id).collect();
    let mut children: HashMap<Option<Uuid>, Vec<&db_program::Model>> = HashMap::new();
    for p in programs {
        let parent = p.program_id.filter(|parent| ids.contains(parent));
        children.entry(parent).or_default().push(p);
    }
    for list in children.values_mut() {
        list.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then_with(|| a.code.cmp(&b.code)));
    }

    fn walk(
        parent: Option<Uuid>,
        depth: u32,
        children: &HashMap<Option<Uuid>, Vec<&db_program::Model>>,
        out: &mut Vec<(db_program::Model, u32)>,
    ) {
        for p in children.get(&parent).into_iter().flatten() {
            out.push(((*p).clone(), depth));
            walk(Some(p.id), depth + 1, children, out);
        }
    }

    // Roots grouped by system code, then module code, then their own order
    let mut roots: Vec<&db_program::Model> = children.get(&None).cloned().unwrap_or_default();
    roots.sort_by(|a, b| {
        let key = |p: &db_program::Model| {
            (
                systems.get(&p.system_id).cloned().unwrap_or_default(),
                modules.get(&p.module_id).cloned().unwrap_or_default(),
                p.sort_order,
                p.code.clone(),
            )
        };
        key(a).cmp(&key(b))
    });
    let mut out = Vec::new();
    for r in roots {
        out.push((r.clone(), 0));
        walk(Some(r.id), 1, &children, &mut out);
    }
    out
}
