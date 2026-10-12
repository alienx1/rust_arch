//! SUPG003 group-matrix query: every program in tree order with the group's active grants.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::db_supg003_queries::DbSupg003Queries;
use super::program_tree_order::order_programs;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    MatrixAction, MatrixRowDto, PermissionMatrixDto, su_permission,
};
use domain::entity::{db_master_lookup, db_module, db_program, db_program_label, db_system};

impl DbSupg003Queries {
    pub(super) async fn group_matrix(&self, id: Uuid) -> Result<Option<PermissionMatrixDto>, Supg003Error> {
        let Some(group) = self.get_group(id).await? else { return Ok(None) };

        let system_codes: HashMap<Uuid, String> =
            db_system::Entity::find().all(&self.db).await?.into_iter().map(|s| (s.id, s.code)).collect();
        let module_codes: HashMap<Uuid, String> =
            db_module::Entity::find().all(&self.db).await?.into_iter().map(|m| (m.id, m.code)).collect();
        let names = fold_names(
            db_program_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.program_id, l.language_id, l.label)),
        );
        let programs = db_program::Entity::find().all(&self.db).await?;
        let has_children: HashSet<Uuid> = programs.iter().filter_map(|p| p.program_id).collect();

        // Action of each lookup id, so every duplicate lookup row of a code counts
        let action_of: HashMap<Uuid, MatrixAction> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("permission_type"))
            .all(&self.db)
            .await?
            .into_iter()
            .filter_map(|l| MatrixAction::from_code(&l.code).map(|a| (l.id, a)))
            .collect();
        let mut granted: HashSet<(Uuid, MatrixAction)> = HashSet::new();
        for p in su_permission::Entity::find()
            .filter(su_permission::Column::PermissionGroupId.eq(id))
            .filter(su_permission::Column::Active.eq(true))
            .all(&self.db)
            .await?
        {
            let Some(a) = action_of.get(&p.master_lookup_id) else { continue };
            granted.insert((p.program_id, *a));
        }

        let rows = order_programs(&programs, &system_codes, &module_codes)
            .into_iter()
            .map(|(p, depth)| {
                let (name_th, name_en) = names.get(&p.id).cloned().unwrap_or_default();
                MatrixRowDto {
                    program_id: p.id,
                    system_code: system_codes.get(&p.system_id).cloned().unwrap_or_default(),
                    module_code: module_codes.get(&p.module_id).cloned().unwrap_or_default(),
                    is_folder: has_children.contains(&p.id),
                    read: granted.contains(&(p.id, MatrixAction::Read)),
                    write: granted.contains(&(p.id, MatrixAction::Write)),
                    approve: granted.contains(&(p.id, MatrixAction::Approve)),
                    program_active: p.active,
                    parent_id: p.program_id,
                    program_code: p.code,
                    name_th,
                    name_en,
                    depth,
                }
            })
            .collect();
        Ok(Some(PermissionMatrixDto { group, rows }))
    }
}
