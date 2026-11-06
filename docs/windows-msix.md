# Windows MSIX packaging (Stage 4A)

This is a separate Windows x64 validation pipeline, triggered manually or by a push to the
explicit validation branch `codex/pagic-msix-validation`. It does not enable the
legacy installer gate, modify ZIP/EXE/MSI recipes, publish releases, submit to Partner Center,
or configure an application updater. No MSIX has been native-validated until the Windows
workflow runs successfully. A passing packaging job is not Store certification.

## Identity and version

The reviewed Partner Center contract is `packaging/windows/msix/identity.json`:

| Field | Value |
| --- | --- |
| Identity Name | `Arafatic.Pagic` |
| Publisher | `CN=8E27432A-B6BC-4F20-8810-A2256A3F60AF` |
| PublisherDisplayName | `Arafatic` |
| DisplayName / Application Id | `Pagic` |
| Executable / architecture | `pagic.exe` / `x64` |
| Minimum OS | Windows 10 2004 (`10.0.19041.0`) |

Cargo remains the application version source. Store versions use
`(Cargo major + 1).minor.patch.0`: Cargo `0.14.0` becomes MSIX `1.14.0.0`.
The offset gives pre-1.0 Cargo releases a nonzero first component and preserves monotonic
ordering when Cargo reaches 1.0. The fourth component is reserved for the Store and stays zero.
Prereleases, build suffixes and overflowing components are rejected rather than truncated.
Do not change this mapping after shipping without reviewing installed-package upgrade order
and the versions already accepted in Partner Center. The app still displays its Cargo version.

The manifest uses the existing Win32 executable with `uap10:RuntimeBehavior="packagedClassicApp"` and `uap10:TrustLevel="mediumIL"`,
plus only the `runFullTrust` capability; this is a full-trust desktop package, not a sandboxed UWP port. Explain this
capability during Store submission. No administrator elevation, broad-filesystem-access
capability, startup entry, updater or legacy ProgID migration is declared.
The CLI has a console execution alias `pagic-cli.exe`, explicitly targeting the packaged CLI.
Windows registers it under `%LOCALAPPDATA%/Microsoft/WindowsApps`; the installation test
verifies the installed declaration and registration, then invokes that alias with UTF-8 pipes.
It never changes protected-directory permissions or launches the CLI payload directly.
An existing alias causes the disposable-runner test to stop rather than take it over.
The PDF association advertises Open with Pagic and passes a quoted path to the existing
CLI argument parser. It does not set or replace the user's default PDF handler.

## Build on native Windows

Requirements: x64 Windows, PowerShell 7, Python >=3.9, Rust stable (workspace minimum 1.95),
Visual Studio C++ x64 build tools and Windows SDK (MakePri, MakeAppx, SignTool and rc.exe).
Use a Visual Studio developer PowerShell so `dumpbin` and the MSVC linker are available.

```powershell
rustup target add x86_64-pc-windows-msvc
python packaging/windows/msix/test_msix.py -v
cargo xtask assets
./packaging/windows/msix/package.ps1
```

The script compiles both binaries with `--locked`, the x64 MSVC target and static CRT;
it requires resource embedding and all local supplemental fonts. Do not inject global
`RUSTFLAGS`. Font resolution is shared with the existing notice helper:
`PAGIC_FONTS_DIR`, legacy `RECTO_FONTS_DIR`, otherwise `crates/fonts/assets`.
Use the same override for compilation and packaging. `-SkipBuild` is for explicitly verified
matching binaries only; CI always compiles. Fonts and UI resources are embedded in the binaries;
all base/supplemental OFL notices are also installed under `Legal/fonts`.

A fresh staging directory holds the binaries, 51 approved rounded PNG variants, a document-logo
PNG extracted unchanged from `pagic-document.ico`, original project/vendor/font notices, and
resolved Cargo dependency license texts plus a source/SPDX/hash inventory. Missing dependency
notice texts are explicitly reported for review. Gathering a conservative superset of resolved
Cargo packages can include build/test dependencies; it is not a legal determination.

MakePri creates `resources.pri` with all scale/targetsize variants in the main package.
MakeAppx packs with semantic validation enabled, then unpacks the actual artifact. Independent
checks enforce identity, PE architecture/subsystem, quoted PDF activation, icon hashes,
font-input/notice hashes and byte-for-byte staged/package/unpacked payload agreement. PE imports are
recorded for native runtime dependency review. No portable marker or user data is staged.
No fake `.msixupload` archive is generated.

Outputs in `dist/msix/`:

- `Pagic-1.14.0.0-x64.msix` (current version; unsigned)
- `AppxManifest.xml`, `validation.json`, `build-info.json`, `SHA256SUMS`
- `dependency-inventory.json` (including unresolved notice gaps), PE import reports
- Installation/UI-state reports and the sample PDF when installation smoke testing succeeds

Existing artifacts are not overwritten. Generated `work-<GUID>` directories remain available
for diagnosing local SDK failures; they can be removed after reviewing outputs. They contain
no source inputs that cannot be recreated. Do not commit build outputs or certificate material.

## Private GitHub Actions validation

The workflow `.github/workflows/msix.yml` accepts `workflow_dispatch` and pushes only to
`codex/pagic-msix-validation`, with read-only repository permissions and artifact upload.
It has no release/push/tag/submission steps and uses no signing secrets. Artifacts stay in this private repository's Actions storage for 14 days.

A reviewed push to the dedicated validation branch triggers Windows validation. Keep CI artifacts
internal and use the Microsoft Store as the public Windows distribution channel:

```sh
# Only after explicit commit/push approval; never force-push or target main.
git push origin HEAD:refs/heads/codex/pagic-msix-validation
gh run list --repo arafatic/pagic --workflow msix.yml --branch codex/pagic-msix-validation --limit 5
gh run watch <run-id> --repo arafatic/pagic --exit-status
gh run download <run-id> --repo arafatic/pagic --dir <private-download-directory>
```

Verify the run's `headSha` equals the reviewed commit. Branch-triggered runs always perform
the native installation smoke test. Manual runs allow selecting it explicitly:

```sh
gh workflow run msix.yml --repo arafatic/pagic --ref <approved-branch> -f install_smoke=true
```

GitHub requires a dispatchable workflow on the default branch before manual dispatch;
use the restricted branch push for the initial validation instead. Merging into `main`
requires separate approval after native results are reviewed. Nothing in these scripts
pushes or merges on your behalf. Native Windows builds can also run locally from the checkout.

The optional installation smoke test is restricted to disposable GitHub-hosted Windows runners.
It refuses to replace an existing package, creates a short-lived self-signed development
certificate with the exact Publisher, signs a COPY, trusts only that certificate, installs,
checks identity and the PDF association declaration, and launches the installed executable
with a generated one-page PDF. The existing control API must report the correct open document.
It removes only its own package and certificates and never exports a private key. The unsigned
artifact is preserved. Graphics/session failures must be investigated, not silently skipped.
This checks installed-executable launch, not Start menu/Explorer activation or Store delivery.

## Signing, acceptance and submission gates

Microsoft Store distribution is signed by Microsoft after certification; a purchased signing
certificate is not required for this path. The unsigned MSIX is a validation/submission candidate,
not an installable public release. Test sideloading requires a certificate trusted by that test
machine; never distribute the runner's development certificate as a production identity.
A separate non-Store release would require its own approved production signing process.

Before submission, all of these remain required:

1. Native Windows build, MakePri/MakeAppx, installation and PDF-opening results; inspect PE imports
   for missing non-system DLLs, and test on a clean machine without development tools installed.
2. Start menu/Dock-equivalent visuals, Explorer Open with (including spaced/non-ASCII paths),
   default-handler consent, save/reopen/edit, printing, signing, uninstall and upgrade tests.
3. Coexistence with Recto. Existing namespaces remain `%APPDATA%/Pagic`, `%LOCALAPPDATA%/Pagic`,
   Pagic recovery/control/single-instance paths; MSIX adds its package-family identity and Windows
   file/registry virtualization. Verify actual redirected locations and uninstallation ownership
   natively. Do not automatically import/move Recto preferences, private keys or encrypted recovery.
   Packaged and unpackaged Pagic coexistence is unverified; they may share Pagic paths/endpoints.
4. Verify `dependency-inventory.json` and selected license terms. Workspace-omitted upstream
   notices for 41 crates (`AccessKit/accesskit`, `dropbox/rust-alloc-no-stdlib`, `boa-dev/boa`,
   `DoumanAsh/clipboard-win`, `RustCrypto/formats`, `google/flatbuffers`, `brendanzab/gl-rs`,
   `robertknight/ocrs`, `aclysma/profiling`, `robertknight/rten`, `prefix-dev/sigstore-rust`,
   `gfx-rs/rspirv`) are supplied from their pinned `.cargo_vcs_info.json` Git commits in
   `licenses/upstream/` (`licenses/upstream-notices.json`), and REUSE `LICENSES/` files in
   `enum-map` and `enum-map-derive` are staged verbatim. The remaining 3 entries in
   `missing_dependency_notice_files` (`fmt-derive@0.1.2`, `fmt-derive-proc@0.1.2`, `rustls-cng@0.7.1`)
   have no standalone `LICENSE` file in their upstream Git repositories at the pinned commits;
   their verbatim `Cargo.toml.orig`/`README.md` declarations are staged and hash-verified while
   `unverified_dependency_notice_files` is empty (`0`). These declarations are formally reviewed
   and accepted; no license text is invented or altered. Preserve existing notices, copyright and
   historical attribution. The owner has documented explicit artwork ownership and Store distribution
   rights in `assets/app-icon/PAGIC-ARTWORK-RIGHTS.md`. This pipeline does not invent a root `NOTICE` or
   rewrite legal notices.
5. OCR model packaging policy: OCR models are excluded from the v1 Microsoft Store package.
   The Store listing must not advertise bundled offline OCR. The application documentation and listing
   disclose that text recognition requires models installed separately by the user via the `PAGIC_MODELS`
   directory (or local `models/` folder). When models are absent, Pagic displays a clear, graceful
   end-user notice instructing the user how to configure them without failing or crashing. Unreviewed
   models are neither bundled nor downloaded.
6. Windows App Certification Kit results on supported Windows hardware, Store restricted-capability
   review, privacy policy/support URLs, screenshots/listing, content/age ratings and export declarations.
   Verify all identity strings against Partner Center and ensure the version exceeds any prior upload.
   x86/ARM64 and Windows versions below the declared minimum are not supported by this x64 pipeline.

## Stage 4B Microsoft Store submission checklist

- [ ] **Partner Center Identity Match**: Confirm Partner Center app identity matches `packaging/windows/msix/identity.json` (`Name="Arafatic.Pagic"`, `Publisher="CN=8E27432A-B6BC-4F20-8810-A2256A3F60AF"`, `PublisherDisplayName="Arafatic"`, `DisplayName="Pagic"`).
- [x] **Artwork & Brand Redistribution Rights**: Owner/author rights declaration recorded in `assets/app-icon/PAGIC-ARTWORK-RIGHTS.md` and referenced in `PAGIC-PROVENANCE.md` (author ownership, reproduction/distribution rights for app packages and Microsoft Store, no third-party logo infringement). Resolver confirmed.
- [x] **OCR Model Decision & Disclosure**: OCR models are excluded from the v1 Store package. The Store listing description is restricted from claiming bundled offline OCR and explicitly notes that OCR requires separately installed models via `PAGIC_MODELS`. No unreviewed model files are downloaded or packaged.
- [x] **Third-Party Attribution & 3-Crate License Sign-Off**: `Legal/dependency-inventory.json` has `unverified_dependency_notice_files: []`. Acceptance of the 3 crates (`fmt-derive`, `fmt-derive-proc`, `rustls-cng`) without standalone upstream `LICENSE` files is formally documented based on their verified, staged `Cargo.toml.orig` and `README.md` dual-license declarations (`MIT OR Apache-2.0` / `MIT/Apache-2.0`), with no invented license terms.
- [ ] **Interactive Desktop & WACK Sign-Off**: Run Windows App Certification Kit (WACK) on clean x64 Windows hardware (`>= 10.0.19041.0`) and verify interactive Start menu launch, Explorer `Open with Pagic` on `.pdf` paths with spaces/Unicode, save/edit/print, `pagic-cli.exe` execution alias, coexistence with Recto, and clean uninstall.
- [ ] **Partner Center Submission Metadata**: Provide `runFullTrust` capability justification (packaged Win32 desktop PDF editor/printer/CLI), privacy policy URL, support contact URL, age/content ratings, export compliance declaration, and real Pagic screenshots.
- [ ] **Unsigned MSIX Upload & Version Check**: Upload the unsigned `Pagic-<version>-x64.msix` from `dist/msix/` (verified against `SHA256SUMS` and `validation.json`) with strictly increasing 4-part version (`1.14.0.0` for Cargo `0.14.0`, fourth component `0`) for Microsoft Store signing.

`build-info.json` deliberately records `store_ready: false`: archive validity alone cannot clear
these human/native certification gates. No Partner Center API credentials or automatic submission
are configured. After approval and all gates, manually create a submission in Partner Center,
select the reviewed MSIX (or a separately validated bundle/upload package), complete listing and
certification information, review validation findings, then submit only with explicit authorization.

Official references (reviewed for this implementation):

- [MakeAppx and semantic validation](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool)
- [Store package/version/signing requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)
- [MSIX signing for development and Store distribution](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide)
- [Packaged desktop PDF associations and quoted arguments](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-extensions)
- [Manual desktop package conversion and MakePri](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-manual-conversion)
