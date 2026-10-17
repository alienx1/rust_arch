//! Contact rules: which kinds exist, who may have them, what a value must look like, defaults.
//! Run: `cargo test -p application-master --test contacts_test`

use application_master::features::db::contacts::{
    ContactError, ContactInput, ContactOwner, MAX_CONTACTS, normalize_contacts,
};

fn c(kind: &str, value: &str, default: bool) -> ContactInput {
    ContactInput { type_code: kind.to_string(), value: value.to_string(), default_contact: default }
}

fn rejected(owner: ContactOwner, input: Vec<ContactInput>) -> String {
    match normalize_contacts(owner, input) {
        Err(ContactError::Validation(m)) => m,
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[test]
fn test_a_valid_list_is_trimmed_and_the_first_of_each_kind_becomes_default() {
    let out = normalize_contacts(
        ContactOwner::Company,
        vec![
            c("phone", " 02-123-4567 ", false),
            c("PHONE", "+66 81 234 5678", false),
            c("EMAIL", "info@arch.local", false),
            c("TAX_ID", "0105550000001", false),
            c("FAX", "02-123-4568", false),
        ],
    )
    .unwrap();
    assert_eq!(out[0].value, "02-123-4567", "trimmed, the kind code is case-insensitive");
    assert_eq!(out.iter().filter(|x| x.type_code == "PHONE" && x.default_contact).count(), 1);
    assert!(out[0].default_contact && !out[1].default_contact);
    assert!(out.iter().filter(|x| x.type_code != "PHONE").all(|x| x.default_contact), "a lone contact is its kind's default");
}

#[test]
fn test_an_explicit_default_wins_and_two_are_refused() {
    let out = normalize_contacts(ContactOwner::Branch, vec![c("EMAIL", "a@x.io", false), c("EMAIL", "b@x.io", true)]).unwrap();
    assert!(!out[0].default_contact && out[1].default_contact);
    assert!(rejected(ContactOwner::Branch, vec![c("EMAIL", "a@x.io", true), c("EMAIL", "b@x.io", true)]).contains("default"));
}

#[test]
fn test_a_tax_id_is_for_legal_entities_and_an_id_card_for_people() {
    for owner in [ContactOwner::Organization, ContactOwner::Company, ContactOwner::Branch] {
        assert!(normalize_contacts(owner, vec![c("TAX_ID", "0105550000001", false)]).is_ok());
        assert!(rejected(owner, vec![c("ID_CARD", "1101700203451", false)]).contains("ID_CARD"));
    }
    assert!(normalize_contacts(ContactOwner::Employee, vec![c("ID_CARD", "1101700203451", false)]).is_ok());
    assert!(rejected(ContactOwner::Employee, vec![c("TAX_ID", "0105550000001", false)]).contains("TAX_ID"));
    // phone, fax, e-mail and Line are for everybody
    for owner in [ContactOwner::Organization, ContactOwner::Company, ContactOwner::Branch, ContactOwner::Employee] {
        assert!(normalize_contacts(owner, vec![c("PHONE", "021234567", false), c("FAX", "021234568", false), c("LINE", "@arch", false)]).is_ok());
    }
}

#[test]
fn test_values_must_follow_their_kind() {
    let o = ContactOwner::Company;
    assert!(rejected(o, vec![c("EMAIL", "not-an-email", false)]).contains("e-mail"));
    assert!(rejected(o, vec![c("EMAIL", "a@b", false)]).contains("e-mail"));
    assert!(rejected(o, vec![c("EMAIL", "a b@x.io", false)]).contains("e-mail"));
    assert!(rejected(o, vec![c("PHONE", "call me", false)]).contains("phone"));
    assert!(rejected(o, vec![c("PHONE", "123", false)]).contains("phone"));
    assert!(rejected(o, vec![c("FAX", "12345678901234567890123", false)]).contains("phone"));
    assert!(rejected(o, vec![c("TAX_ID", "12 34", false)]).contains("5-30"));
    assert!(rejected(o, vec![c("PHONE", "   ", false)]).contains("required"));
}

#[test]
fn test_unknown_kinds_duplicates_and_too_many_are_refused() {
    let o = ContactOwner::Organization;
    assert!(rejected(o, vec![c("PAGER", "123456", false)]).contains("Unknown"));
    assert!(rejected(o, vec![c("EMAIL", "A@x.io", false), c("EMAIL", "a@x.io", false)]).contains("twice"));
    let many: Vec<_> = (0..=MAX_CONTACTS).map(|i| c("LINE", &format!("id{i}"), false)).collect();
    assert!(rejected(o, many).contains("At most"));
    assert!(normalize_contacts(o, vec![]).unwrap().is_empty(), "an empty list clears the contacts");
}
