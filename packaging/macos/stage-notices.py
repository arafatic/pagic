#!/usr/bin/env python3
"""Stage release notices for the native Cargo dependency closure."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

# Reuse the existing verified notice collector without changing the Store pipeline.
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("pagic_notice_collector", ROOT / "packaging/windows/msix/msix.py")
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)


def dependencies(targets):
    selected = {}
    for target in targets:
        data = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", target],
            cwd=ROOT,
        ))
        packages = {pkg["id"]: pkg for pkg in data["packages"]}
        nodes = {node["id"]: node for node in data["resolve"]["nodes"]}
        pending = [key for key in data["workspace_members"] if packages[key]["name"] in {"pagic", "pagic-cli"}]
        if len(pending) != 2:
            raise ValueError("Cargo metadata is missing the macOS release executables")
        seen = set()
        while pending:
            key = pending.pop()
            if key in seen:
                continue
            seen.add(key)
            selected[key] = packages[key]
            pending.extend(dep["pkg"] for dep in nodes[key]["deps"]
                           if any(kind["kind"] in (None, "build") for kind in dep["dep_kinds"]))
    return selected


def stage(destination, targets):
    packages = dependencies(targets)
    report = collector.legal(destination, {"packages": list(packages.values())})
    supplemental = {(item["name"], item["version"]): item for item in
                    json.loads((HERE / "notices/manifest.json").read_text(encoding="utf-8"))}
    by_source = {(p["name"], p["version"], p["source"]): (key, p) for key, p in packages.items()}
    for record in report["dependencies"]:
        key, pkg = by_source[(record["name"], record["version"], record["source"])]
        supplement = supplemental.get((pkg["name"], pkg["version"]))
        if not supplement:
            continue
        directory = Path(pkg["manifest_path"]).parent
        vcs = json.loads((directory / ".cargo_vcs_info.json").read_text(encoding="utf-8"))
        if vcs["git"]["sha1"] != supplement["vcs_sha1"]:
            raise ValueError(f"Notice source revision mismatch for {key}")
        label = f'{pkg["name"]}-{pkg["version"]}-{hashlib.sha256(key.encode()).hexdigest()[:8]}'
        for notice in supplement["notices"]:
            source = HERE / "notices" / notice["file"]
            data = source.read_bytes()
            digest = hashlib.sha256(data).hexdigest()
            blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
            if digest != notice["sha256"] or blob != notice["git_blob_sha1"]:
                raise ValueError(f"Verified upstream notice bytes differ for {source.name}")
            if "status" in supplement:
                # Preserve the maintainer's original SPDX declaration and authorship metadata.
                original = "Cargo.toml.orig" if notice["name"] == "Cargo.toml" else notice["name"]
                if (directory / original).read_bytes() != data:
                    raise ValueError(f"Upstream metadata differs from the pinned crate for {key}")
            relative = Path("Legal/dependencies") / label / "macos-upstream" / notice["name"]
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            record.setdefault("upstream_metadata_files" if "status" in supplement else "notices", []).append({
                "path": relative.as_posix(),
                "sha256": digest,
                "source": f'https://github.com/{supplement["repo"]}/blob/{supplement["vcs_sha1"]}/{notice["upstream_path"]}',
            })
        if "status" in supplement:
            record["upstream_notice_status"] = supplement["status"]
        elif key in report["missing_dependency_notice_files"]:
            report["missing_dependency_notice_files"].remove(key)
        if key in report["unverified_dependency_notice_files"]:
            report["unverified_dependency_notice_files"].remove(key)

    if report["unverified_dependency_notice_files"]:
        missing = "\n".join(report["unverified_dependency_notice_files"])
        raise ValueError(f"Unverified macOS dependency notices:\n{missing}")
    report["targets"] = targets
    report["submission_review_required"] = [
        "Review upstream metadata where no standalone notice exists.",
        "OCR models require separately verified packaging inputs.",
    ]
    (destination / "Legal/dependency-inventory.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f'PASS: verified notice inputs for {len(report["dependencies"])} external dependency versions')


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("targets", nargs="+")
    args = parser.parse_args()
    stage(args.destination, args.targets)
