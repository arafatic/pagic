#!/usr/bin/env python3
"""Stage and independently inspect Pagic MSIX inputs using only the Python standard library.

Microsoft MakePri/MakeAppx perform the native schema/resource/package validation.
This tool never creates a substitute ZIP renamed to .msix.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import string
import struct
import subprocess
import xml.etree.ElementTree as ET
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NS = {'f': 'http://schemas.microsoft.com/appx/manifest/foundation/windows10',
      'uap': 'http://schemas.microsoft.com/appx/manifest/uap/windows10',
      'uap3': 'http://schemas.microsoft.com/appx/manifest/uap/windows10/3',
      'uap10': 'http://schemas.microsoft.com/appx/manifest/uap/windows10/10',
      'rescap': 'http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity():
    return json.loads((HERE / 'identity.json').read_text(encoding='utf-8'))


def workspace_version():
    metadata = json.loads(subprocess.check_output([
        'cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], cwd=ROOT))
    return next(pkg['version'] for pkg in metadata['packages'] if pkg['name'] == 'pagic')


def version(cargo_version):
    """Store version reserves revision=0; offset major so Cargo 0.x has major>=1."""
    require(re.fullmatch(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', cargo_version),
            'MSIX builds require a stable numeric Cargo version; no silently dropped prerelease')
    major, minor, patch = map(int, cargo_version.split('.'))
    require(major < 65535 and minor <= 65535 and patch <= 65535, 'MSIX version component overflow')
    return f'{major + 1}.{minor}.{patch}.0'


def render_manifest(cargo_version):
    values = dict(identity(), version=version(cargo_version))
    # Contract values are reviewed constants, not workflow-supplied XML.
    return string.Template((HERE / 'AppxManifest.xml.in').read_text(encoding='utf-8')).substitute(values)


def pe(path, subsystem):
    data = path.read_bytes()
    require(data[:2] == b'MZ', f'Not a PE executable: {path}')
    offset = struct.unpack_from('<I', data, 0x3c)[0]
    require(data[offset:offset+4] == b'PE\0\0', f'Invalid PE signature: {path}')
    require(struct.unpack_from('<H', data, offset+4)[0] == 0x8664, f'Expected x64 executable: {path}')
    require(struct.unpack_from('<H', data, offset+24)[0] == 0x20b, f'Expected PE32+: {path}')
    require(struct.unpack_from('<H', data, offset+24+68)[0] == subsystem, f'Wrong PE subsystem: {path}')


def document_png():
    data = (ROOT / 'assets/app-icon/pagic-document.ico').read_bytes()
    count = struct.unpack_from('<H', data, 4)[0]
    for index in range(count):
        w, h, _, _, _, _, size, offset = struct.unpack_from('<BBBBHHII', data, 6+index*16)
        payload = data[offset:offset+size]
        if (w or 256, h or 256) == (256, 256) and payload.startswith(b'\x89PNG\r\n\x1a\n'):
            return payload
    raise ValueError('Document ICO must contain its existing 256px PNG representation')


def copy(src, dst):
    require(src.is_file() and src.stat().st_size > 0, f'Missing/empty required input: {src}')
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dst)


def cargo_metadata():
    return json.loads(subprocess.check_output([
        'cargo', 'metadata', '--locked', '--format-version', '1',
        '--filter-platform', identity()['target']], cwd=ROOT))


def legal(stage, metadata):
    """Conservative superset of resolved dependency notices, with explicit gaps for review."""
    for name in ('LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE'):
        copy(ROOT / name, stage / 'Legal' / name)
    for name in ('licenses',):
        shutil.copytree(ROOT / name, stage / 'Legal' / name)
    for name in ('assets/icons/LICENSE-lucide.txt', 'assets/app-icon/LICENSE-recto.txt',
                 'assets/app-icon/PAGIC-PROVENANCE.md', 'docs/brand/LICENSE-brand.txt'):
        copy(ROOT / name, stage / 'Legal' / name)
    records, gaps = [], []
    for pkg in sorted(metadata['packages'], key=lambda p: p['id']):
        if not pkg.get('source'):
            continue
        folder = Path(pkg['manifest_path']).parent.resolve()
        label = f"{pkg['name']}-{pkg['version']}-{hashlib.sha256(pkg['id'].encode()).hexdigest()[:8]}"
        def is_notice(path):
            return path.is_file() and any(word in path.name.lower() for word in
                                         ('license', 'licence', 'notice', 'copying', 'copyright'))
        candidates = {p.resolve(): p.relative_to(folder) for p in folder.rglob('*') if is_notice(p)}
        declared = pkg.get('license_file')
        if declared:
            src = (folder / declared).resolve()
            candidates.setdefault(src, src.relative_to(folder) if src.is_relative_to(folder) else Path('declared-license.txt'))
        if pkg['source'].startswith('git+'):
            # Cargo Git crates can inherit notices from their pinned workspace root.
            # Stop at the Git checkout boundary; never harvest the user's home directory.
            for ancestor in (folder, *folder.parents):
                if (ancestor / '.git').exists():
                    for src in ancestor.iterdir():
                        if is_notice(src):
                            candidates.setdefault(src.resolve(), Path('workspace') / src.name)
                    break
        notices = []
        for src, relative in sorted(candidates.items()):
            dst = stage / 'Legal/dependencies' / label / relative
            copy(src, dst)
            notices.append({'path': dst.relative_to(stage).as_posix(), 'sha256': digest(dst)})
        if not notices:
            gaps.append(pkg['id'])
        records.append({'name': pkg['name'], 'version': pkg['version'], 'source': pkg['source'],
                        'license_expression': pkg.get('license'), 'notices': notices})
    # Vendored/path dependencies can have notices not represented as external Cargo sources.
    for src in (ROOT / 'vendor').rglob('*'):
        if src.is_file() and any(word in src.name.lower() for word in ('license', 'licence', 'notice', 'copying', 'copyright')):
            copy(src, stage / 'Legal/vendor' / src.relative_to(ROOT / 'vendor'))
    report = {'dependencies': records, 'missing_dependency_notice_files': gaps,
              'submission_review_required': [
                  'Review dependency SPDX choices and any missing full notice texts.',
                  'Confirm distribution rights for approved Pagic artwork.',
                  'OCR model bundle is not supplied by this pipeline; OCR requires separately reviewed models.']}
    (stage / 'Legal/dependency-inventory.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def stage_package(destination, binaries, metadata):
    require(not destination.exists(), 'Staging destination must be new (no stale files)')
    cargo_version = workspace_version()
    destination.mkdir(parents=True)
    for name, subsystem in (('pagic.exe', 2), ('pagic-cli.exe', 3)):
        pe(binaries / name, subsystem)
        copy(binaries / name, destination / name)
    shutil.copytree(HERE / 'Assets', destination / 'Assets')
    (destination / 'Assets/DocumentLogo.png').write_bytes(document_png())
    (destination / 'AppxManifest.xml').write_text(render_manifest(cargo_version), encoding='utf-8')
    report = legal(destination, metadata)
    return report


def font_inventory(stage):
    # Match build.rs and the shared PowerShell notice helper, including an empty override.
    raw = os.environ.get('PAGIC_FONTS_DIR', os.environ.get('RECTO_FONTS_DIR'))
    font_root = Path(raw) if raw is not None else ROOT / 'crates/fonts/assets'
    records = []
    expected_notices = {}
    for manifest in ('manifest.txt', 'simplified-chinese-manifest.txt'):
        path = font_root / 'fonts' / manifest
        require(path.is_file(), f'Missing required font manifest: {path}')
        copy(path, stage / 'Legal/fonts/manifests' / manifest)
        for row in path.read_text(encoding='utf-8').splitlines():
            if not row.strip() or row.lstrip().startswith('#'):
                continue
            fields = [field.strip() for field in row.split(' | ')]
            require(len(fields) >= 6 and fields[4] == 'OFL-1.1', 'Invalid font manifest license')
            font, notice = font_root / fields[2], font_root / fields[5]
            require(font.is_file(), f'Missing font input: {font}')
            require(re.fullmatch(r'fonts/[a-zA-Z0-9_-]+/OFL\.txt', fields[5]), 'Unsafe OFL path')
            require(font.parent == notice.parent, 'Font/OFL family mismatch')
            expected_notices['OFL-' + notice.parent.name + '.txt'] = digest(notice)
            records.append({'path': fields[2], 'sha256': digest(font), 'notice': fields[5]})
    for notice in (ROOT / 'assets/fonts').glob('OFL-*.txt'):
        expected_notices[notice.name] = digest(notice)
    for name, sha in expected_notices.items():
        require(digest(stage / 'Legal/fonts' / name) == sha, f'Font notice mismatch: {name}')
    report = {'font_inputs': records, 'notice_hashes': expected_notices}
    (stage / 'Legal/font-input-inventory.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def inventory(stage):
    return {p.relative_to(stage).as_posix(): digest(p) for p in sorted(stage.rglob('*')) if p.is_file()}


def validate_manifest(path):
    root = ET.parse(path).getroot()
    contract = identity()
    ident = root.find('f:Identity', NS)
    for key, value in (('Name', contract['name']), ('Publisher', contract['publisher']),
                       ('ProcessorArchitecture', contract['architecture'])):
        require(ident.get(key) == value, f'MSIX identity mismatch: {key}')
    current = workspace_version()
    require(ident.get('Version') == version(current), 'MSIX/Cargo version mismatch')
    app = root.find('f:Applications/f:Application', NS)
    require(app.get('Executable') == contract['executable'] and app.get('Id') == contract['application_id']
            and app.get('{'+NS['uap10']+'}RuntimeBehavior') == 'packagedClassicApp'
            and app.get('{'+NS['uap10']+'}TrustLevel') == 'mediumIL', 'Incorrect application activation')
    require(root.find('f:Properties/f:DisplayName', NS).text == contract['display_name'], 'Wrong display name')
    require(root.find('f:Properties/f:PublisherDisplayName', NS).text == contract['publisher_display_name'], 'Wrong publisher display name')
    require([n.get('Name') for n in root.find('f:Capabilities', NS)] == ['runFullTrust'], 'Unexpected capabilities')
    family = root.find('f:Dependencies/f:TargetDeviceFamily', NS)
    require(family.attrib == {'Name': 'Windows.Desktop', 'MinVersion': contract['minimum_windows_version'],
                             'MaxVersionTested': contract['maximum_tested_windows_version']}, 'Wrong OS contract')
    require([n.text for n in root.findall('.//uap:FileType', NS)] == ['.pdf'], 'Incorrect file associations')
    require(root.find('.//uap3:Verb', NS).get('Parameters') == '"%1"', 'PDF paths must be quoted')
    require('MigrationProgId' not in path.read_text(encoding='utf-8') and 'Recto' not in path.read_text(encoding='utf-8'), 'Legacy ownership must not be inherited')


def validate_stage(stage):
    validate_manifest(stage / 'AppxManifest.xml')
    for name, subsystem in (('pagic.exe', 2), ('pagic-cli.exe', 3)):
        pe(stage / name, subsystem)
    for name in ('portable.txt', 'pagic.portable', 'PagicData', 'RectoData', '.pagic-control.json'):
        require(not (stage / name).exists(), f'Forbidden package data/portable marker: {name}')
    require((stage / 'resources.pri').stat().st_size > 0, 'Missing native resource index')
    for src in (HERE / 'Assets').glob('*.png'):
        require(digest(src) == digest(stage / 'Assets' / src.name), f'Changed approved icon: {src.name}')
    require((stage / 'Assets/DocumentLogo.png').read_bytes() == document_png(), 'Document artwork changed')
    font_report = json.loads((stage / 'Legal/font-input-inventory.json').read_text(encoding='utf-8'))
    for name, sha in font_report['notice_hashes'].items():
        require(digest(stage / 'Legal/fonts' / name) == sha, f'Font notice hash mismatch: {name}')
    # Verify the legal inventory survived staging byte-for-byte.
    report = json.loads((stage / 'Legal/dependency-inventory.json').read_text(encoding='utf-8'))
    for pkg in report['dependencies']:
        for notice in pkg['notices']:
            require(digest(stage / notice['path']) == notice['sha256'], 'Dependency notice hash mismatch')
    for name in ('LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE'):
        require(digest(ROOT / name) == digest(stage / 'Legal' / name), f'Altered license: {name}')


def verify_archive(package, stage):
    validate_stage(stage)
    with zipfile.ZipFile(package) as archive:
        require(archive.testzip() is None, 'MSIX ZIP CRC failure')
        names = archive.namelist()
        require(len(names) == len(set(n.lower() for n in names)), 'Duplicate package members')
        require(all(not n.startswith(('/', '\\')) and '..' not in n.replace('\\', '/').split('/') for n in names), 'Unsafe package member')
        for relative, expected in inventory(stage).items():
            require(hashlib.sha256(archive.read(relative)).hexdigest() == expected, f'Package content mismatch: {relative}')
        require({'AppxBlockMap.xml', '[Content_Types].xml'} <= set(names), 'Missing MSIX metadata')
        blockmap = ET.fromstring(archive.read('AppxBlockMap.xml'))
        require(blockmap.get('HashMethod') == 'http://www.w3.org/2001/04/xmlenc#sha256', 'MSIX block map must use SHA-256')
    return {'package': package.name, 'sha256': digest(package), 'files_verified': len(inventory(stage)),
            'native_installation_and_certification': 'See Windows runner/manual results; not inferred from archive validation'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    stage = sub.add_parser('stage')
    stage.add_argument('--destination', type=Path, required=True)
    stage.add_argument('--binaries', type=Path, required=True)
    check = sub.add_parser('verify')
    check.add_argument('--stage', type=Path, required=True)
    check.add_argument('--package', type=Path, required=True)
    check.add_argument('--report', type=Path, required=True)
    sub.add_parser('preflight')
    fonts = sub.add_parser('fonts')
    fonts.add_argument('--stage', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'preflight':
        ET.fromstring(render_manifest(workspace_version()))
        require(len(list((HERE / 'Assets').glob('*.png'))) == 51, 'Missing approved MSIX artwork')
        document_png()
        print('PASS: identity, version mapping, XML and 51 MSIX images plus document icon input')
    elif args.command == 'fonts':
        print(json.dumps(font_inventory(args.stage)))
    elif args.command == 'stage':
        report = stage_package(args.destination, args.binaries, cargo_metadata())
        print(f"Staged; {len(report['missing_dependency_notice_files'])} dependency notice gaps require submission review")
    else:
        report = verify_archive(args.package, args.stage)
        args.report.write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report))


if __name__ == '__main__':
    main()
