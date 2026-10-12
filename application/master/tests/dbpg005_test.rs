//! Tests for DBPG005 (Geography): input rules shared by the executor. Paging/filter/derivation
//! behaviour is covered through the router tests (`handler/api/master/tests/dbpg005_router_test.rs`).
//! Located at `application/master/tests/dbpg005_test.rs`.

use application_master::features::db::dbpg005::{
    Dbpg005Error, GeoFilterQuery, MAX_PAGE_SIZE, UpdateGeoNodeCommand, normalize_postal_code,
};

#[test]
fn test_postal_code_accepts_digits_letters_space_and_dash() {
    assert_eq!(normalize_postal_code(" 10110 ").unwrap(), "10110");
    assert_eq!(normalize_postal_code("sw1a 1aa").unwrap(), "SW1A 1AA");
    assert_eq!(normalize_postal_code("123-45").unwrap(), "123-45");
}

#[test]
fn test_postal_code_rejects_blank_symbols_and_too_long() {
    for bad in ["", "   ", "10110!", "ไทย", &"1".repeat(21)] {
        assert!(
            matches!(normalize_postal_code(bad), Err(Dbpg005Error::Validation(_))),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn test_page_size_limit_covers_a_whole_level_under_one_parent() {
    // 77 provinces must fit in one page so selectors can load them with a single request
    assert!(MAX_PAGE_SIZE >= 77);
}

#[test]
fn test_defaults_leave_everything_unfiltered() {
    let f = GeoFilterQuery::default();
    assert!(f.province_id.is_none() && f.district_id.is_none() && f.search.is_none());
    let cmd = UpdateGeoNodeCommand {
        active: true,
        name_th: "กรุงเทพมหานคร".to_string(),
        name_en: String::new(),
        row_version: 1,
    };
    assert!(cmd.active);
}
