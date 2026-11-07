# Vendored Pagic libraries (former craft-libs)

Source repository. https://github.com/arafatic/craft-libs

Pinned Git commit. `e11f9d9aebea5f6b8e74643c1ea1ed172ba5f545`

`vendor/pagic-libs/` contains the exact former `arafatic/craft-libs` source subset
pinned above, previously stored locally at `vendor/craft-libs/`. This is only a
local vendor-directory/repository-identity rename. Internal `craft-*` package
names, source bytes, licenses, notices, copyright and upstream provenance are
unchanged; no application or packaging identity changes are included.

Pagic builds these crates locally so release builds do not need the source repository.
The source URL is attribution only.

The nine runtime crates are docpath, print, ui, menu, keymap, quit, update,
fonts-build and i18n. Atomic, settings and the fonts-build consumer fixture are
retained for upstream tests. Their source, crate manifests and test data were
copied byte for byte from the pinned Git tree.

The local root Cargo.toml retains the original package metadata, required workspace
dependencies and lints. It omits unrelated workspace members and upstream root
patches. Pagic's existing root patches select egui and winit. The original manifest
is preserved as Cargo.toml.upstream. UPSTREAM-SHA256.json records the copied bytes.

LICENSE-MIT, LICENSE-APACHE, NOTICE and ATTRIBUTION.md are the original upstream
files. Some entries in the upstream notice describe components outside this subset.
The existing release notice collector includes these files from vendor/.

Do not run cargo fmt over this copy. Pagic must preserve the pinned upstream source.
