# Pagic Stage 2: application rebranding

Stage 1 is committed separately as `9e641a5` (identity isolation and historical compatibility).
Stage 2 renames first-party Cargo packages/imports, binary targets and app directories to
Pagic. Core crate directory suffixes and third-party dependency versions/pins stay unchanged.
The desktop/CLI/web entry points are `apps/pagic`, `apps/pagic-cli`, `apps/pagic-web`.

Menus, titles, Home/About, diagnostics, accessibility labels, localization lookup keys and
translations now use Pagic. Translation changes are limited to product-bearing rows:
French `recto verso` and other ordinary technical words are not application branding.

## Preserved compatibility and attribution

- Frozen Recto PDF/preference fixtures and historical notices retain their bytes and names.
- `%Recto`, `/PCMark`, `/PCAdded`, generated font/profile identifiers and old signed bytes
  are document formats/provenance, not UI labels. Existing readers remain unchanged.
- Explicit import reads the old `recto`/`printcraft` storage keys and never overwrites Pagic
  preferences or imports private keys, recovery, signatures or document paths.
- Existing `recto://` MCP resource URIs and `recto:*` embedding messages are version-one
  compatibility protocols. Server/app display names and the web canvas use Pagic.
- Runtime `PAGIC_*` environment overrides take precedence; `RECTO_*` inputs remain aliases.
- Original copyright, font/third-party licenses, contributor attribution and vendored source
  are retained. New documents may identify Pagic as their producer; old files are not rewritten
  merely to change their metadata.

## Application and installer identities

`packaging/pagic-identity.json` remains the identity contract: `local.pagic.desktop`,
Pagic/pagic data namespaces, independent Inno/MSI IDs and `pagic.document`. Source checks
compare Windows registration rows with WiX and reject Recto registry/product/component
ownership. Mac bundle and Linux launcher/AppStream identities are checked against the same
contract. No Partner Center identity is inferred or assigned.

Legacy recipes are still inactive: native packaging scripts and the release workflow require
`packaging/check-identity.py --require-release-ready`. The contract has
`release_packaging_enabled: false`; no environment override bypass is provided. Static
identity checks do not certify Windows/Linux installation, repair, association or uninstall.
Manual workflow dispatch still cannot publish; the existing tag-only publication guard remains.

## Artwork and distribution blockers

No logo was invented or relabelled. See `assets/app-icon/PAGIC-INTEGRATION.md` for the expected
master/generated input paths. Developer builds use the platform default icon when final
artwork is absent. The old Recto artwork and its provenance remain archived and byte-identical.

Release/installer activation still requires final artwork, native Windows/Linux/macOS package
acceptance and resolution of missing notice/model inputs. MSIX additionally needs real Partner
Center identity values, a manifest/assets/package pipeline, signing/version policy and native
activation/update/uninstall/certification checks. No MSIX package is created in this stage.

The update service remains unconfigured for the private repository; no Recto release endpoint
or embedded credential is used. No changes are pushed, merged or published by this stage.

## Verification scope

Checks run on the macOS development host. Targeted tests cover product names, menus,
accessibility, locales, preference import, ownership, fonts, legacy PDFs and MCP resources.
The full workspace suite is not repeated. Native Windows/Linux installers and browser Wasm
are not validated here; only the macOS Rust target is installed. PowerShell and actionlint
are not available locally. Shell, Python, YAML and platform-source identity syntax/contracts
are checked with available tools. Results follow.


| Validation | Result |
| --- | --- |
| Workspace all-target compilation (`cargo check --workspace --all-targets --locked`) | PASS on aarch64-apple-darwin |
| Historical PDF/encryption/signature compatibility | 3 passed |
| Preference import/localization/identity/updater unit tests | 69 passed |
| Targeted UI integration tests (Home/About, menus/languages, identity, fonts) | 38 passed |
| Desktop/security unit tests | 69 passed |
| CLI targeted integration tests | 18 passed |
| MCP compatibility tests | 6 passed |
| Historical URL link-edit fixture | 1 passed |
| Font tests and font build-script tests | 90 + 3 passed |
| Historical document markers and engine identity | 7 + 1 passed |
| Platform packaging identity policy tests | 4 passed |
| Font-notice shell regression tests | PASS, including environment precedence |
| Workspace layering | PASS, 37 crates |
| Edited Rust formatting / whitespace | PASS |
| Whole-workspace formatting | Existing extra blank lines in unchanged `crates/ui-egui/src/dialogs/credits.rs`; left untouched |
| Shell/Python/YAML syntax | PASS with available tools |

Total selected Rust tests: **305 passed, zero failed**; packaging policy: **4 passed**.
No tests were disabled or newly ignored. Logs are retained under ignored `target/stage2-*.log`.
Cargo lockfile comparison confirms all 750 external package records retain their versions,
source revisions, checksums and dependency relationships. Changes are first-party names only.
Frozen compatibility fixtures, original artwork, vendor files and license assets remain
byte-identical. The original Recto checkout and its Git configuration match the saved
migration baseline. Stage 2 is committed separately after approval; the Stage 1 commit remains unchanged.

Synthetic test URLs such as `https://recto.dev` are retained as historical test data, not
application download/update endpoints. Source identity checks and packaging gates are
preparation only: final artwork, required notice/model inputs and native packaging acceptance
remain blockers. No cross-platform native runner success is claimed.

Pre-commit review: every removed source/recipe path has a corresponding renamed Pagic path.
Historical fixtures, archived artwork, bundled fonts/licenses and vendored source remain
byte-identical. Duplicate Windows build-script environment checks were removed and the legacy
preference-import comment corrected without changing behavior. The complete artwork handoff
is `assets/app-icon/PAGIC-INTEGRATION.md`; MSIX paths there are explicitly proposed only.
