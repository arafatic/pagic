# Pagic Stage 1: compatibility and identity foundation

Baseline: `9fded4f241f8b457dd68e751404c9b59afbeab04`, branch
`codex/pagic-identity-foundation`. Local `main` remains the rollback point.
Origin is `https://github.com/arafatic/pagic.git`; no Recto push remote is configured.
The pre-migration bundle is outside both projects at
`/Users/arafat/Documents/Repository Backups/recto-before-pagic-2026-11-03_142157/recto-all-refs.bundle`.
SHA-256: `41ab7795a205c678352629d0e415bd63c2d3882e565ce4c92878a9e348dec8ba`.

## Runtime boundaries

`recto_engine::identity` and `packaging/pagic-identity.json` define the Pagic namespaces.
Cargo packages, executable targets, window titles and artwork intentionally retain their
current names in this stage. Desktop app ID and macOS bundle ID are `local.pagic.desktop`;
this is not a Microsoft Store identity.

Settings use eframe's Pagic data directory (or XDG `pagic/app.ron`) and the `pagic`
storage key. Portable mode writes `PagicData`; `recto.portable` does not enable it.
The generic `portable.txt` still works, but never causes writes to `RectoData`.
Recovery and newly created digital IDs use the separate Pagic location. Single-instance
sockets use `Pagic` on Windows and `pagic` on Unix. Control endpoints are application-tagged
and published without replacing an existing file or link: use a fresh `--control` filename
for each launch. Old endpoint files must be inspected/removed explicitly by their owner.

No Recto/PrintCraft directory is moved on startup. The existing same-application Linux
migration remains Pagic data -> Pagic XDG configuration, not Recto -> Pagic.

## Explicit preference import

`recto_ui_egui::preference_import::preferences` accepts a read-only source storage and a
separate destination storage. It reads the existing `recto`/`printcraft` JSON preference
format, keeps only reviewed non-file-bearing preferences and writes the `pagic` key only
when absent. It does not run automatically, expose a new UI, flush files, copy directories,
open recent documents or enumerate recovery. The caller must explicitly select the source
and flush its own destination. Existing Pagic state always wins, even if malformed.

Private keys, digital-ID references, trusted certificates, signatures, recent paths, sessions,
recovery and unknown future fields are excluded. A future import UI needs separate review.
Recovery import needs source-session lock checks and password handling; key import must be
explicit. Neither is implemented by this preference-only operation.

## Compatibility fixtures

`crates/engine/tests/it/fixtures/recto-marks.pdf` was generated with the unchanged baseline
PDF writer by `cargo run -p recto-engine --example recto_compat_fixture --locked` before any
writer rebranding. It contains `/PCMark`, `q %Recto` and the existing closing wrapper, a
watermark and ordinary document text. Do not regenerate it automatically or rename its markers.
Engine tests edit, save incrementally/in full, reopen and remove the old mark. Encryption and
owner restrictions are checked; the existing OpenSSL-signed fixture proves unchanged saves
and rejected full rewrites/sanitization. Existing legacy-close tests remain in recto-edit.
The original signed fixture's attribution is in `crates/sign/tests/data/README.md`.

The preference fixture uses the baseline enum/field serialization (for example `Light`,
`millimetres`). Encrypted recovery and key-ownership tests use the real engine encryption
path and existing test-only PKCS#12 identity, not user keys.

## Installation and update policy

`packaging/pagic-identity.json` reserves independent Inno/MSI identities, a Pagic ProgID,
registry root and eventual packaged executable name. No Partner Center identity is invented.
Legacy Windows/Linux release recipes are **not approved Pagic installers yet**: their file,
shortcut and registry ownership must be integrated and checked during the next packaging
stage, together with executable/resource naming. Do not distribute them as Pagic or run
installation tests against real Recto installations. Stage 1 tests validate the contract,
not a compiled native installer. The macOS development runner uses a separate Pagic.app.

The update API is deliberately unconfigured. Neither startup nor a manual check contacts
Recto or the unauthenticated private Pagic release API. Manual checks explain the absence
of an update source. Choosing a distribution/update endpoint is a separate decision.

## Rollback and validation

Source/legacy preferences, recovery and key files remain untouched. Reverting this branch
leaves Recto able to read its own original data. Shared PDF markers and existing automation
protocols stay intact. Neither a commit nor a push is part of this stage.

Run engine compatibility tests, UI import/ownership/update/recovery tests and desktop
settings/control/socket tests. Pure path tests cover all OS branches but are not native
Windows/Linux acceptance. Native installer/uninstall, file activation and coexistence checks
remain mandatory before distribution. Do not create MSIX packages in Stage 1.

## Verified results on the macOS development host

| Check | Result |
|---|---|
| UI integration | ok. 1177 passed; 0 failed; 10 ignored; 0 measured; 0 filtered out; finished in 17.32s |
| UI unit | ok. 590 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.18s |
| Desktop/security | ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s |
| Engine compatibility | ok. 3 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.12s |
| Engine identity | ok. 1 passed; 0 failed; 0 ignored; 0 measured; 374 filtered out; finished in 0.00s |
| Legacy marker handling | ok. 7 passed; 0 failed; 0 ignored; 0 measured; 275 filtered out; finished in 0.04s |
| Workspace compilation | `cargo check --workspace --locked`: PASS |
| Formatting | All edited/new Rust files PASS; workspace-wide check reports 84 unchanged baseline files |
| Diff whitespace | `git diff --check`: PASS |

There are 14 net additional tests; five existing tests were replaced with stronger endpoint/update ownership-policy tests. No new ignored tests were added. The 11 ignored UI tests predate this branch. Windows/Linux native execution and installer/uninstall ownership are not validated here.

## Exact changed/new paths

- `apps/recto/src/main.rs`
- `apps/recto/src/settings.rs`
- `apps/recto/src/single_instance.rs`
- `apps/recto/src/tests.rs`
- `crates/engine/examples/recto_compat_fixture.rs`
- `crates/engine/src/identity.rs`
- `crates/engine/src/lib.rs`
- `crates/engine/tests/it/fixtures/README.md`
- `crates/engine/tests/it/fixtures/recto-marks.pdf`
- `crates/engine/tests/it/main.rs`
- `crates/engine/tests/it/recto_compatibility.rs`
- `crates/ui-egui/src/app.rs`
- `crates/ui-egui/src/lib.rs`
- `crates/ui-egui/src/persist.rs`
- `crates/ui-egui/src/persist/import.rs`
- `crates/ui-egui/src/persist/recovery.rs`
- `crates/ui-egui/src/platform/macos_association.rs`
- `crates/ui-egui/src/platform/portable.rs`
- `crates/ui-egui/src/updater.rs`
- `crates/ui-egui/tests/fixtures/recto-preferences.json`
- `crates/ui-egui/tests/it/identity_foundation.rs`
- `crates/ui-egui/tests/it/main.rs`
- `crates/ui-egui/tests/it/updates.rs`
- `docs/rebranding-stage1.md`
- `packaging/linux/local.recto.desktop.desktop`
- `packaging/macos/Info.plist.in`
- `packaging/macos/run.sh`
- `packaging/pagic-identity.json`
