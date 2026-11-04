#!/usr/bin/env python3
"""Packaging regression tests; synthetic PE/ZIP data are test inputs, never deliverables."""
import argparse
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
import zipfile

SPEC = importlib.util.spec_from_file_location('pagic_msix', Path(__file__).with_name('msix.py'))
msix = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(msix)


def sample_pdf(path):
    # Helvetica is a PDF standard font; calculate real byte offsets and stream length.
    stream = b'BT /F1 20 Tf 36 100 Td (Pagic MSIX PDF opening check) Tj ET\n'
    objects = [b'<< /Type /Catalog /Pages 2 0 R >>',
               b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
               b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 500 200] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>',
               b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
               b'<< /Length ' + str(len(stream)).encode() + b' >>\nstream\n' + stream + b'endstream']
    output = b'%PDF-1.4\n'
    offsets = [0]
    for number, obj in enumerate(objects, 1):
        offsets.append(len(output))
        output += str(number).encode() + b' 0 obj\n' + obj + b'\nendobj\n'
    xref = len(output)
    output += b'xref\n0 6\n0000000000 65535 f \n'
    output += b''.join(f'{offset:010} 00000 n \n'.encode() for offset in offsets[1:])
    output += b'trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n' + str(xref).encode() + b'\n%%EOF\n'
    path.write_bytes(output)


def fake_pe(path, subsystem=2, machine=0x8664):
    data = bytearray(256)
    data[:2] = b'MZ'
    struct.pack_into('<I', data, 0x3c, 64)
    data[64:68] = b'PE\0\0'
    struct.pack_into('<H', data, 68, machine)
    struct.pack_into('<H', data, 88, 0x20b)
    struct.pack_into('<H', data, 156, subsystem)
    path.write_bytes(data)


class PackagingTests(unittest.TestCase):
    def test_store_version_ordering_and_bounds(self):
        self.assertEqual(msix.version('0.14.0'), '1.14.0.0')
        self.assertEqual(msix.version('1.0.0'), '2.0.0.0')
        self.assertEqual(msix.version('65534.65535.65535'), '65535.65535.65535.0')
        for invalid in ('0.14.0-dev', '0.14.0+build', '65535.0.0', '1.65536.0', '01.2.3', '1.2'):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                msix.version(invalid)

    @patch.object(msix, 'workspace_version', return_value='0.14.0')
    def test_identity_and_quoted_pdf_activation(self, _):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / 'AppxManifest.xml'
            rendered = msix.render_manifest('0.14.0')
            manifest.write_text(rendered)
            msix.validate_manifest(manifest)
            for old, new in [('Arafatic.Pagic', 'local.recto.desktop'),
                             ('CN=8E27432A-B6BC-4F20-8810-A2256A3F60AF', 'CN=Wrong'),
                             ('1.14.0.0', '0.14.0.0'), ('x64', 'arm64'),
                             ('&quot;%1&quot;', '%1'), ('runFullTrust', 'internetClient')]:
                with self.subTest(new=new), self.assertRaises(ValueError):
                    manifest.write_text(rendered.replace(old, new))
                    msix.validate_manifest(manifest)

    @patch.object(msix, 'workspace_version', return_value='0.14.0')
    def test_cli_alias_requires_console_and_correct_payload(self, _):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / 'AppxManifest.xml'
            rendered = msix.render_manifest('0.14.0')
            manifest.write_text(rendered)
            msix.validate_manifest(manifest)
            for old, new in [('Executable="pagic-cli.exe"', 'Executable="pagic.exe"'),
                             ('Alias="pagic-cli.exe"', 'Alias="recto-cli.exe"'),
                             ('Subsystem="console"', 'Subsystem="windows"'),
                             ('windows.appExecutionAlias', 'windows.startupTask')]:
                with self.subTest(new=new), self.assertRaisesRegex(ValueError, 'CLI|alias'):
                    manifest.write_text(rendered.replace(old, new))
                    msix.validate_manifest(manifest)

    @patch.object(msix, 'workspace_version', return_value='0.14.0')
    def test_console_alias_requires_application_multiple_instances(self, _):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / 'AppxManifest.xml'
            rendered = msix.render_manifest('0.14.0')
            manifest.write_text(rendered)
            msix.validate_manifest(manifest)
            required = 'uap10:SupportsMultipleInstances="true"'
            self.assertIn(required, rendered)
            for replacement in ('', 'uap10:SupportsMultipleInstances="false"',
                                'SupportsMultipleInstances="true"'):
                with self.subTest(replacement=replacement):
                    manifest.write_text(rendered.replace(required, replacement))
                    with self.assertRaisesRegex(ValueError, 'Application uap10:SupportsMultipleInstances'):
                        msix.validate_manifest(manifest)

    def test_wrong_executable_architecture_and_subsystem_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            exe = Path(directory) / 'test.exe'
            fake_pe(exe)
            msix.pe(exe, 2)
            with self.assertRaises(ValueError):
                msix.pe(exe, 3)
            fake_pe(exe, machine=0xaa64)
            with self.assertRaises(ValueError):
                msix.pe(exe, 2)

    def test_packaged_bytes_and_crc_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory) / 'stage'
            stage.mkdir()
            (stage / 'payload.txt').write_text('source notice bytes')
            package = Path(directory) / 'synthetic-test-only.zip'
            with zipfile.ZipFile(package, 'w') as archive:
                archive.write(stage / 'payload.txt', 'payload.txt')
                archive.writestr('AppxBlockMap.xml', '<BlockMap HashMethod="http://www.w3.org/2001/04/xmlenc#sha256"/>')
                archive.writestr('[Content_Types].xml', '<Types/>')
            # This unit test covers byte verification, not native SDK package validation.
            with patch.object(msix, 'validate_stage'):
                self.assertEqual(msix.verify_archive(package, stage)['files_verified'], 1)
                (stage / 'payload.txt').write_text('tampered attribution')
                with self.assertRaises(ValueError):
                    msix.verify_archive(package, stage)

    def test_makeappx_uri_encoded_license_paths_preserve_notice_bytes(self):
        # Reproduces the native Windows MakeAppx ash-license member spelling.
        relative = 'Legal/dependencies/ash-0.38.0+1.3.281-e7f9f4e9/LICENSE-APACHE'
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory) / 'stage'
            notice = stage / relative
            notice.parent.mkdir(parents=True)
            notice.write_bytes(b'Copyright fixture\r\nComplete license terms\n')
            # Also preserve a literal percent sequence: URI decoding must happen only once.
            literal = stage / 'Legal/literal%2B+notice.txt'
            literal.write_bytes(b'Original literal-percent notice\n')
            package = Path(directory) / 'makeappx-test-only.zip'
            with zipfile.ZipFile(package, 'w') as archive:
                archive.write(notice, relative.replace('+', '%2B'))
                archive.write(literal, 'Legal/literal%252B+notice.txt')
                archive.writestr('AppxBlockMap.xml', '<BlockMap HashMethod="http://www.w3.org/2001/04/xmlenc#sha256"/>')
                archive.writestr('[Content_Types].xml', '<Types/>')
            with patch.object(msix, 'validate_stage'):
                self.assertEqual(msix.verify_archive(package, stage)['files_verified'], 2)
                notice.write_bytes(b'Truncated or altered license')
                with self.assertRaisesRegex(ValueError, 'content mismatch'):
                    msix.verify_archive(package, stage)

    def test_uri_decoding_rejects_ambiguous_or_escaping_members(self):
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory) / 'stage'
            stage.mkdir()
            for names in (('Legal/a+b', 'Legal/a%2Bb'), ('%2E%2E/outside',)):
                with self.subTest(names=names):
                    package = Path(directory) / 'invalid-test-only.zip'
                    with zipfile.ZipFile(package, 'w') as archive:
                        for name in names:
                            archive.writestr(name, b'fixture')
                    with patch.object(msix, 'validate_stage'), self.assertRaises(ValueError):
                        msix.verify_archive(package, stage)

    def test_unsafe_package_paths_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory) / 'stage'
            stage.mkdir()
            package = Path(directory) / 'synthetic-test-only.zip'
            with zipfile.ZipFile(package, 'w') as archive:
                archive.writestr('../outside', 'no')
            with patch.object(msix, 'validate_stage'), self.assertRaises(ValueError):
                msix.verify_archive(package, stage)

    def test_document_artwork_is_existing_ico_png(self):
        png = msix.document_png()
        self.assertEqual(struct.unpack('>II', png[16:24]), (256, 256))

    def test_legal_inventory_retains_complete_notice_bytes_and_reports_gaps(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            crate = folder / 'crate'
            crate.mkdir()
            (crate / 'LICENSE').write_bytes(b'Copyright fixture\nAll original terms\r\n')
            stage = folder / 'stage'
            stage.mkdir()
            pkg = dict(id='fixture1', name='fixture', version='1.0.0', source='registry+test',
                       manifest_path=str(crate / 'Cargo.toml'), license='MIT')
            report = msix.legal(stage, {'packages': [pkg]})
            notice = report['dependencies'][0]['notices'][0]
            self.assertEqual((stage / notice['path']).read_bytes(), (crate / 'LICENSE').read_bytes())
            self.assertFalse(report['missing_dependency_notice_files'])
            # A missing notice must be disclosed; an SPDX label alone isn't its text.
            empty = folder / 'empty'
            empty.mkdir()
            stage2 = folder / 'stage2'
            stage2.mkdir()
            pkg['manifest_path'] = str(empty / 'Cargo.toml')
            self.assertEqual(msix.legal(stage2, {'packages': [pkg]})['missing_dependency_notice_files'], ['fixture1'])

    def test_font_override_notices_are_verified_byte_for_byte(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            inputs = folder / 'inputs'
            family = inputs / 'fonts/test'
            family.mkdir(parents=True)
            (family / 'font.ttf').write_bytes(b'synthetic font fixture; not a delivered font')
            (family / 'OFL.txt').write_bytes(b'original font copyright and OFL fixture\n')
            row = 'Test | Regular | fonts/test/font.ttf | Latn | OFL-1.1 | fonts/test/OFL.txt\n'
            for manifest in ('manifest.txt', 'simplified-chinese-manifest.txt'):
                (inputs / 'fonts' / manifest).write_text(row)
            stage = folder / 'stage'
            notices = stage / 'Legal/fonts'
            notices.mkdir(parents=True)
            msix.copy(family / 'OFL.txt', notices / 'OFL-test.txt')
            for src in (msix.ROOT / 'assets/fonts').glob('OFL-*.txt'):
                msix.copy(src, notices / src.name)
            with patch.dict(msix.os.environ, {'PAGIC_FONTS_DIR': str(inputs), 'RECTO_FONTS_DIR': 'must-not-be-used'}):
                report = msix.font_inventory(stage)
                self.assertEqual(len(report['font_inputs']), 2)
                (notices / 'OFL-test.txt').write_text('truncated copyright')
                with self.assertRaises(ValueError):
                    msix.font_inventory(stage)

    def test_pinned_git_workspace_notices_are_included(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            repo = folder / 'checkout'
            crate = repo / 'crates/fixture'
            crate.mkdir(parents=True)
            (repo / '.git').mkdir()
            (repo / 'LICENSE').write_bytes(b'Pinned workspace license fixture\n')
            (repo / 'NOTICE').write_bytes(b'Required workspace attribution fixture\n')
            stage = folder / 'stage'
            stage.mkdir()
            pkg = dict(id='git-fixture1', name='fixture', version='1.0.0', source='git+fixture#pin',
                       manifest_path=str(crate / 'Cargo.toml'), license='MIT')
            report = msix.legal(stage, {'packages': [pkg]})
            self.assertFalse(report['missing_dependency_notice_files'])
            self.assertEqual(len(report['dependencies'][0]['notices']), 2)

    def test_verified_upstream_notices_and_reuse_licenses_directory(self):
        msix.verify_upstream_notices()
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            reuse = folder / 'reuse-crate'
            (reuse / 'LICENSES').mkdir(parents=True)
            (reuse / 'LICENSES/MIT.txt').write_bytes(b'MIT license fixture\n')
            (reuse / 'README.md').write_text('<!-- SPDX-FileCopyrightText: 2025 Fixture Author -->\n')
            upstream = folder / 'ocrs-crate'
            upstream.mkdir()
            (upstream / '.cargo_vcs_info.json').write_text(json.dumps({
                'git': {'sha1': 'e5f1c5205326804637368422fe420aa1b9769676'}}))
            stage = folder / 'stage'
            stage.mkdir()
            packages = [
                dict(id='reuse-1', name='reuse-fixture', version='1.0.0', source='registry+test',
                     manifest_path=str(reuse / 'Cargo.toml'), license='MIT'),
                dict(id='ocrs-1', name='ocrs', version='0.13.1', source='registry+test',
                     manifest_path=str(upstream / 'Cargo.toml'), license='MIT OR Apache-2.0'),
            ]
            report = msix.legal(stage, {'packages': packages})
            self.assertFalse(report['missing_dependency_notice_files'])
            self.assertFalse(report['unverified_dependency_notice_files'])
            by_name = {pkg['name']: pkg for pkg in report['dependencies']}
            self.assertEqual(len(by_name['reuse-fixture']['notices']), 2)
            self.assertEqual(len(by_name['ocrs']['notices']), 2)
            # A mismatched upstream VCS commit must be rejected rather than silently accepted.
            (upstream / '.cargo_vcs_info.json').write_text(json.dumps({'git': {'sha1': '0' * 40}}))
            stage2 = folder / 'stage2'
            stage2.mkdir()
            with self.assertRaisesRegex(ValueError, 'VCS commit mismatch'):
                msix.legal(stage2, {'packages': [packages[1]]})

    def test_validation_workflow_cannot_publish(self):
        text = (msix.ROOT / '.github/workflows/msix.yml').read_text()
        self.assertIn('workflow_dispatch:', text)
        self.assertIn('contents: read', text)
        self.assertIn('branches: [codex/pagic-msix-validation]', text)
        self.assertIn("if: github.event_name == 'push' || inputs.install_smoke", text)
        for forbidden in ('contents: write', 'gh release', 'git push', 'tags:', 'softprops/action-gh-release'):
            self.assertNotIn(forbidden, text)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--sample-pdf', type=Path)
    args, remainder = parser.parse_known_args()
    if args.sample_pdf:
        sample_pdf(args.sample_pdf)
    else:
        unittest.main(argv=[__file__] + remainder)
