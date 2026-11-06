//! Explicit preference-only import. Never run at startup and never open legacy folders here.
//! Callers supply read-only source storage and a separate Pagic destination. Recovery, private
//! keys, signatures, trust and paths are intentionally outside this operation.

use eframe::Storage;
use pagic_engine::identity::STORAGE_KEY;

#[derive(Debug, PartialEq, Eq)]
pub enum ImportResult {
    Imported,
    AlreadyPresent,
    NoPreferences,
}

/// Copy only supported, non-file-bearing preferences from the existing Recto JSON format.
/// Existing Pagic state (even malformed state) always wins. The caller chooses whether/when
/// to flush its destination; this API neither claims recovery snapshots nor copies key files.
pub fn preferences(source: &dyn Storage, destination: &mut dyn Storage) -> Result<ImportResult, String> {
    if destination.get_string(STORAGE_KEY).is_some() {
        return Ok(ImportResult::AlreadyPresent);
    }
    let Some(json) = source.get_string("recto").or_else(|| source.get_string("printcraft")) else {
        return Ok(ImportResult::NoPreferences);
    };
    let mut value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let values = value.as_object_mut().ok_or("preferences must be a JSON object")?;
    // Keys are taken from PagicApp::persist/restore, not a new serialization format. Unknown
    // future fields stay excluded until reviewed, especially anything carrying file paths.
    values.retain(|key, _| {
        matches!(
            key.as_str(),
            "theme"
                | "language"
                | "ui_scale"
                | "scroll_speed"
                | "animation_effects"
                | "default_mode"
                | "default_layout"
                | "default_zoom"
                | "default_page_gaps"
                | "always_default_view"
                | "highlight_fields"
                | "units"
                | "reading_colors"
                | "date_format"
                | "date_language"
                | "author"
                | "fill_text_size"
                | "flatten_fill_sign"
                | "comment_tool_pinned"
                | "hide_empty_highlight_hover"
                | "system_title_bar"
                | "javascript"
        )
    });
    destination.set_string(STORAGE_KEY, value.to_string());
    Ok(ImportResult::Imported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct Memory(BTreeMap<String, String>);
    impl Storage for Memory {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_owned(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }

    #[test]
    fn explicit_import_restores_real_preferences_without_keys_paths_or_recovery() {
        let original = include_str!("../../tests/fixtures/recto-preferences.json");
        let mut legacy = crate::PagicApp::new();
        legacy.restore(original);
        assert_eq!(legacy.digital_ids.len(), 1, "the fixture uses the real legacy identity-reference schema");
        let source = Memory(BTreeMap::from([("recto".into(), original.into())]));
        let mut destination = Memory::default();
        assert_eq!(preferences(&source, &mut destination), Ok(ImportResult::Imported));
        assert_eq!(source.get_string("recto").as_deref(), Some(original));
        let imported = destination.get_string(STORAGE_KEY).unwrap();
        let values: serde_json::Value = serde_json::from_str(&imported).unwrap();
        assert_eq!(values["language"], "ja");
        assert_eq!(values["theme"], "Light");
        for forbidden in [
            "digital_ids",
            "trusted",
            "signature",
            "signature_text",
            "signature_image",
            "recent",
            "last_session",
            "last_views",
            "pinned_folders",
            "recovery",
            "future_private_key",
        ] {
            assert!(values.get(forbidden).is_none(), "{forbidden}");
        }
        let mut app = crate::PagicApp::new();
        app.restore(&imported);
        let restored: serde_json::Value = serde_json::from_str(&app.persist()).unwrap();
        assert_eq!(restored["language"], "ja");
        assert_eq!(restored["theme"], "Light");
        assert!(app.digital_ids.is_empty());
        assert!(app.views.is_empty());
        assert_eq!(preferences(&source, &mut destination), Ok(ImportResult::AlreadyPresent));
        assert_eq!(destination.get_string(STORAGE_KEY), Some(imported));
    }

    #[test]
    fn existing_or_invalid_destination_is_never_overwritten() {
        let source = Memory(BTreeMap::from([("recto".into(), "{}".into())]));
        let mut destination = Memory(BTreeMap::from([(STORAGE_KEY.into(), "KEEP".into())]));
        assert_eq!(preferences(&source, &mut destination), Ok(ImportResult::AlreadyPresent));
        assert_eq!(destination.get_string(STORAGE_KEY).as_deref(), Some("KEEP"));
    }

    #[test]
    fn malformed_or_missing_source_does_not_write_destination() {
        for json in ["not JSON", "[]", "null"] {
            let source = Memory(BTreeMap::from([("recto".into(), json.into())]));
            let mut destination = Memory::default();
            assert!(preferences(&source, &mut destination).is_err());
            assert!(destination.0.is_empty());
        }
        assert_eq!(preferences(&Memory::default(), &mut Memory::default()), Ok(ImportResult::NoPreferences));
    }
}
