//! Pagic runtime namespaces. Cargo packages and display branding use these Pagic namespaces.
//! Legacy document markers and protocol names are deliberately independent of this identity.

pub const APP_ID: &str = "local.pagic.desktop";
pub const DATA_NAME: &str = "Pagic";
pub const DATA_SLUG: &str = "pagic";
pub const STORAGE_KEY: &str = "pagic";
pub const PORTABLE_DATA: &str = "PagicData";
pub const PORTABLE_MARKER: &str = "pagic.portable";
pub const CONTROL_FILE: &str = ".pagic-control.json";
/// No unauthenticated update endpoint is approved for the private Pagic repository.
pub const UPDATE_API: Option<&str> = None;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_and_installation_namespaces_are_distinct_from_recto() {
        let policy = include_str!("../../../packaging/pagic-identity.json");
        for (key, actual, old) in [
            ("application_id", APP_ID, "local.recto.desktop"),
            ("data_name", DATA_NAME, "Recto"),
            ("data_slug", DATA_SLUG, "recto"),
            ("storage_key", STORAGE_KEY, "recto"),
            ("portable_data", PORTABLE_DATA, "RectoData"),
            ("portable_marker", PORTABLE_MARKER, "recto.portable"),
            ("control_file", CONTROL_FILE, ".recto-control.json"),
        ] {
            assert!(policy.contains(&format!("\"{key}\": \"{actual}\"")));
            assert_ne!(actual, old);
        }
        for old in ["C6F89F3A-81B5-4D19-9B21-7A3EBE87A18F", "FF3C761B-F699-5787-9C4D-13CBDF33A529", "recto.document", "recto.exe"] {
            assert!(!policy.contains(old));
        }
        assert!(UPDATE_API.is_none());
    }
}
