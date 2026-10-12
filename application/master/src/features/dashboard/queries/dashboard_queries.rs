use crate::features::dashboard::errors::DashboardError;
use crate::features::dashboard::models::{
    DashboardKpis, DashboardSummary, ModuleItem, ProgramItem,
};
use crate::features::dashboard::queries::DashboardQueries;
use async_trait::async_trait;
use domain::entity::{
    db_branch, db_company, db_department, db_district, db_employee, db_label, db_language,
    db_module, db_module_label, db_organization, db_program, db_program_label, db_province,
    db_sub_district, db_system, su_user,
};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect};

/// Direct Sea-ORM query runner for dashboard analytics and real-time counts.
pub struct DbDashboardQueries {
    db: DatabaseConnection,
}

impl DbDashboardQueries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl DashboardQueries for DbDashboardQueries {
    async fn fetch_summary(&self) -> Result<DashboardSummary, DashboardError> {
        let companies = db_company::Entity::find().count(&self.db).await?;
        let branches = db_branch::Entity::find().count(&self.db).await?;
        let organizations = db_organization::Entity::find().count(&self.db).await?;
        let departments = db_department::Entity::find().count(&self.db).await?;
        let employees = db_employee::Entity::find().count(&self.db).await?;
        let provinces = db_province::Entity::find().count(&self.db).await?;
        let districts = db_district::Entity::find().count(&self.db).await?;
        let sub_districts = db_sub_district::Entity::find().count(&self.db).await?;
        let labels = db_label::Entity::find().count(&self.db).await?;
        let languages = db_language::Entity::find().count(&self.db).await?;
        let systems = db_system::Entity::find().count(&self.db).await?;
        let modules_count = db_module::Entity::find().count(&self.db).await?;
        let programs_count = db_program::Entity::find().count(&self.db).await?;
        let users_count = su_user::Entity::find().count(&self.db).await?;

        let kpis = DashboardKpis {
            companies,
            branches,
            organizations,
            departments,
            employees,
            provinces,
            districts,
            sub_districts,
            labels,
            languages,
            systems,
            modules: modules_count,
            programs: programs_count,
            users: users_count,
        };

        // Fetch modules with system code and labels
        let modules_entities = db_module::Entity::find()
            .order_by_asc(db_module::Column::Code)
            .all(&self.db)
            .await?;

        let systems_map: std::collections::HashMap<_, _> = db_system::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s.code))
            .collect();

        let module_labels = db_module_label::Entity::find().all(&self.db).await?;

        let mut modules = Vec::new();
        for m in modules_entities {
            let sys_code = systems_map.get(&m.system_id).cloned().unwrap_or_default();
            let label_th = module_labels
                .iter()
                .find(|l| {
                    l.module_id == m.id && l.language_id.to_string().ends_with("000000000001")
                })
                .map(|l| l.label.clone())
                .unwrap_or_else(|| m.code.clone());
            let label_en = module_labels
                .iter()
                .find(|l| {
                    l.module_id == m.id && l.language_id.to_string().ends_with("000000000002")
                })
                .map(|l| l.label.clone())
                .unwrap_or_else(|| m.code.clone());

            modules.push(ModuleItem {
                code: m.code,
                label_th,
                label_en,
                system_code: sys_code,
                active: m.active,
            });
        }

        // Fetch programs with Thai labels
        let program_entities = db_program::Entity::find()
            .order_by_asc(db_program::Column::Code)
            .limit(10)
            .all(&self.db)
            .await?;

        let program_labels = db_program_label::Entity::find().all(&self.db).await?;

        let mut recent_programs = Vec::new();
        for p in program_entities {
            let label_th = program_labels
                .iter()
                .find(|l| {
                    l.program_id == p.id && l.language_id.to_string().ends_with("000000000001")
                })
                .map(|l| l.label.clone())
                .unwrap_or_else(|| p.code.clone());

            let label_en = program_labels
                .iter()
                .find(|l| {
                    l.program_id == p.id && l.language_id.to_string().ends_with("000000000002")
                })
                .map(|l| l.label.clone())
                .unwrap_or_else(|| p.code.clone());

            recent_programs.push(ProgramItem {
                code: p.code,
                label_th,
                label_en,
                active: p.active,
            });
        }

        Ok(DashboardSummary {
            kpis,
            modules,
            recent_programs,
            region: "ap-southeast-1 (Bangkok / Multi-Region)".to_string(),
            architecture: "Active-Active Spock pgEdge".to_string(),
        })
    }
}
