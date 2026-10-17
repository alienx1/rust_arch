//! What a request may look at: the task-local scope and the text that tells scopes apart in cache keys.
//! Run: `cargo test -p application-core --test view_scope_test`

use application_core::view_scope::{self, ViewScope};
use uuid::Uuid;

#[tokio::test]
async fn test_nothing_is_narrowed_outside_a_request() {
    assert_eq!(view_scope::current(), None);
    assert_eq!(view_scope::companies(), None);
    assert_eq!(view_scope::signature(), "-");
}

#[tokio::test]
async fn test_a_scope_is_visible_only_inside_it() {
    let (company, branch) = (Uuid::now_v7(), Uuid::now_v7());
    view_scope::scope(ViewScope::one_branch(company, branch), async {
        assert_eq!(view_scope::companies(), Some(vec![company]));
        let v = view_scope::current().unwrap();
        assert_eq!((v.single_company(), v.single_branch()), (Some(company), Some(branch)));
    })
    .await;
    assert_eq!(view_scope::current(), None);
}

#[test]
fn test_single_company_and_branch_only_when_exactly_one_remains() {
    let (a, b, d) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    let many = ViewScope { companies: Some(vec![a, b]), branches: Some(vec![a, b]), ..ViewScope::default() };
    assert_eq!((many.single_company(), many.single_branch()), (None, None));
    let partial = ViewScope {
        companies: Some(vec![a]),
        branches: Some(vec![a]),
        partial_branches: vec![b],
        departments: vec![d],
    };
    assert_eq!(partial.single_branch(), None, "a branch shown only partly is a second branch");
    assert_eq!(partial.branch_rows(), Some(vec![a, b]), "the partial branch row shows too");
    assert_eq!(ViewScope::default().branch_rows(), None);
    assert!(!ViewScope::default().is_limited() && ViewScope::one_company(a).is_limited());
}

#[tokio::test]
async fn test_scopes_get_different_cache_signatures() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let sig = |scope: ViewScope| async move { view_scope::scope(scope, async { view_scope::signature() }).await };
    let overview = sig(ViewScope::default()).await;
    let company_a = sig(ViewScope::one_company(a)).await;
    let company_b = sig(ViewScope::one_company(b)).await;
    let branch_of_a = sig(ViewScope::one_branch(a, b)).await;
    let two_branches = sig(ViewScope { companies: Some(vec![a]), branches: Some(vec![a, b]), ..ViewScope::default() }).await;
    let with_department = sig(ViewScope { companies: Some(vec![a]), branches: Some(vec![a]), partial_branches: vec![b], departments: vec![b] }).await;
    let all = [&overview, &company_a, &company_b, &branch_of_a, &two_branches, &with_department];
    for (i, x) in all.iter().enumerate() {
        for y in &all[i + 1..] {
            assert_ne!(x, y, "answers narrowed differently must not share a cache entry");
        }
    }
    assert_ne!(overview, "-", "an overview scope is not the same as no scope");
}
