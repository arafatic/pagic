# Stage 3: approved Pagic logo integration

Status: generated, locally validated and approved for a separate Stage 3 commit. No push, release,
installer or MSIX package was created. Stage 2 identities remain unchanged.

## Inputs and preservation

The owner-supplied `assets/app-icon/pagic.svg` and `pagic-document.svg` are valid, identical,
self-contained square SVGs (viewBox 330.68 by 330.68). Their original SHA-256 is
`b30946388d16aec1adc1468167b7a6e65420f62fad0f094d46d87c18942f697c`.
Both originals retain their exact bytes, geometry, colors and opaque background. No logo,
PDF-page graphic, rounded mask or theme artwork was invented. macOS adds only the agreed
transparent inset, with proportional scaling. MSIX theme variants keep the approved plate.

Authorship/license details are not embedded in the SVGs or separately supplied. See
`assets/app-icon/PAGIC-PROVENANCE.md`; resolve accurate redistribution terms before release.
The archived Recto license does not describe the new artwork.

## Changed files

- Modified `packaging/icons.sh`: validate square vector inputs, preserve the supplied master,
  generate the specified 51 MSIX PNGs as well as the existing desktop outputs.
- Modified `assets/app-icon/PAGIC-INTEGRATION.md`: record actual inputs, generation/validation
  commands and MSIX image-only status.
- Added the two supplied SVGs and 64 generated derivatives: two Windows ICOs, two macOS
  PNG/ICNS resources, nine Linux PNG/SVG resources and 51 MSIX PNG resources.
- Added `assets/app-icon/pagic-assets.sha256.json`: exact paths and hashes of all 66 assets.
- Added `assets/app-icon/PAGIC-PROVENANCE.md`, `packaging/check-icons.py` and this report.

Existing build/installer resource references already use these Pagic filenames. No source
change was needed to activate them. The latest macOS build-script output binds
`APP_ICON_PNG` to the new `pagic-1024.png` with `Some(include_bytes!(...))`.
MSIX has image resources only; no manifest, PRI or package has been introduced.

## Validation

- `python3 packaging/check-icons.py`: PASS for 66 assets; PNG dimensions/CRC/RGBA decoding,
  expected alpha/insets, all nine entries in each ICO, ten ICNS representations including
  iconutil round-trip decoding, source references and SHA-256 manifest.
- Direct approved 256 px SVG render and Linux runtime PNG: byte-identical. Visual inspection
  of the direct render and macOS 1024 px render confirms unchanged artwork and uniform scale.
- `cargo check -p pagic --all-targets --locked`: PASS on aarch64-apple-darwin.
- `cargo xtask assets`: PASS in existing local-font fallback mode, verifying 13 font files.
  `ATTRIBUTION.toml` remains absent; this does not certify full-repository attribution.
- `python3 packaging/check-identity.py`: PASS; four packaging identity tests PASS.
- `bash -n packaging/icons.sh` and `git diff --check`: PASS.

Renderer: existing locked resvg 0.45.1 via a temporary rendering helper; macOS 26.3.1 iconutil.
No Rust dependencies, PDF functionality, app identities or packaging activation flags changed.
Unused first-party Recto images were removed after the audit below; font/license, vendor
and compatibility fixture bytes are unchanged. Original Recto files and Git
configuration match the saved migration baseline. No full test suite was repeated.

## Remaining limits

Windows PE embedding, Linux desktop display, packaged MSIX image selection and native
install/update/uninstall behavior await native validation. Full attribution inputs, artwork
rights details, actual Partner Center identities, manifest/PRI generation and package/signing
policy remain separate distribution blockers. Legacy release/installer gates remain disabled.


## Focused archived-branding audit

Sixteen remaining images were original Recto-owned artwork according to
`assets/app-icon/LICENSE-recto.txt` and the prior README. There were no other tracked
first-party promotional images/screenshots outside the icon directory. `docs/brand/`
contains historical trademark guidance, not a remaining image.

No active Rust build/test or direct packaging image reference required these old files.
References were in the old icon README and historical stage reports. Linux, FreeBSD and
Flatpak recipes copied the entire hicolor directory; removing obsolete images prevents
unnecessary Recto icons from being included through those directory-wide copies. Current
build/packaging references already point to Pagic. The macOS recipe retains the historical
LICENSE-recto.txt notice; it does not load or copy the removed Recto images.

Removed from the current working tree (history unchanged):
- `assets/app-icon/recto.svg`
- `assets/app-icon/recto-small.svg`
- `assets/app-icon/recto-document.svg`
- `assets/app-icon/recto-1024.png`
- `assets/app-icon/recto.icns`
- `assets/app-icon/recto.ico`
- `assets/app-icon/recto-document.ico`
- `assets/app-icon/hicolor/128x128/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/16x16/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/24x24/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/256x256/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/32x32/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/48x48/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/512x512/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/64x64/apps/local.recto.desktop.png`
- `assets/app-icon/hicolor/scalable/apps/local.recto.desktop.svg`


Preserved byte-for-byte: `assets/app-icon/LICENSE-recto.txt`,
`docs/brand/LICENSE-brand.txt`, Lucide SVGs and their license, bundled fonts/OFL notices,
vendored resources/licenses, historical Recto PDF and preference fixtures. Product ownership
is based on the existing notices; no third-party asset was classified as first-party merely
because its filename contains Recto. Only the 16 reviewed images were deleted.

The current artwork README, integration guide and provenance record now explain removal
rather than describing old images as available current inputs. Historical Stage 1/2 reports
retain their original facts. New Pagic artwork's unresolved rights details and native
Windows/Linux/MSIX limitations remain as described above.
