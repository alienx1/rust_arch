//! Integration and unit tests for Application Master Dashboard feature.
//! Located at `application/master/tests/` (same layer as `src`).

use application_master::features::dashboard::{
    DashboardCommandExecutor, DashboardCommands, DashboardError, DashboardKpis, DashboardQueries,
    DashboardSummary, GetDashboardCommand, ModuleItem, ProgramItem,
};
use async_trait::async_trait;
use std::sync::Arc;

struct MockDashboardQueries {
    should_fail: bool,
}

#[async_trait]
impl DashboardQueries for MockDashboardQueries {
    async fn fetch_summary(&self) -> Result<DashboardSummary, DashboardError> {
        if self.should_fail {
            return Err(DashboardError::Database(sea_orm::DbErr::Custom(
                "Mock DB error".to_string(),
            )));
        }

        Ok(DashboardSummary {
            kpis: DashboardKpis {
                companies: 1,
                branches: 1,
                organizations: 1,
                departments: 1,
                employees: 2,
                provinces: 77,
                districts: 1008,
                sub_districts: 8939,
                labels: 292,
                languages: 2,
                systems: 4,
                modules: 4,
                programs: 21,
                users: 2,
            },
            modules: vec![
                ModuleItem {
                    code: "DBMT".to_string(),
                    label_th: "ทะเบียนข้อมูลหลัก".to_string(),
                    label_en: "Database Master Registration".to_string(),
                    system_code: "DB".to_string(),
                    active: true,
                },
                ModuleItem {
                    code: "SYMT".to_string(),
                    label_th: "ระบบงานหลัก".to_string(),
                    label_en: "System Core".to_string(),
                    system_code: "SY".to_string(),
                    active: true,
                },
            ],
            recent_programs: vec![
                ProgramItem {
                    code: "DBPG001".to_string(),
                    label_th: "จัดการบริษัท".to_string(),
                    label_en: "Manage".to_string(),
                    active: true,
                },
                ProgramItem {
                    code: "DBPG002".to_string(),
                    label_th: "จัดการสาขา".to_string(),
                    label_en: "Manage".to_string(),
                    active: true,
                },
            ],
            region: "ap-southeast-1 (Bangkok / Multi-Region)".to_string(),
            architecture: "Active-Active Spock pgEdge".to_string(),
        })
    }
}

#[tokio::test]
async fn test_dashboard_executor_success() {
    let mock_queries = Arc::new(MockDashboardQueries { should_fail: false });
    let executor = DashboardCommandExecutor::new(mock_queries);

    let result = executor.get_dashboard_summary().await;
    assert!(result.is_ok());

    let summary = result.unwrap();
    assert_eq!(summary.kpis.companies, 1);
    assert_eq!(summary.kpis.provinces, 77);
    assert_eq!(summary.kpis.labels, 292);
    assert_eq!(summary.modules.len(), 2);
    assert_eq!(summary.modules[0].code, "DBMT");
    assert_eq!(summary.recent_programs.len(), 2);
}

#[tokio::test]
async fn test_dashboard_executor_db_error() {
    let mock_queries = Arc::new(MockDashboardQueries { should_fail: true });
    let executor = DashboardCommandExecutor::new(mock_queries);

    let result = executor.get_dashboard_summary().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DashboardError::Database(err) => assert!(err.to_string().contains("Mock DB error")),
        other => panic!("Expected Database error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_get_dashboard_command_handler() {
    let mock_queries = Arc::new(MockDashboardQueries { should_fail: false });
    let executor: Arc<dyn DashboardCommands> =
        Arc::new(DashboardCommandExecutor::new(mock_queries));
    let handler =
        application_master::features::dashboard::GetDashboardCommandHandler::new(executor);

    let cmd = GetDashboardCommand;
    let result = handler.handle(cmd).await;
    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.kpis.employees, 2);
    assert_eq!(summary.kpis.districts, 1008);
}

#[test]
fn test_dashboard_models_serialization() {
    let summary = DashboardSummary {
        kpis: DashboardKpis {
            companies: 1,
            branches: 1,
            organizations: 1,
            departments: 1,
            employees: 2,
            provinces: 77,
            districts: 1008,
            sub_districts: 8939,
            labels: 292,
            languages: 2,
            systems: 4,
            modules: 4,
            programs: 21,
            users: 2,
        },
        modules: vec![],
        recent_programs: vec![],
        region: "ap-southeast-1".to_string(),
        architecture: "Active-Active Spock pgEdge".to_string(),
    };

    let json = serde_json::to_string(&summary).expect("Serialization failed");
    let deserialized: DashboardSummary =
        serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(deserialized.kpis.companies, 1);
    assert_eq!(deserialized.region, "ap-southeast-1");
}

// ---- Valkey read-through ----

/// Counts how often the database is asked
struct CountingQueries(std::sync::atomic::AtomicUsize);

#[async_trait]
impl DashboardQueries for CountingQueries {
    async fn fetch_summary(&self) -> Result<DashboardSummary, DashboardError> {
        let n = self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u64 + 1;
        Ok(DashboardSummary {
            kpis: DashboardKpis { employees: n, ..DashboardKpis::default() },
            modules: vec![],
            recent_programs: vec![],
            region: "test".to_string(),
            architecture: "test".to_string(),
        })
    }
}

#[tokio::test]
async fn test_cached_summary_reaches_the_database_once_until_the_namespace_is_cleared() {
    use application_core::traits::{CacheService, InMemoryCache, ReadCache, ReadCaches, cache_ns};
    let memory = Arc::new(InMemoryCache::new());
    let queries = Arc::new(CountingQueries(Default::default()));
    let exec = DashboardCommandExecutor::new(queries.clone())
        .with_cache(ReadCaches::new(Some(memory.clone() as Arc<dyn CacheService>)));

    assert_eq!(exec.get_dashboard_summary().await.unwrap().kpis.employees, 1);
    assert_eq!(exec.get_dashboard_summary().await.unwrap().kpis.employees, 1, "second read is a hit");
    assert_eq!(queries.0.load(std::sync::atomic::Ordering::SeqCst), 1);

    // An employee was added through the API: the middleware clears the dashboard namespace
    ReadCache::invalidate_namespace(memory.as_ref(), cache_ns::DASHBOARD).await;
    assert_eq!(exec.get_dashboard_summary().await.unwrap().kpis.employees, 2, "fresh numbers after the write");
}

#[tokio::test]
async fn test_without_a_cache_every_summary_read_hits_the_database() {
    let queries = Arc::new(CountingQueries(Default::default()));
    let exec = DashboardCommandExecutor::new(queries.clone());
    exec.get_dashboard_summary().await.unwrap();
    exec.get_dashboard_summary().await.unwrap();
    assert_eq!(queries.0.load(std::sync::atomic::Ordering::SeqCst), 2);
}
