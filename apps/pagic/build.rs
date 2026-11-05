//! Embed desktop executable metadata. On Windows, icons and VERSIONINFO in `pagic.exe` mean it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab. A separate PDF document icon
//! uses resource ID 2; the installer references it as `pagic.exe,-2`.
//!
//! On other targets this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PAGIC_REQUIRE_WINRES=1` (legacy `RECTO_REQUIRE_WINRES`) turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/pagic.ico");
    println!("cargo:rerun-if-changed=../../assets/app-icon/pagic-document.ico");
    println!("cargo:rerun-if-changed=pagic.exe.manifest");
    println!("cargo:rerun-if-changed=src/windows_manifest.rs");
    println!("cargo:rerun-if-env-changed=RECTO_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=PAGIC_REQUIRE_WINRES");
    // Artwork is supplied separately. Never relabel or embed the archived Recto logo.
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let mac = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "macos");
    let icon =
        root.join(if mac { "../../assets/app-icon/pagic-1024.png" } else { "../../assets/app-icon/hicolor/256x256/apps/local.pagic.desktop.png" });
    println!("cargo::rerun-if-changed={}", icon.display());
    let source = if icon.is_file() {
        format!("pub const APP_ICON_PNG: Option<&[u8]> = Some(include_bytes!({:?}));", icon)
    } else {
        "pub const APP_ICON_PNG: Option<&[u8]> = None;".to_owned()
    };
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("app_icon.rs");
    if let Err(e) = std::fs::write(out, source) {
        println!("cargo::error=preparing Pagic icon integration failed: {e}");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if root.join("../../assets/app-icon/pagic.ico").is_file() {
        res.set_icon("../../assets/app-icon/pagic.ico");
    }
    if root.join("../../assets/app-icon/pagic-document.ico").is_file() {
        res.set_icon_with_id("../../assets/app-icon/pagic-document.ico", "2");
    }
    res.set_manifest_file("pagic.exe.manifest")
        .set("ProductName", "Pagic")
        .set("FileDescription", "Pagic PDF application")
        .set("LegalCopyright", "Copyright (c) the Recto contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "pagic.exe")
        .set("InternalName", "pagic");
    if let Err(e) = res.compile() {
        if std::env::var_os("PAGIC_REQUIRE_WINRES").or_else(|| std::env::var_os("RECTO_REQUIRE_WINRES")).is_some() {
            println!("cargo::error=embedding Windows resources failed: {e}");
            return;
        }
        println!("cargo:warning=pagic.exe built without icon/version resources: {e}");
    }
}
