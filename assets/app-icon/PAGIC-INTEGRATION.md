# Pagic artwork handoff

All paths below are relative to the Pagic repository root. No artwork is generated in this
stage. Supply approved original artwork and its author, copyright, source and permission/license
before generating or distributing derivatives. Do not relabel archived Recto artwork.

## Master inputs

- `assets/app-icon/pagic.svg`: square application symbol; self-contained vector paths,
  outlined text if any, no external fonts/images/links. The current generator assumes a
  full-bleed 512-by-512 viewBox. Approve small-size legibility and background treatment.
- `assets/app-icon/pagic-document.svg`: separate square PDF document/file-association symbol,
  subject to the same vector requirements. Do not use an unlicensed third-party PDF logo.
- An optional horizontal wordmark is not currently consumed by the application; no wordmark
  or splash-screen destination is wired. Agree its use before adding files.

## Existing desktop integration paths

| Platform/use | Destination | Current generator output |
| --- | --- | --- |
| Windows executable/application | `assets/app-icon/pagic.ico` | PNG-compressed 32-bit entries: 16, 20, 24, 32, 40, 48, 64, 128, 256 px square |
| Windows PDF association | `assets/app-icon/pagic-document.ico` | Same sizes, executable resource ID 2; application icon is first/default |
| macOS runtime Dock | `assets/app-icon/pagic-1024.png` | 1024 px transparent canvas, 824 px body centered with 100 px inset |
| macOS bundle | `assets/app-icon/pagic.icns` | iconset 16/32/128/256/512 px, each at 1x and 2x, including 1024 px |
| Linux launcher/runtime | `assets/app-icon/hicolor/{S}x{S}/apps/local.pagic.desktop.png` | S = 16, 24, 32, 48, 64, 128, 256, 512 |
| Linux scalable launcher | `assets/app-icon/hicolor/scalable/apps/local.pagic.desktop.svg` | Approved application master |

The macOS inset is the existing generator's layout contract, not a requirement to redraw the
logo as the old rounded tile. Review the final silhouette before generation. The document
master is currently consumed by Windows only. No Linux/macOS document-icon registration is
introduced by this handoff. The runtime uses macOS PNG or hicolor/256x256 on other platforms.
Developer builds use platform defaults while Pagic artwork is absent.

After artwork approval, `packaging/icons.sh` can generate these resources using resvg,
Python and (on macOS) iconutil. Do not run it yet. Its output must undergo visual inspection,
provenance/hash recording and native resource checks. The missing `ATTRIBUTION.toml` workflow
must be resolved before claiming those checks complete; retain existing license texts.

## MSIX: proposed paths, not yet implemented

Reserve `packaging/windows/msix/Assets/` for package PNGs; no directory, manifest or generator
has been activated. Use RGBA PNGs with reviewed transparency and theme contrast.

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
These filenames are a proposed repository convention, not Partner Center identity values.

## Replace use, preserve originals

Replace active application/document branding with the new Pagic files above. Keep archived
`recto.svg`, `recto-small.svg`, `recto-document.svg`, `recto-1024.png`, `recto.icns`, both
`recto*.ico` files and `hicolor/*/apps/local.recto.desktop.*` unchanged; they are no longer
Pagic runtime inputs. Retain `assets/app-icon/LICENSE-recto.txt` and historical provenance.

Do not replace Lucide toolbar/action SVGs or `assets/icons/LICENSE-lucide.txt`, font binaries
and OFL notices under `assets/fonts/` and `crates/fonts/assets/`, vendored resources/licenses,
third-party marks, or historical PDF/compatibility fixtures. New artwork needs its own truthful
provenance record; it must not inherit the Recto artwork's authorship claim.
