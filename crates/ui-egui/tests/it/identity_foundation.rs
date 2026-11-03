//! Cross-platform identity contract and recovery ownership. Native installer acceptance is separate.
use recto_engine::{Edit, Protection, Session, identity};
use recto_ui_egui::{RecoveryMeta, RecoveryStore};
use std::sync::Arc;

#[test]
fn identity_contract_reserves_separate_installation_and_storage_namespaces() {
    let policy: serde_json::Value = serde_json::from_str(include_str!("../../../../packaging/pagic-identity.json")).unwrap();
    assert_eq!(policy["application_id"], identity::APP_ID);
    assert_eq!(policy["data_name"], identity::DATA_NAME);
    assert_eq!(policy["data_slug"], identity::DATA_SLUG);
    assert_eq!(policy["storage_key"], identity::STORAGE_KEY);
    assert_eq!(policy["windows"]["registry_root"], r"Software\pagic");
    for key in ["inno_app_id", "msi_upgrade_code"] {
        let id = policy["windows"][key].as_str().unwrap();
        assert_eq!(id.len(), 36);
        assert!(id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
        assert!(!["C6F89F3A-81B5-4D19-9B21-7A3EBE87A18F", "FF3C761B-F699-5787-9C4D-13CBDF33A529"].contains(&id));
    }
    assert_eq!(policy["windows"]["progid"], "pagic.document");
    assert!(policy["updates"]["release_api"].is_null());
}

#[test]
fn independent_recovery_store_never_claims_encrypted_recto_snapshots_or_keys() {
    let root = std::env::temp_dir().join(format!("pagic-recovery-ownership-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let legacy = RecoveryStore::new(root.join("Recto/Recovery"));
    let destination = RecoveryStore::new(root.join("Pagic/Recovery"));
    let mut session = Session::new();
    let original = Arc::new(include_bytes!("../../../engine/tests/it/fixtures/recto-marks.pdf").to_vec());
    let id = session.open("legacy.pdf", None, original, None).unwrap();
    session.apply(id, Edit::Protect(Protection { open_password: Some("fixture-password".into()), ..Default::default() })).unwrap();
    let encrypted = session.save_bytes(id).unwrap();
    let meta = RecoveryMeta {
        key: "legacy".into(),
        name: "Legacy confidential.pdf".into(),
        path: None,
        saved_at: 1_700_000_000,
        encrypted: true,
        instance: None,
    };
    legacy.write(&meta, &encrypted).unwrap();
    std::fs::create_dir_all(root.join("Recto/Digital IDs")).unwrap();
    let test_key = include_bytes!("../../../sign/tests/data/ec-p256.p12");
    std::fs::write(root.join("Recto/Digital IDs/keep.p12"), test_key).unwrap();
    assert!(destination.list().is_empty());
    assert!(!destination.dir().exists(), "inspection creates no new store and copies nothing");
    assert_eq!(legacy.list(), vec![meta]);
    assert_eq!(std::fs::read(legacy.dir().join("legacy.pdf")).unwrap(), *encrypted);
    assert_eq!(std::fs::read(root.join("Recto/Digital IDs/keep.p12")).unwrap(), test_key.as_slice());
    assert!(session.open("encrypted.pdf", None, encrypted, None).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
