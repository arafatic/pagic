#!/usr/bin/env bash
# Regenerate the app icons from assets/app-icon/pagic-app-icon.svg and the Windows PDF file icon
# from assets/app-icon/pagic-document.svg. Both are master vectors.
#
# Needs: resvg (brew install resvg / cargo install resvg) and python3 (stdlib only, for the .ico).
# On macOS, iconutil also writes the .icns. The outputs are committed, so building and packaging
# never need these tools. Record and validate generated hashes with
# `python3 packaging/check-icons.py --write-manifest` followed by `python3 packaging/check-icons.py`.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/pagic-app-icon.svg"
ID="local.pagic.desktop"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

[[ -s "$SVG" && -s "$DIR/pagic-document.svg" ]] || { echo "Final Pagic master artwork has not been supplied." >&2; exit 1; }

python3 - "$SVG" "$DIR/pagic-document.svg" <<'PYXML'
import sys, xml.etree.ElementTree as ET
for name in sys.argv[1:]:
    root = ET.parse(name).getroot()
    x, y, w, h = map(float, root.attrib['viewBox'].split())
    if w <= 0 or w != h:
        raise SystemExit(name + ': artwork must have a positive square viewBox')
    for node in root.iter():
        if node.tag.rsplit('}', 1)[-1] in {'text', 'image', 'script', 'foreignObject'}:
            raise SystemExit(name + ': expected self-contained vector paths')
        if any(k.rsplit('}', 1)[-1] == 'href' for k in node.attrib):
            raise SystemExit(name + ': external resources are not supported')
PYXML

command -v resvg >/dev/null || { echo "error: resvg not found (brew install resvg)" >&2; exit 1; }

# Preserve the supplied square master, including its background and aspect ratio.
# macOS uses an 824 px body centred on a transparent 1024 canvas.
MAC="$TMP/macos.svg"
{
  echo '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 1024 1024">'
  echo '<image x="100" y="100" width="824" height="824" xlink:href="'"$SVG"'"/>'
  echo '</svg>'
} >"$MAC"

render() {
  local src="$1" size="$2" out="$3"
  resvg -w "$size" -h "$size" "$src" "$out" </dev/null
}

# 1024 px PNG on Apple's grid (also the runtime Dock icon on macOS, see apps/pagic/src/main.rs).
render "$MAC" 1024 "$DIR/pagic-1024.png"

# Linux hicolor theme (full bleed; hicolor/256x256 is also the runtime icon on Windows and Linux).
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$SVG" "$s" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
cp "$SVG" "$DIR/hicolor/scalable/apps/$ID.svg"

# Windows .ico: PNG-compressed entries, 16-256 px. The document icon has its own resource
# in pagic.exe; the app icon remains the first/default icon.
for name in pagic pagic-document; do
  ICO_PNGS=()
  for s in 16 20 24 32 40 48 64 128 256; do
    source="$SVG"
    if [[ "$name" = pagic-document ]]; then source="$DIR/pagic-document.svg"; fi
    render "$source" "$s" "$TMP/$name-$s.png"
    ICO_PNGS+=("$TMP/$name-$s.png")
  done
  python3 - "$DIR/$name.ico" "${ICO_PNGS[@]}" <<'PY'
import struct, sys
out, pngs = sys.argv[1], sys.argv[2:]
blobs = [open(p, "rb").read() for p in pngs]
head = struct.pack("<HHH", 0, 1, len(blobs))
entries, data, offset = b"", b"", 6 + 16 * len(blobs)
for b in blobs:
    w, h = struct.unpack(">II", b[16:24])  # IHDR
    entries += struct.pack("<BBBBHHII", w % 256, h % 256, 0, 0, 1, 32, len(b), offset)
    data += b
    offset += len(b)
open(out, "wb").write(head + entries + data)
PY
done

# macOS .icns.
if command -v iconutil >/dev/null; then
  for name in pagic pagic-document; do
    source="$MAC"
    if [[ "$name" = pagic-document ]]; then source="$DIR/pagic-document.svg"; fi
    SET="$TMP/$name.iconset"
    mkdir -p "$SET"
    for s in 16 32 128 256 512; do
      render "$source" "$s" "$SET/icon_${s}x${s}.png"
      render "$source" $((s * 2)) "$SET/icon_${s}x${s}@2x.png"
    done
    iconutil -c icns -o "$DIR/$name.icns" "$SET"
  done
else
  echo "warning: iconutil not found (macOS only); macOS icons not regenerated" >&2
fi
# MSIX image resources only: no manifest, package, identity or signing is created.
MSIX="$ROOT/packaging/windows/msix/Assets"
mkdir -p "$MSIX"
for scale in 100 200 400; do
  render "$SVG" $((44 * scale / 100)) "$MSIX/Square44x44Logo.scale-$scale.png"
  render "$SVG" $((150 * scale / 100)) "$MSIX/Square150x150Logo.scale-$scale.png"
  render "$SVG" $((50 * scale / 100)) "$MSIX/StoreLogo.scale-$scale.png"
done
for s in 16 20 24 30 32 36 40 48 60 64 72 80 96 256; do
  base="$MSIX/Square44x44Logo.targetsize-$s"
  render "$SVG" "$s" "$base.png"
  # Keep the approved rounded background and transparent corners for every theme.
  # Qualifiers suppress an additional OS plate; they do not alter the supplied art.
  cp "$base.png" "${base}_altform-unplated.png"
  cp "$base.png" "${base}_altform-lightunplated.png"
done
echo "desktop icons written to $DIR; MSIX image resources written to $MSIX"
