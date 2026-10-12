//! Integration tests for CBOR wire format encoding and decoding.

use infrastructure::wire::{from_cbor_slice, to_cbor_vec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TestKpis {
    total_users: u32,
    active_sessions: u32,
    service_name: String,
}

#[test]
fn test_cbor_serialization_roundtrip() {
    let kpi = TestKpis {
        total_users: 1500,
        active_sessions: 320,
        service_name: "Master Data Service".to_string(),
    };

    let cbor_bytes = to_cbor_vec(&kpi).expect("Serialization to CBOR should succeed");
    assert!(!cbor_bytes.is_empty());

    let decoded: TestKpis = from_cbor_slice(&cbor_bytes).expect("Deserialization from CBOR should succeed");
    assert_eq!(decoded, kpi);
}

#[test]
fn test_cbor_size_efficiency() {
    let kpi = TestKpis {
        total_users: 100,
        active_sessions: 25,
        service_name: "Test".to_string(),
    };

    let json_bytes = serde_json::to_vec(&kpi).unwrap();
    let cbor_bytes = to_cbor_vec(&kpi).unwrap();

    // CBOR binary packing should be smaller than raw JSON bytes
    assert!(cbor_bytes.len() < json_bytes.len());
}
