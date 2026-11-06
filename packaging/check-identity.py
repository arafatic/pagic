#!/usr/bin/env python3
"""Check source identities; release assembly stays disabled pending native/artwork review."""
import argparse
import json
import plistlib
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def sources(root=ROOT):
    return {name: (root / name).read_text(encoding="utf-8") for name in [
        "packaging/windows/setup.iss", "packaging/windows/pagic.iss",
        "packaging/windows/pagic.wxs", "packaging/windows/install-policy.wxi",
        "packaging/windows/install-policy.json", "packaging/macos/Info.plist.in",
        "packaging/linux/local.pagic.desktop.desktop",
        "packaging/linux/local.pagic.desktop.metainfo.xml.in",
    ]}


def validate(identity, files):
    errors = []
    windows = identity["windows"]
    for name in ["setup.iss", "pagic.iss"]:
        text = files["packaging/windows/" + name]
        if "AppId={{" + windows["inno_app_id"] + "}" not in text:
            errors.append(name + ": wrong Inno uninstall identity")
        if '#define MyAppExeName "' + windows["executable"] + '"' not in text:
            errors.append(name + ": wrong executable identity")
    wix = ET.fromstring(files["packaging/windows/pagic.wxs"])
    package = next(e for e in wix if e.tag.endswith("Package"))
    if package.attrib["UpgradeCode"] != windows["msi_upgrade_code"]:
        errors.append("wrong MSI upgrade identity")
    policy = json.loads(files["packaging/windows/install-policy.json"])
    if policy["upgradeCode"] != windows["msi_upgrade_code"] or policy["id"] != identity["data_slug"]:
        errors.append("Windows policy disagrees with identity contract")
    for name in ["pagic.wxs", "install-policy.wxi"]:
        xml = ET.fromstring(files["packaging/windows/" + name])
        for node in xml.iter():
            key = node.attrib.get("Key", "")
            if re.search(r"recto|printcraft", key, re.I):
                errors.append(name + ": foreign registry ownership: " + key)
    # Original Recto product/component GUIDs cannot be reused by Pagic.
    old = ["C6F89F3A-81B5-4D19-9B21-7A3EBE87A18F", "FF3C761B-F699-5787-9C4D-13CBDF33A529",
           "281A468D-3403-5800-AE33-BEB806F8FC9B", "4C1537FA-D972-5473-A3F8-9CA37CBFE31C",
           "BA054791-80CB-52A6-AD8F-D3376D1D998E", "D9C23853-98C4-5731-8813-3A4B5788B4FB",
           "DEF3F349-4556-55EA-9FE2-763814468153"]
    for name, text in files.items():
        if name.startswith("packaging/windows/") and any(guid in text for guid in old):
            errors.append(name + ": reused Recto installer/component GUID")
    for entry in policy["registry"]:
        if re.search(r"recto|printcraft", entry[1], re.I):
            errors.append("foreign Windows policy registry ownership")
    registry = [e.attrib for e in wix.iter() if e.tag.endswith("RegistryValue")]
    for row in policy["registry"]:
        matches = [a for a in registry if a.get("Root") == "HKMU" and a.get("Key") == row[1] and a.get("Name", "") == row[2] and a.get("Value", "") == row[3]]
        if len(matches) != 1:
            errors.append("Windows policy registry row differs from WiX: " + row[1])
    if windows["progid"] not in files["packaging/windows/pagic.wxs"]:
        errors.append("MSI ProgID disagrees with contract")
    mac = plistlib.loads(files["packaging/macos/Info.plist.in"].encode())
    if mac["CFBundleIdentifier"] != identity["application_id"] or mac["CFBundleExecutable"] != "pagic":
        errors.append("macOS bundle identity disagrees with contract")
    desktop = dict(line.split("=", 1) for line in files["packaging/linux/local.pagic.desktop.desktop"].splitlines() if "=" in line)
    if desktop["StartupWMClass"] != identity["application_id"] or desktop["Icon"] != identity["application_id"] or desktop["Exec"] != "pagic %U":
        errors.append("Linux launcher identity disagrees with contract")
    appstream = ET.fromstring(files["packaging/linux/local.pagic.desktop.metainfo.xml.in"])
    if appstream.findtext("id") != identity["application_id"] or appstream.findtext("launchable") != identity["application_id"] + ".desktop":
        errors.append("AppStream identity disagrees with contract")
    return errors


def release_blockers(identity, root=ROOT):
    blockers = []
    if identity.get("release_packaging_enabled") is not True:
        blockers.append("Pagic release/installer packaging is disabled pending native ownership review")
    for asset in ["pagic.svg", "pagic-document.svg", "pagic-1024.png", "pagic.ico", "pagic-document.ico", "pagic.icns"]:
        if not (root / "assets/app-icon" / asset).is_file():
            blockers.append("final artwork missing: " + asset)
    for notice in ["NOTICE", "LICENSE-APACHE"]:
        if not (root / notice).is_file():
            blockers.append("unresolved packaging notice input: " + notice)
    return blockers


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--require-release-ready", action="store_true")
    args = parser.parse_args()
    identity = json.loads((ROOT / "packaging/pagic-identity.json").read_text(encoding="utf-8"))
    errors = validate(identity, sources())
    if args.require_release_ready:
        errors += release_blockers(identity)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("PASS: Pagic source identities agree; native installer validation is separate")
    return 0


if __name__ == "__main__":
    sys.exit(main())
