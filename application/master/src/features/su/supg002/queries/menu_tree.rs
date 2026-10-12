//! SUPG002 menu tree for an account: permission-filtered read of active programs

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::db_supg002_queries::DbSupg002Queries;
use super::menu_tree_builder::build_menu_tree;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    ENGLISH_LANGUAGE_ID, MenuNodeDto, THAI_LANGUAGE_ID,
};
use domain::entity::{
    db_master_lookup, db_module, db_program, db_program_label, db_system, su_permission,
    su_permission_group, su_user, su_user_assignment,
};

impl DbSupg002Queries {
    pub(super) async fn menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error> {
        let Some(user) = su_user::Entity::find()
            .filter(su_user::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .filter(|u| u.active)
        else {
            return Ok(Vec::new());
        };

        // Every group the user holds (assignments; the primary group for a user without any). A
        // switched-off group grants nothing, and the menu shows the union over the held groups.
        let mut held: Vec<Uuid> = su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::UserId.eq(user.id))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|a| a.permission_group_id)
            .collect();
        if held.is_empty() {
            held.push(user.permission_group_id);
        }
        let active_groups: Vec<Uuid> = su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Id.is_in(held))
            .filter(su_permission_group::Column::Active.eq(true))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|g| g.id)
            .collect();
        if active_groups.is_empty() {
            return Ok(Vec::new());
        }

        // The permission_type lookup table can repeat rows per code, so match READ by code
        let read_ids: Vec<Uuid> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("permission_type"))
            .all(&self.db)
            .await?
            .into_iter()
            .filter(|l| l.code.eq_ignore_ascii_case("READ"))
            .map(|l| l.id)
            .collect();

        let permitted: HashSet<Uuid> = su_permission::Entity::find()
            .filter(su_permission::Column::PermissionGroupId.is_in(active_groups))
            .filter(su_permission::Column::Active.eq(true))
            .filter(su_permission::Column::MasterLookupId.is_in(read_ids))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|p| p.program_id)
            .collect();
        if permitted.is_empty() {
            return Ok(Vec::new());
        }

        let active_systems: HashSet<Uuid> = db_system::Entity::find()
            .filter(db_system::Column::Active.eq(true))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|s| s.id)
            .collect();
        let active_modules: HashSet<Uuid> = db_module::Entity::find()
            .filter(db_module::Column::Active.eq(true))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect();
        let programs: Vec<db_program::Model> = db_program::Entity::find()
            .filter(db_program::Column::Active.eq(true))
            .all(&self.db)
            .await?
            .into_iter()
            .filter(|p| active_systems.contains(&p.system_id) && active_modules.contains(&p.module_id))
            .collect();

        let mut names: HashMap<Uuid, (String, String)> = HashMap::new();
        for l in db_program_label::Entity::find().all(&self.db).await? {
            let entry = names.entry(l.program_id).or_default();
            if l.language_id == THAI_LANGUAGE_ID {
                entry.0 = l.label;
            } else if l.language_id == ENGLISH_LANGUAGE_ID {
                entry.1 = l.label;
            }
        }

        Ok(build_menu_tree(&programs, &permitted, &names))
    }
}
