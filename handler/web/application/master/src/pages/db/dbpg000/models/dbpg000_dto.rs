//! DBPG000 DTOs: organization & company list models (also reused by DBPG001 forms)

use serde::{Deserialize, Serialize};

/// Serialize an id held as a UUID string as a real UUID: 16 bytes in CBOR (what the API's `Uuid`
/// fields decode), the usual string in JSON. Fails on a string that is not a UUID.
pub fn serialize_id<S>(id: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    uuid::Uuid::parse_str(id)
        .map_err(serde::ser::Error::custom)?
        .serialize(serializer)
}

/// `Option` ids: serialize `Some(uuid-string)` as a real UUID and `None` as null (see `serialize_id`).
pub fn serialize_opt_id<S>(id: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => serialize_id(id, serializer),
        None => serializer.serialize_none(),
    }
}

/// Deserializes an optional UUID (string or 16 bytes) into `Option<String>`.
pub fn deserialize_opt_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{DeserializeSeed, Visitor};
    use std::fmt;

    struct IdSeed;
    impl<'de> DeserializeSeed<'de> for IdSeed {
        type Value = String;
        fn deserialize<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<String, D2::Error> {
            deserialize_id(d)
        }
    }

    struct OptVisitor;
    impl<'de> Visitor<'de> for OptVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("null or a UUID string / 16-byte UUID")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            IdSeed.deserialize(d).map(Some)
        }
    }

    deserializer.deserialize_option(OptVisitor)
}

pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct IdVisitor;

    impl<'de> Visitor<'de> for IdVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a UUID string or 16-byte UUID byte array")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if let Ok(u) = uuid::Uuid::from_slice(v) {
                Ok(u.to_string())
            } else {
                Err(E::custom(format!("invalid uuid bytes: len {}", v.len())))
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_bytes(&v)
        }
    }

    deserializer.deserialize_any(IdVisitor)
}

/// Detailed Organization DTO
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub registration_number: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub company_count: u64,
    pub row_version: i64,
}

impl OrganizationDto {
    pub fn display_name(&self) -> String {
        format!("{} ({})", web_package::core::i18n::localized(&self.name_th, &self.name_en), self.code)
    }
}

/// Detailed Company DTO
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub organization_id: String,
    pub organization_code: String,
    pub organization_name: String,
    pub code: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub master_lookup_id: String,
    pub company_type_name: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}

/// Company Type Option for dropdown selection
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyTypeOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// High-level organization KPI summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct OrganizationSummaryDto {
    pub total_organizations: u64,
    pub active_organizations: u64,
    pub total_companies: u64,
    pub active_companies: u64,
}
