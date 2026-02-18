use super::MachineEnvelope;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Payload {
    value: u32,
}

#[test]
fn envelope_ok_reflects_errors_empty() {
    let env = MachineEnvelope {
        version: "v1".to_string(),
        command: "lint".to_string(),
        ok: true,
        data: Payload { value: 1 },
        warnings: vec![],
        errors: vec![],
        meta: super::MachineMeta {
            generated_at: "2026-02-18T00:00:00Z".to_string(),
        },
    };

    let json = serde_json::to_value(&env).expect("serialize");
    assert_eq!(json["version"], "v1");
    assert_eq!(json["command"], "lint");
    assert_eq!(json["ok"], true);
}

#[test]
fn envelope_serializes_core_fields_for_contract() {
    let env = MachineEnvelope {
        version: "v1".to_string(),
        command: "search".to_string(),
        ok: false,
        data: Payload { value: 2 },
        warnings: vec!["warn".to_string()],
        errors: vec!["err".to_string()],
        meta: super::MachineMeta {
            generated_at: "2026-02-18T00:00:00Z".to_string(),
        },
    };

    let json = serde_json::to_value(&env).expect("serialize");
    assert!(json.get("data").is_some());
    assert!(json.get("warnings").is_some());
    assert!(json.get("errors").is_some());
    assert!(json.get("meta").is_some());
}
