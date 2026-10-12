//! Tests for the request language helpers: `Accept-Language` parsing and picking the text of
//! the language in scope. Located at `application/master/tests/request_language_test.rs`.

use application_master::features::localization::request_language::{
    Language, current, pick, pick_for, scope,
};

#[test]
fn test_accept_language_primary_tag_decides() {
    assert_eq!(Language::from_accept_language("en-US,en;q=0.9,th;q=0.8"), Language::English);
    assert_eq!(Language::from_accept_language("EN"), Language::English);
    assert_eq!(Language::from_accept_language("th"), Language::Thai);
    assert_eq!(Language::from_accept_language("fr-FR"), Language::Thai);
    assert_eq!(Language::from_accept_language(""), Language::Thai);
}

#[test]
fn test_pick_falls_back_to_the_other_language_when_empty() {
    assert_eq!(pick_for(Language::Thai, "สาขา", "Branch"), "สาขา");
    assert_eq!(pick_for(Language::English, "สาขา", "Branch"), "Branch");
    assert_eq!(pick_for(Language::English, "สาขา", ""), "สาขา");
    assert_eq!(pick_for(Language::Thai, "", "Branch"), "Branch");
}

#[tokio::test]
async fn test_scope_sets_the_language_of_the_request_only() {
    assert_eq!(current(), Language::Thai, "no scope: Thai");
    let inside = scope(Language::English, async { (current(), pick("ไทย", "English").to_string()) }).await;
    assert_eq!(inside, (Language::English, "English".to_string()));
    assert_eq!(current(), Language::Thai, "the scope ends with the request");
}
