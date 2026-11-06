"""Source-level regressions only; these do not install or publish anything."""
import importlib.util
import json
import sys
sys.dont_write_bytecode = True
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("identity_check", Path(__file__).with_name("check-identity.py"))
check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check)


class IdentityTests(unittest.TestCase):
    def setUp(self):
        self.identity = json.loads((check.ROOT / "packaging/pagic-identity.json").read_text())
        self.files = check.sources()

    def test_platform_sources_agree(self):
        self.assertEqual(check.validate(self.identity, self.files), [])

    def test_macos_pdf_association_requires_document_icon(self):
        name = "packaging/macos/Info.plist.in"
        self.files[name] = self.files[name].replace(
            "<string>pagic-document</string>", "<string>pagic</string>"
        )
        self.assertIn(
            "macOS PDF association must reference the Pagic document icon",
            check.validate(self.identity, self.files),
        )

    def test_recto_upgrade_identity_is_rejected(self):
        name = "packaging/windows/pagic.wxs"
        self.files[name] = self.files[name].replace(self.identity["windows"]["msi_upgrade_code"], "FF3C761B-F699-5787-9C4D-13CBDF33A529")
        self.assertTrue(check.validate(self.identity, self.files))

    def test_recto_registry_ownership_is_rejected(self):
        name = "packaging/windows/install-policy.wxi"
        self.files[name] = self.files[name].replace("Key=\"Software\\pagic\"", "Key=\"Software\\recto\"")
        self.assertTrue(check.validate(self.identity, self.files))

    def test_macos_release_gate_does_not_open_shared_release_gate(self):
        self.assertEqual(check.release_blockers(self.identity, macos=True), [])
        self.assertFalse(self.identity["release_packaging_enabled"])
        self.assertTrue(check.release_blockers(self.identity))

    def test_packaging_is_not_activated_without_artwork_and_native_review(self):
        self.assertFalse(self.identity["release_packaging_enabled"])
        self.assertTrue(check.release_blockers(self.identity))


if __name__ == "__main__":
    unittest.main()
