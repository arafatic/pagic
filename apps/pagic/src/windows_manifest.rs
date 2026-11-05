#[allow(dead_code)]
pub(crate) const WINDOWS_MANIFEST: &str = include_str!("../pagic.exe.manifest");

#[cfg(test)]
mod tests {
    use super::WINDOWS_MANIFEST;

    #[test]
    fn declares_assembly_identity() {
        assert!(WINDOWS_MANIFEST.contains("<assemblyIdentity"));
        assert!(WINDOWS_MANIFEST.contains("name=\"Arafatic.Pagic\""));
    }

    #[test]
    fn declares_per_monitor_v2_with_per_monitor_fallback() {
        assert!(
            WINDOWS_MANIFEST
                .contains("<dpiAwareness xmlns=\"http://schemas.microsoft.com/SMI/2016/WindowsSettings\">PerMonitorV2, PerMonitor</dpiAwareness>")
        );
    }

    #[test]
    fn legacy_declaration_is_per_monitor_aware() {
        assert!(WINDOWS_MANIFEST.contains("<dpiAware xmlns=\"http://schemas.microsoft.com/SMI/2005/WindowsSettings\">true/PM</dpiAware>"));
    }

    #[test]
    fn windows_resource_build_script_embeds_this_manifest() {
        let build_script = include_str!("../build.rs");
        assert!(build_script.contains("set_manifest_file(\"pagic.exe.manifest\")"));
    }
}
