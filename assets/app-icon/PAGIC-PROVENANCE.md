# Pagic supplied artwork provenance

The project owner supplied `pagic.svg` and `pagic-document.svg` and approved their original
artwork for Stage 3 local integration. Both files were present before integration and are
preserved byte-for-byte. They are identical; the document icon intentionally uses the same
approved mark. No design, recoloring, shape changes or alternate theme artwork was invented.

SHA-256 of each original SVG:
`b30946388d16aec1adc1468167b7a6e65420f62fad0f094d46d87c18942f697c`

Self-contained vector paths, circle and opaque `#1e1e1e` background, with `#f5f5f7` foreground;
viewBox `0 0 330.68 330.68`. No text, fonts, raster images or external resources.

Generated derivatives are recorded in `pagic-assets.sha256.json`. Renderer: the already-locked
resvg 0.45.1 library, through a temporary size-only helper (no new project dependency), plus
macOS 26.3.1 iconutil for ICNS. Future regeneration can use the normal resvg CLI with
`packaging/icons.sh`. Scaling is uniform. macOS adds the previously agreed transparent inset.
MSIX theme variants keep the original background and colors; they are not redesigned glyphs.

The supplied SVGs contain no author, copyright or license metadata. Exact creator/rights and
redistribution/trademark terms were not separately provided. Record those facts before public
release; do not infer them from the archived Recto license or invent an authorship claim.
Approval here is for the requested integration, not a new third-party license notice.

Unused Recto artwork was removed from the current tree after the reference audit; its
original bytes remain in Git history. LICENSE-recto.txt remains unchanged. Lucide icons/licenses, bundled
font binaries/OFL notices and vendored resources are untouched. No existing notice is removed.
