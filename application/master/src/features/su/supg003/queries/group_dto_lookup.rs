//! SUPG003 group DTO assembly: user counts and readable-program counts per group.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::db_supg003_queries::DbSupg003Queries;
use super::group_holders::user_counts;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    MatrixAction, PermissionGroupDto, is_admin_group, su_permission, su_permission_group,
};
use domain::entity::db_master_lookup;

impl DbSupg003Queries {
    /// DTOs of `rows` whose code contains `term` (lower-case; empty = all)
    pub(super) async fn group_dtos(
        &self,
        rows: Vec<su_permission_group::Model>,
        term: &str,
    ) -> Result<Vec<PermissionGroupDto>, Supg003Error> {
        // Distinct users per group: primary group or scope assignment
        let users = user_counts(&self.db).await?;

        // READ ids by lookup code (the lookup table can repeat rows per code)
        let read_ids: HashSet<Uuid> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("permission_type"))
            .all(&self.db)
            .await?
            .into_iter()
            .filter(|l| l.code.eq_ignore_ascii_case(MatrixAction::Read.code()))
            .map(|l| l.id)
            .collect();
        let mut readable: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        for p in su_permission::Entity::find()
            .filter(su_permission::Column::Active.eq(true))
            .all(&self.db)
            .await?
        {
            if read_ids.contains(&p.master_lookup_id) {
                readable.entry(p.permission_group_id).or_default().insert(p.program_id);
            }
        }

        Ok(rows
            .into_iter()
            .filter(|g| term.is_empty() || g.code.to_lowercase().contains(term))
            .map(|g| PermissionGroupDto {
                id: g.id,
                is_protected: is_admin_group(&g.code),
                user_count: users.get(&g.id).copied().unwrap_or(0),
                readable_programs: readable.get(&g.id).map_or(0, |s| s.len() as u64),
                code: g.code,
                active: g.active,
                row_version: g.row_version,
            })
            .collect())
    }
}
