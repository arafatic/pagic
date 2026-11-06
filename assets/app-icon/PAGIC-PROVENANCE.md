# Pagic supplied artwork provenance

The project owner supplied `pagic.svg` and `pagic-document.svg` and approved their original
artwork for Stage 3 local integration. Both files were present before integration and are
preserved byte-for-byte. They are identical; the document icon intentionally uses the same
approved mark. No design, recoloring, shape changes or alternate theme artwork was invented.

SHA-256 of each original SVG:
`b30946388d16aec1adc1468167b7a6e65420f62fad0f094d46d87c18942f697c`

Self-contained vector paths, circle and opaque `#1e1e1e` background, with `#f5f5f7` foreground;
viewBox `0 0 330.68 330.68`. No text, fonts, raster images or external resources.

## Approved rounded application icon update

The owner subsequently supplied `pagic-app-icon.svg` for application icons. Its SHA-256 is
`d03d1618f03c362a1c056e3b0ac471b6ca680e55f994f52733da05eadf9d72a5`.
It is preserved byte-for-byte: self-contained paths, viewBox 1032.694 by 1032.694, transparent
corners around its supplied rounded dark background and light Pagic mark. No masking,
redesign, recoloring, cropping or nonuniform scaling is applied. Its embedded title
`Untitled-1` is retained as supplied; this metadata is not the application's display name.

Application derivatives now use this new master. `pagic.svg` remains the unchanged original
brand logo; `pagic-document.svg` and `pagic-document.ico` remain byte-identical to their
previous versions and retain their document-specific role. The two original SVGs above do
not become copies of the rounded icon. This update adds no new dependencies or identities.

Generated derivatives are recorded in `pagic-assets.sha256.json`. Renderer: the already-locked
resvg 0.45.1 library, through a temporary size-only helper (no new project dependency), plus
macOS 26.3.1 iconutil for ICNS. Future regeneration can use the normal resvg CLI with
`packaging/icons.sh`. Scaling is uniform. macOS adds the previously agreed transparent inset.
MSIX theme variants keep the original background and colors; they are not redesigned glyphs.

## Artwork Ownership and Rights Confirmation

The project owner and author originally created, designed, and owns all rights to the Pagic
artwork files listed above (`pagic.svg`, `pagic-app-icon.svg`, `pagic-document.svg`).
Complete usage, reproduction, and distribution permissions—including for desktop application
packages, installers, documentation, marketing materials, and Microsoft Store distribution—are
formally documented and affirmed in `PAGIC-ARTWORK-RIGHTS.md`. The artwork does not intentionally
incorporate third-party logos or trademarked marks. The artwork-rights blocker for distribution
is resolved.

Unused Recto artwork was removed from the current tree after the reference audit; its
original bytes remain in Git history. LICENSE-recto.txt remains unchanged. Lucide icons/licenses, bundled
font binaries/OFL notices and vendored resources are untouched. No existing notice is removed.
