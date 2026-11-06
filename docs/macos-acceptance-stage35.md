# Pagic Stage 3.5 — macOS acceptance corrections

Final cleanup validated and approved for separate local commits. No push, release or MSIX
packaging. Rounded-icon assets are committed separately from the acceptance corrections.

## Corrections

- All tools > Edit a PDF now executes the shared, guarded `edit.edit_text` action, keeping
  All tools mode/back navigation. No-document/restricted cases retain command guards.
- The Edit panel has an explicit active-mode checkbox and click/drag guidance. Empty cached
  paragraph inventories produce qualified feedback about what the editor could identify,
  including unsupported mappings/structures, outlines or scans; they do not assert text is
  absent. Security restrictions are explained. Existing apply errors still retain typed text.
- Native Edit includes Edit text & images. Display navigation is Export and Fill & Sign;
  the latter's accessibility name identifies it as a workspace. Internal Mode::Convert/Sign
  and serialized control values remain unchanged. The Edit group no longer displays Planned M7.
- macOS uses Settings in the panel tooltip and settings-dialog title; other platforms retain
  Preferences. Existing translated navigation/settings labels are reused.
- About contains only Pagic, the current version and Developed and maintained by Arafatic,
  plus its standard Close control. Help > Credits & Licenses retains historical Recto/open-source
  attribution and selectable verbatim project, egui, hayro, Lucide, bundled-font and historical
  notices. No original or third-party license/copyright text was removed or rewritten.
- The click/drag guidance remains unobtrusively below the active-mode control in the Edit panel.
  Geometry tests verify it does not overlap the PDF; inactive-mode tests verify it disappears.

## Verified licensing

Upstream's initial commit and current Cargo metadata already declare MIT OR Apache-2.0, while
only MIT texts were present. Existing LICENSE/LICENSE-MIT and original third-party notices
remain byte-identical. LICENSE-APACHE is the exact Apache Software Foundation text; the pinned
egui MIT notice matches upstream's dependency commit. README/Help license access/native credits now match the
existing declaration. Sources and SHA-256 hashes are recorded in `licenses/README.md`.
No dependency declaration/pin or authorship/license terms were invented or removed. The icon
artwork's separate rights status and broader transitive NOTICE inventory remain unresolved.

## Validation

| Check | Result |
| --- | --- |
| Targeted UI integration selection | 23 passed, 0 failed, 0 ignored |
| Menu/localization unit tests | 68 passed, 0 failed, 0 ignored |
| Command registry tests | 4 passed, 0 failed, 0 ignored |
| macOS `cargo check -p pagic --all-targets --locked` | PASS |
| Icon validator | PASS, all 67 masters/derivatives; no regeneration |
| Edited Rust formatting and whitespace | PASS |
| Original MIT/notices, dependency versions/revisions and vendor/font files | Unchanged |
| Cargo.lock metadata | Only two Windows printing Git URLs corrected; pins unchanged |
| Original Recto tracked files and Git configuration | Unchanged |

The entry-route regression clicks All tools > Edit a PDF, opens existing text, types a
replacement, applies, undoes/redoes, saves/reopens and verifies Back navigation. Additional
regressions cover source-font selection, selected formatting and retaining failed drafts.
About/Help tests verify the minimal About content and absence of its old attribution/button,
original MIT and canonical Apache terms, egui authorship, localized Chinese content, Help-menu
license access, Close and document-state preservation. No test was disabled. All 95 selected
tests passed; the full suite was not repeated.

Logs: `/private/tmp/pagic-final-acceptance-integration.log`, `pagic-final-acceptance-unit.log`,
`pagic-final-acceptance-commands.log`, `pagic-final-acceptance-macos.log` and
`pagic-final-acceptance-build.log`.

## Limits

The owner manually accepted existing-text editing, undo/redo/save, the rounded icon and general
navigation. The final About/Help relocation is tested headlessly and awaits a visual spot-check.
The unavailable resume's specific content was not inspected and no engine fix is claimed.
The PDF engine algorithms were not changed; engine/catalog.rs changes availability only.
New guidance/About copy is translated in Simplified Chinese and has the normal English
fallback in other locales. Existing Export/Fill & Sign/Settings translations are retained.

## Verified Windows lock metadata correction

Pinned upstream manifest:
https://github.com/arafatic/craft-libs/blob/e11f9d9aebea5f6b8e74643c1ea1ed172ba5f545/crates/print/Cargo.toml
Git blob `1e6a63507e7a704ccc1994cb7a839848f60b0608` declares `storytold/winprint.rs`.
Cargo.lock incorrectly retained `arafatic/winprint.rs` for exactly two records. Corrected URLs:

- craft-winprint-gdi 0.1.0: revision `417ac3c4d14d3da5485a99c7d5ce8f05664ec312` unchanged.
- winprint 0.2.1: revision `5f80b9baf6b58937c401656634e42d33f00ddf49` unchanged.

The complete lock diff is exactly two URL replacements. No package version, dependency list,
registry checksum, pinned revision or craft-libs declaration changed. Locked validation now
works directly in Pagic, without the temporary source-copy workaround used for manual launch.

## Remaining Windows/MSIX work

Native Windows build/resource embedding and installer/association/install/update/uninstall
acceptance; actual Partner Center identity; MSIX manifest/PRI and version/signing pipeline;
complete transitive NOTICE/provenance inventory and artwork redistribution details. Release
packaging stays disabled. No Partner Center values or new licensing terms were invented.

## Stage 3.5 file inventory

Excludes the separately committed rounded-icon changes:

- `Cargo.lock`
- `LICENSE-APACHE`
- `README.md`
- `crates/engine/src/catalog.rs`
- `crates/engine/src/commands.rs`
- `crates/ui-egui/src/app/commands.rs`
- `crates/ui-egui/src/chrome.rs`
- `crates/ui-egui/src/commands.rs`
- `crates/ui-egui/src/dialogs.rs`
- `crates/ui-egui/src/dialogs/licenses.rs`
- `crates/ui-egui/src/home.rs`
- `crates/ui-egui/src/i18n/tests.rs`
- `crates/ui-egui/src/i18n/zh-hans.tsv`
- `crates/ui-egui/src/menus.rs`
- `crates/ui-egui/src/panels/edit.rs`
- `crates/ui-egui/src/state.rs`
- `crates/ui-egui/src/workflows/js_ui.rs`
- `crates/ui-egui/tests/it/control.rs`
- `crates/ui-egui/tests/it/editing.rs`
- `crates/ui-egui/tests/it/ui.rs`
- `docs/macos-acceptance-stage35.md`
- `licenses/README.md`
- `licenses/egui-MIT.txt`
- `packaging/macos/Credits.rtf`
