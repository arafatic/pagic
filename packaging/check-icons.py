#!/usr/bin/env python3
"""Validate Pagic icon dimensions, alpha, containers, paths and recorded SHA-256 hashes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import xml.etree.ElementTree as ET
import zlib

ROOT = Path(__file__).resolve().parents[1]
ICONS = ROOT / 'assets/app-icon'
MSIX = ROOT / 'packaging/windows/msix/Assets'
MANIFEST = ICONS / 'pagic-assets.sha256.json'
SIZES = (16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256)
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 128, 256)


def png(data, size, mac=False):
    assert data[:8] == b'\x89PNG\r\n\x1a\n', 'not a PNG'
    offset, compressed = 8, b''
    while offset < len(data):
        length = struct.unpack_from('>I', data, offset)[0]
        kind = data[offset + 4:offset + 8]
        body = data[offset + 8:offset + 8 + length]
        crc = struct.unpack_from('>I', data, offset + 8 + length)[0]
        assert zlib.crc32(kind + body) & 0xffffffff == crc, 'PNG CRC mismatch'
        if kind == b'IHDR':
            width, height, depth, color, comp, filt, interlace = struct.unpack('>IIBBBBB', body)
            assert (width, height) == (size, size), (width, height, size)
            assert (depth, color, comp, filt, interlace) == (8, 6, 0, 0, 0), 'expected noninterlaced RGBA8'
        elif kind == b'IDAT':
            compressed += body
        offset += length + 12
    raw = zlib.decompress(compressed)
    stride = size * 4
    assert len(raw) == (stride + 1) * size
    previous, pixels = bytearray(stride), bytearray()
    for y in range(size):
        start = y * (stride + 1)
        mode, row = raw[start], bytearray(raw[start + 1:start + stride + 1])
        assert mode in range(5)
        for i in range(stride):
            left = row[i - 4] if i >= 4 else 0
            up = previous[i]
            corner = previous[i - 4] if i >= 4 else 0
            if mode == 1:
                row[i] = (row[i] + left) & 255
            elif mode == 2:
                row[i] = (row[i] + up) & 255
            elif mode == 3:
                row[i] = (row[i] + (left + up) // 2) & 255
            elif mode == 4:
                p = left + up - corner
                distances = (abs(p - left), abs(p - up), abs(p - corner))
                predictor = (left, up, corner)[distances.index(min(distances))]
                row[i] = (row[i] + predictor) & 255
        pixels.extend(row)
        previous = row
    alpha = pixels[3::4]
    if mac:
        # The unchanged approved master fills an 824/1024 square with a transparent inset.
        lo, hi = size * 100 // 1024, (size * 924 + 1023) // 1024
        assert min(alpha) == 0 and max(alpha) == 255
        assert not any(alpha[:lo * size]) and not any(alpha[hi * size:])
        for y in range(size):
            assert not any(alpha[y * size:y * size + lo])
            assert not any(alpha[y * size + hi:(y + 1) * size])
        assert alpha[(size // 2) * size + size // 2] == 255
    else:
        assert min(alpha) == max(alpha) == 255, 'approved full-background artwork must remain opaque'
    return bytes(pixels)


def validate():
    files = []
    for name in ('pagic.svg', 'pagic-document.svg'):
        path = ICONS / name
        tree = ET.parse(path).getroot()
        _, _, w, h = map(float, tree.attrib['viewBox'].split())
        assert w == h and w > 0
        assert not any(n.tag.rsplit('}', 1)[-1] in {'text', 'image', 'script', 'foreignObject'} for n in tree.iter())
        files.append(path)
    scalable = ICONS / 'hicolor/scalable/apps/local.pagic.desktop.svg'
    assert scalable.read_bytes() == (ICONS / 'pagic.svg').read_bytes()
    files.append(scalable)
    for size in (16, 24, 32, 48, 64, 128, 256, 512):
        path = ICONS / ('hicolor/{0}x{0}/apps/local.pagic.desktop.png'.format(size))
        png(path.read_bytes(), size)
        files.append(path)
    path = ICONS / 'pagic-1024.png'
    png(path.read_bytes(), 1024, mac=True)
    files.append(path)
    for name in ('pagic.ico', 'pagic-document.ico'):
        path = ICONS / name
        data = path.read_bytes()
        assert struct.unpack_from('<HHH', data) == (0, 1, len(ICO_SIZES))
        offset = 6 + 16 * len(ICO_SIZES)
        for index, size in enumerate(ICO_SIZES):
            w, h, colors, reserved, planes, bits, length, start = struct.unpack_from('<BBBBHHII', data, 6 + index * 16)
            assert (w or 256, h or 256, colors, reserved, planes, bits, start) == (size, size, 0, 0, 1, 32, offset)
            png(data[start:start + length], size)
            offset += length
        assert offset == len(data)
        files.append(path)
    icns = ICONS / 'pagic.icns'
    data = icns.read_bytes()
    assert data[:4] == b'icns' and struct.unpack_from('>I', data, 4)[0] == len(data)
    offset, types = 8, set()
    while offset < len(data):
        kind, length = struct.unpack_from('>4sI', data, offset)
        assert length >= 8 and offset + length <= len(data)
        types.add(kind)
        offset += length
    assert offset == len(data)
    assert {b'ic04', b'ic05', b'ic07', b'ic08', b'ic09', b'ic10', b'ic11', b'ic12', b'ic13', b'ic14'} <= types
    if shutil.which('iconutil'):
        with tempfile.TemporaryDirectory(prefix='pagic-icon-check-') as temp:
            out = Path(temp) / 'pagic.iconset'
            subprocess.run(['iconutil', '-c', 'iconset', '-o', str(out), str(icns)], check=True)
            for size in (16, 32, 128, 256, 512):
                for scale in (1, 2):
                    suffix = '@2x' if scale == 2 else ''
                    png((out / ('icon_{0}x{0}{1}.png'.format(size, suffix))).read_bytes(), size * scale, mac=True)
    files.append(icns)
    for scale in (100, 200, 400):
        for name, size in (('Square44x44Logo', 44), ('Square150x150Logo', 150), ('StoreLogo', 50)):
            path = MSIX / '{}.scale-{}.png'.format(name, scale)
            png(path.read_bytes(), size * scale // 100)
            files.append(path)
    for size in SIZES:
        base = MSIX / 'Square44x44Logo.targetsize-{}.png'.format(size)
        for suffix in ('', '_altform-unplated', '_altform-lightunplated'):
            path = base.with_name(base.stem + suffix + '.png')
            png(path.read_bytes(), size)
            assert path.read_bytes() == base.read_bytes(), 'theme files must preserve the approved background'
            files.append(path)
    assert len(files) == 66
    references = {
        'apps/pagic/build.rs': ('pagic.ico', 'pagic-document.ico', 'pagic-1024.png', 'local.pagic.desktop.png'),
        'packaging/macos/Info.plist.in': ('pagic',),
        'packaging/linux/local.pagic.desktop.desktop': ('Icon=local.pagic.desktop',),
        'packaging/windows/setup.iss': ('pagic.ico',),
    }
    for source, values in references.items():
        text = (ROOT / source).read_text()
        assert all(value in text for value in values), source
    return {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-manifest', action='store_true')
    args = parser.parse_args()
    hashes = validate()
    if args.write_manifest:
        MANIFEST.write_text(json.dumps(hashes, indent=2, sort_keys=True) + '\n')
    assert json.loads(MANIFEST.read_text()) == hashes, 'asset manifest is stale'
    print('PASS: 66 source/generated assets; PNG alpha/dimensions/CRC, ICO entries, ICNS representations, references and hashes')
