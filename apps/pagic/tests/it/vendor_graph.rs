//! Dependency-graph verification for the egui 0.37 window stack.
//!
//! The `winit_*` modules retain source guards for the older `vendor/winit` copy.
//! Runtime winit resolves to the exact pinned tree in `vendor/pagic-winit`; its own
//! tests cover the fork's behavior. The egui graph and retained evidence are unchanged.
//! These tests verify the runtime source as well as the historical provenance.

/// Return the workspace root above the Pagic app crate.
fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `[[package]]` block in `Cargo.lock` whose `name` is exactly `name`.
fn locked_packages(name: &str) -> Vec<String> {
    let lock = std::fs::read_to_string(root().join("Cargo.lock")).expect("Cargo.lock");
    let needle = format!("name = \"{name}\"");
    lock.split("[[package]]").filter(|package| package.lines().any(|line| line.trim() == needle.as_str())).map(str::to_owned).collect()
}

/// The one resolved package called `name`. A duplicate name is a lockfile problem, not something
/// to resolve by taking the first match.
fn locked(name: &str) -> String {
    let mut packages = locked_packages(name);
    assert_eq!(packages.len(), 1, "Cargo.lock must contain exactly one `{name}` package, found {}", packages.len());
    packages.remove(0)
}

/// The resolved package must be the exact canonical arafatic git source at the reviewed
/// 40-character revision, including the `Cargo.lock` URL and fragment. A path source (no `source`
/// line) is refused: it would make the graph depend on a developer's checkout. A published path
/// patch is refused as well.
fn assert_resolved(name: &str, version: &str, url: &str, rev: &str) {
    let package = locked(name);
    let expected_version = format!("version = \"{version}\"");
    assert!(package.lines().any(|line| line.trim() == expected_version.as_str()), "{name} must resolve to {version}: {package}");
    let sources: Vec<&str> = package.lines().filter_map(|line| line.trim().strip_prefix("source = ")).collect();
    assert_eq!(sources.len(), 1, "{name} must have exactly one `source` in Cargo.lock; a local path source is not accepted: {package}");
    let expected_source = format!("\"git+{url}?rev={rev}#{rev}\"");
    assert_eq!(sources[0], expected_source.as_str(), "{name} must resolve to the exact canonical git URL, revision and fragment, not {}", sources[0]);

    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).expect("root Cargo.toml");
    assert!(!manifest.contains(&format!("{name} = {{ path = \"vendor/")), "vendor/{name} cannot stay a published path patch");
}

/// winit must resolve only to the complete local copy of the reviewed revision.
fn assert_local_winit() {
    let package = locked("winit");
    assert!(package.lines().any(|line| line.trim() == "version = \"0.30.13\""));
    assert!(!package.lines().any(|line| line.trim().starts_with("source = ")), "winit must be local: {package}");
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).expect("root manifest");
    assert!(manifest.contains("winit = { path = \"vendor/pagic-winit\" }"));
    let provenance = std::fs::read_to_string(root().join("vendor/pagic-winit/NOTICE-PAGIC-PROVENANCE.md")).expect("winit provenance");
    assert!(provenance.contains("a517b3846ee8457d84ee4975110bfda6797c44dc"));
}

/// Regression test: the resolved window stack points at the reviewed sources and the retained vendor provenance
/// files remain.
#[test]
fn vendor_provenance_window_stack_graph_is_resolved() {
    assert_local_winit();
    assert_resolved("egui-wgpu", "0.37.0", "https://github.com/arafatic/egui.git", "6d518fc799681571963842f95b55a6b68431b15a");
    // Retained provenance: the audited copies and their recorded W1–W13 / E2+E3 rows.
    let readme = std::fs::read_to_string(root().join("vendor/README.md")).expect("vendor/README.md");
    for marker in ["W1–W13", "surface_fit", "catch_errors", "e9809ef54b18499bb4f2cac945719ecc2a61061b"] {
        assert!(readme.contains(marker), "vendor/README.md lost window-stack provenance: {marker}");
    }
    let vcs = std::fs::read_to_string(root().join("vendor/winit/.cargo_vcs_info.json")).expect("vendor/winit provenance");
    assert!(vcs.contains("e9809ef54b18499bb4f2cac945719ecc2a61061b"), "vendor/winit keeps its upstream base");
    for path in ["vendor/winit/src/platform_impl", "vendor/egui-wgpu/src/winit.rs"] {
        assert!(root().join(path).exists(), "retain {path} as provenance for the pinned fork");
    }
}

/// Renderer, security and PostScript vendors are unrelated to the UI port and must stay path patches.
#[test]
fn renderer_and_security_vendors_remain() {
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).expect("root Cargo.toml");
    for vendor in [
        "vendor/hayro",
        "vendor/hayro-interpret",
        "vendor/hayro-syntax",
        "vendor/hayro-cmap",
        "vendor/hayro-jbig2",
        "vendor/hayro-postscript",
        "vendor/lopdf",
        "vendor/vello_common",
    ] {
        assert!(manifest.contains(&format!("path = \"{vendor}\"")), "{vendor} must stay a path patch");
    }
}

/// A fresh release checkout must resolve the shared libraries without their former Git host.
#[test]
fn craft_libraries_resolve_to_local_pinned_source() {
    let compiler = std::process::Command::new("rustc").arg("-vV").output().expect("rustc host");
    assert!(compiler.status.success(), "rustc host query failed");
    let version = String::from_utf8(compiler.stdout).expect("rustc version UTF-8");
    let host = version.lines().find_map(|line| line.strip_prefix("host: ")).expect("native host triple");
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", host])
        .current_dir(root())
        .output()
        .expect("Cargo metadata");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Cargo metadata JSON");
    let packages = metadata["packages"].as_array().expect("resolved packages");
    for name in ["docpath", "print", "ui", "menu", "keymap", "quit", "update", "fonts-build", "i18n"] {
        let matches: Vec<_> = packages.iter().filter(|p| p["name"] == format!("craft-{name}")).collect();
        assert_eq!(matches.len(), 1, "exactly one craft-{name}");
        let package = matches[0];
        assert_eq!(package["version"], "0.1.0", "craft-{name} version");
        assert!(package["source"].is_null(), "craft-{name} must be a local path dependency");
        let actual = std::path::Path::new(package["manifest_path"].as_str().expect("manifest path")).canonicalize().unwrap();
        let expected = root().join(format!("vendor/pagic-libs/crates/{name}/Cargo.toml")).canonicalize().unwrap();
        assert_eq!(actual, expected, "craft-{name} must use the vendored manifest");
    }
}

/// Every native desktop consumer must resolve the one exact local winit source.
#[test]
fn winit_resolves_to_local_pinned_source() {
    assert_local_winit();
    let compiler = std::process::Command::new("rustc").arg("-vV").output().expect("rustc host");
    assert!(compiler.status.success(), "rustc host query failed");
    let version = String::from_utf8(compiler.stdout).expect("rustc version UTF-8");
    let host = version.lines().find_map(|line| line.strip_prefix("host: ")).expect("native host triple");
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", host])
        .current_dir(root())
        .output()
        .expect("Cargo metadata");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Cargo metadata JSON");
    let packages = metadata["packages"].as_array().expect("resolved packages");
    let matches: Vec<_> = packages.iter().filter(|p| p["name"] == "winit").collect();
    assert_eq!(matches.len(), 1, "exactly one winit package");
    let package = matches[0];
    assert_eq!(package["version"], "0.30.13");
    assert!(package["source"].is_null(), "winit must use the vendored source");
    let actual = std::path::Path::new(package["manifest_path"].as_str().expect("winit manifest")).canonicalize().unwrap();
    let expected = root().join("vendor/pagic-winit/Cargo.toml").canonicalize().unwrap();
    assert_eq!(actual, expected, "all consumers must share the pinned local winit");
}
