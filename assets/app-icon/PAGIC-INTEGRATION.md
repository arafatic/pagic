# Pagic artwork handoff

All paths below are relative to the Pagic repository root. The three SVG masters supplied
and approved by the project owner are preserved unchanged, with distinct roles below. Desktop and MSIX image derivatives are generated.
See `PAGIC-PROVENANCE.md` for source hashes and the outstanding distribution-license details.
Do not relabel archived Recto artwork.

## Master inputs

- `assets/app-icon/pagic-app-icon.svg`: active application-icon master, a self-contained
  rounded-square vector (viewBox 1032.694 by 1032.694) with transparent corners. All application
  ICO/ICNS/PNG, Linux scalable and MSIX outputs use this master, without shape/color changes.
- `assets/app-icon/pagic.svg`: original brand logo (viewBox 330.68 by 330.68), retained unchanged;
  no longer used to generate application icons.
- `assets/app-icon/pagic-document.svg`: approved PDF document/file-association input, currently
  byte-identical to the original brand logo, not the rounded application-icon master. No distinct document graphic was invented.
- An optional horizontal wordmark is not currently consumed by the application; no wordmark
  or splash-screen destination is wired. Agree its use before adding files.

## Existing desktop integration paths

| Platform/use | Destination | Current generator output |
| --- | --- | --- |
| Windows executable/application | `assets/app-icon/pagic.ico` | PNG-compressed 32-bit entries: 16, 20, 24, 32, 40, 48, 64, 128, 256 px square |
| macOS PDF association | `assets/app-icon/pagic-document.icns` | Generated from the approved document master; bundled separately from the app icon |
| Windows PDF association | `assets/app-icon/pagic-document.ico` | Same sizes, executable resource ID 2; application icon is first/default |
| macOS runtime Dock | `assets/app-icon/pagic-1024.png` | 1024 px transparent canvas, 824 px body centered with 100 px inset |
| macOS bundle | `assets/app-icon/pagic.icns` | iconset 16/32/128/256/512 px, each at 1x and 2x, including 1024 px |
| Linux launcher/runtime | `assets/app-icon/hicolor/{S}x{S}/apps/local.pagic.desktop.png` | S = 16, 24, 32, 48, 64, 128, 256, 512 |
| Linux scalable launcher | `assets/app-icon/hicolor/scalable/apps/local.pagic.desktop.svg` | Approved rounded application-icon master |

The macOS inset is the existing generator's layout contract, not a requirement to redraw the
logo as the old rounded tile. Review the final silhouette before generation. The document
master is currently consumed by Windows only. No Linux/macOS document-icon registration is
introduced by this handoff. Windows document ICO generation remains based on `pagic-document.svg`. The runtime uses macOS PNG or hicolor/256x256 on other platforms.
Developer builds now use the supplied Pagic artwork at these existing integration points.

`packaging/icons.sh` generates desktop and MSIX images using resvg, Python and (on macOS)
iconutil. `python3 packaging/check-icons.py --write-manifest` records validated hashes in
`pagic-assets.sha256.json` (67 masters/derivatives); `python3 packaging/check-icons.py` verifies them. Visual, alpha,
container and source-reference checks are completed for Stage 3; native Windows/Linux checks
remain pending. The broader missing `ATTRIBUTION.toml` workflow remains unresolved; the local
icon manifest does not replace it or license clearance. Retain all existing license texts.

## MSIX image resources: generated, packaging inactive

`packaging/windows/msix/Assets/` contains the generated RGBA PNGs below. No MSIX manifest,
package, identity or signing pipeline is activated. Every application variant retains the supplied rounded dark
background and transparent corners. Unplated qualifiers suppress an extra Windows plate; they do not remove artwork
backgrounds or alter its rounded outline. Light/dark variants are identical because no alternate artwork was supplied.

- `Square44x44Logo.scale-{100,200,400}.png`: 44, 88, 176 px square.
- `Square150x150Logo.scale-{100,200,400}.png`: 150, 300, 600 px square.
- `StoreLogo.scale-{100,200,400}.png`: 50, 100, 200 px square.
- `Square44x44Logo.targetsize-{S}.png` and corresponding `_altform-unplated` and
  `_altform-lightunplated` files: S = 16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256.

A future manifest must reference the matching logical filenames and generate `resources.pri`
for qualified assets. Tile/background layout is separate from unplated application glyphs.
Wide/large tiles, badges and splash images are conditional on future manifest choices;
they are not existing application requirements. Store listing images/screenshots are separate
submission inputs, not substitutes for packaged icons.

Reference: [Microsoft app-icon construction guidance](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction).
These filenames are repository resource conventions, not Partner Center identity values.

## Current branding and historical notices

Active application/document branding uses the new Pagic files above. The focused Stage 3
audit removed 16 unused first-party Recto images from the current tree: the seven root
`recto*` image files and nine `hicolor/*/apps/local.recto.desktop.*` resources. They remain
recoverable from Git history. Keep `assets/app-icon/LICENSE-recto.txt` and historical
trademark/provenance notices; these do not describe the Pagic artwork.

Do not replace Lucide toolbar/action SVGs or `assets/icons/LICENSE-lucide.txt`, font binaries
and OFL notices under `assets/fonts/` and `crates/fonts/assets/`, vendored resources/licenses,
third-party marks, or historical PDF/compatibility fixtures. New artwork needs its own truthful
provenance record; it must not inherit the Recto artwork's authorship claim.
