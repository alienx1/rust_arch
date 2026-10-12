use application_master::features::su::supg005::{
    ActiveDeviceDto, Supg005CommandExecutor, Supg005Commands, Supg005Queries, PaginatedResult,
    UserPaginationQuery, UserSessionOverviewDto,
};
use async_trait::async_trait;
use std::sync::Arc;

struct MockSupg005Queries {
    users: Vec<UserSessionOverviewDto>,
    inactive: Vec<String>,
}

#[async_trait]
impl Supg005Queries for MockSupg005Queries {
    async fn list_users_with_sessions(
        &self,
    ) -> Result<Vec<UserSessionOverviewDto>, application_master::features::su::supg005::Supg005Error> {
        Ok(self.users.clone())
    }

    async fn is_login_allowed(
        &self,
        identifier: &str,
    ) -> Result<bool, application_master::features::su::supg005::Supg005Error> {
        Ok(!self.inactive.iter().any(|u| u == identifier))
    }

    async fn set_user_active(
        &self,
        user_id: &str,
        _active: bool,
    ) -> Result<bool, application_master::features::su::supg005::Supg005Error> {
        Ok(self.users.iter().any(|u| u.user_id == user_id))
    }

    async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, application_master::features::su::supg005::Supg005Error> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(5).clamp(1, 100);
        let offset = (page - 1) * limit;

        let total = self.users.len() as u64;
        let total_pages = if total == 0 { 1 } else { (total + limit - 1) / limit };
        let items = self
            .users
            .clone()
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        Ok(PaginatedResult {
            items,
            total,
            page,
            limit,
            total_pages,
        })
    }
}

#[tokio::test]
async fn test_supg005_executor_list_users() {
    let mock_users = vec![
        UserSessionOverviewDto {
            user_id: "019ca1ec-0000-7000-8010-000000000001".to_string(),
            username: "admin".to_string(),
            email: "admin@arch.local".to_string(),
            full_name: "Admin User".to_string(),
            employee_code: "EMP001".to_string(),
            role_code: "ADMIN".to_string(),
            role_name: "Administrator".to_string(),
            scope_level: "organization".to_string(),
            department_name: "IT".to_string(),
            company_name: "Head Office".to_string(),
            is_active_account: true,
            devices: vec![ActiveDeviceDto {
                id: "dev-01".to_string(),
                user_id: "019ca1ec-0000-7000-8010-000000000001".to_string(),
                device_name: "Chrome Desktop".to_string(),
                ip_address: "127.0.0.1".to_string(),
                location: "Bangkok".to_string(),
                last_active: "now".to_string(),
                authenticated_at: "2026-10-01 08:00".to_string(),
                is_current: true,
            }],
        },
        UserSessionOverviewDto {
            user_id: "019ca1ec-0000-7000-8010-000000000002".to_string(),
            username: "user2".to_string(),
            email: "user2@arch.local".to_string(),
            full_name: "User Two".to_string(),
            employee_code: "EMP002".to_string(),
            role_code: "OPERATOR".to_string(),
            role_name: "Operator".to_string(),
            scope_level: "branch".to_string(),
            department_name: "Operations".to_string(),
            company_name: "Branch 1".to_string(),
            is_active_account: true,
            devices: vec![],
        },
    ];

    let executor = Supg005CommandExecutor::new(Arc::new(MockSupg005Queries { users: mock_users, inactive: vec![] }));
    let list = executor.list_users().await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].username, "admin");
    assert!(list[0].is_online());
    assert_eq!(list[0].active_sessions_count(), 1);
    assert!(!list[1].is_online());
}

#[tokio::test]
async fn test_supg005_executor_revocations() {
    let executor = Supg005CommandExecutor::new(Arc::new(MockSupg005Queries { users: vec![], inactive: vec![] }));
    let res_user = executor
        .revoke_user_sessions("019ca1ec-0000-7000-8010-000000000001")
        .await
        .unwrap();
    assert!(res_user);

    let res_dev = executor.revoke_device_session("dev-01").await.unwrap();
    assert!(res_dev);
}

#[tokio::test]
async fn test_supg005_executor_list_users_paginated() {
    let mut users = Vec::new();
    for i in 1..=8 {
        users.push(UserSessionOverviewDto {
            user_id: format!("user-{i}"),
            username: format!("user{i}"),
            email: format!("user{i}@arch.local"),
            full_name: format!("User {i}"),
            employee_code: format!("EMP00{i}"),
            role_code: "USER".to_string(),
            role_name: "Staff".to_string(),
            scope_level: "company".to_string(),
            department_name: "Dept".to_string(),
            company_name: "Company".to_string(),
            is_active_account: true,
            devices: vec![],
        });
    }

    let executor = Supg005CommandExecutor::new(Arc::new(MockSupg005Queries { users, inactive: vec![] }));

    // Request page 1 with limit 5
    let p1 = executor
        .list_users_paginated(UserPaginationQuery {
            page: Some(1),
            limit: Some(5),
            search: None,
            status: None,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(p1.items.len(), 5);
    assert_eq!(p1.total, 8);
    assert_eq!(p1.page, 1);
    assert_eq!(p1.limit, 5);
    assert_eq!(p1.total_pages, 2);
    assert_eq!(p1.items[0].username, "user1");

    // Request page 2 with limit 5
    let p2 = executor
        .list_users_paginated(UserPaginationQuery {
            page: Some(2),
            limit: Some(5),
            search: None,
            status: None,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(p2.items.len(), 3);
    assert_eq!(p2.page, 2);
    assert_eq!(p2.items[0].username, "user6");
}

#[tokio::test]
async fn test_supg005_executor_suspend_and_reactivate() {
    let admin_id = "019ca1ec-0000-7000-8010-000000000001";
    let user = UserSessionOverviewDto {
        user_id: admin_id.to_string(),
        username: "admin".to_string(),
        email: "admin@arch.local".to_string(),
        full_name: "Admin".to_string(),
        employee_code: "EMP0001".to_string(),
        role_code: "ADMIN".to_string(),
        role_name: "ADMIN".to_string(),
        scope_level: "organization".to_string(),
        department_name: "IT".to_string(),
        company_name: "HQ".to_string(),
        is_active_account: true,
        devices: vec![],
    };
    let executor = Supg005CommandExecutor::new(Arc::new(MockSupg005Queries { users: vec![user], inactive: vec![] }));

    assert!(executor.suspend_user(admin_id).await.unwrap());
    assert!(executor.reactivate_user(admin_id).await.unwrap());
    assert!(executor.suspend_user("019ca1ec-0000-7000-8010-0000000000ff").await.is_err());
}

#[tokio::test]
async fn test_supg005_executor_authorize_login_refuses_inactive_account() {
    let executor = Supg005CommandExecutor::new(Arc::new(MockSupg005Queries {
        users: vec![],
        inactive: vec!["suspended@arch.local".to_string()],
    }));

    assert!(!executor.authorize_login("suspended@arch.local").await.unwrap());
    assert!(executor.authorize_login("admin@arch.local").await.unwrap());
}
