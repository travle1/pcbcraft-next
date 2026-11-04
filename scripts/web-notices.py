#!/usr/bin/env python3
"""Bundle notices from the actual locked browser dependency graph, without source files."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "target/web-licenses"
SUPPLEMENTS = {
    (name, "0.36.2"): ["egui-APACHE.txt"]
    for name in ["eframe", "egui", "egui_extras", "egui_glow", "ecolor", "emath", "epaint", "epaint_default_fonts"]
} | {
    ("accesskit", "0.24.1"): ["accesskit-APACHE.txt", "accesskit-CHROMIUM.txt"],
    ("profiling", "1.0.18"): ["profiling-APACHE.txt"],
    ("enum-map", "2.7.3"): ["enum-map-APACHE.txt"],
    ("enum-map-derive", "0.17.0"): ["enum-map-derive-APACHE.txt"],
}


def main():
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--offline", "--locked", "--format-version", "1",
        "--no-default-features", "--features", "web", "--filter-platform", "wasm32-unknown-unknown",
    ], cwd=ROOT))
    selected = {node["id"] for node in metadata["resolve"]["nodes"]}
    packages = sorted((p for p in metadata["packages"] if p["id"] in selected),
                      key=lambda p: (p["name"], p["version"]))
    forbidden = {"rfd", "tempfile", "solidcraft-engine", "solidcraft-ui-egui"}
    if any(p["name"] in forbidden or p["name"].startswith("truck") for p in packages):
        raise RuntimeError("Unexpected native/application dependency in the browser graph")
    OUT.mkdir(parents=True, exist_ok=True)
    provenance = ROOT / "web/notices/provenance.json"
    for record in json.loads(provenance.read_text()):
        data = (provenance.parent / record["file"]).read_bytes()
        if hashlib.sha256(data).hexdigest() != record["sha256"]:
            raise RuntimeError(f'Notice hash mismatch: {record["file"]}')
    shutil.copyfile(provenance, OUT / "supplements.json")
    for name in ["LICENSE-MIT", "LICENSE-APACHE", "ATTRIBUTION.md"]:
        shutil.copyfile(ROOT / name, OUT / name)
    family = OUT / "family"
    family.mkdir(exist_ok=True)
    for name in ["README.md", "provenance.json", "IBM-PLEX-OFL.txt", "SOLIDCRAFT-NOTICE",
                 "SOLIDCRAFT-LICENSE-MIT", "SOLIDCRAFT-LICENSE-APACHE"]:
        shutil.copyfile(ROOT / "assets/family" / name, family / name)
    register = []
    for package in packages:
        if package["source"] is None:
            continue  # Own notices were copied above; never scan the workspace for source.
        base = Path(package["manifest_path"]).parent
        name = f'{package["name"]}-{package["version"]}'
        destination = OUT / "crates" / name
        destination.mkdir(parents=True, exist_ok=True)
        notices = []
        explicit = package.get("license_file")
        for path in sorted(base.rglob("*")):
            if path.is_symlink() or not path.is_file():
                continue
            relative = path.relative_to(base)
            if not (path.name.lower().startswith(("license", "licence", "copying", "copyright", "notice", "ofl", "ufl"))
                    or "license" in path.name.lower() or path.name == "Hack-Regular.txt" or str(relative) == explicit):
                continue
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
            notices.append(str(relative))
        for supplement in SUPPLEMENTS.get((package["name"], package["version"]), []):
            shutil.copyfile(ROOT / "web/notices" / supplement, destination / supplement)
            notices.append(supplement)
        if not notices:
            raise RuntimeError(f"No license/notice files found for {name}; qualify before distributing")
        register.append({key: package[key] for key in ["name", "version", "license", "repository", "authors", "source"]}
                        | {"notices": notices})
    (OUT / "dependencies.json").write_text(json.dumps(register, indent=2) + "\n")
    (OUT / "README.txt").write_text(
        "PCBcraft browser distribution\nOriginal PCBcraft: MIT OR Apache-2.0.\n"
        "Family artwork/controls and IBM Plex: see family/.\n"
        "Rust dependency versions, declared licenses, authors and retained notices: dependencies.json and crates/.\n"
        "Alternative GPL terms in a dual-license expression are not selected; use Apache-2.0 where offered.\n"
        "epaint_default_fonts includes independently licensed fonts; retain all bundled font notices.\n")
    print(f"Browser notices: {len(register)} locked dependencies")


if __name__ == "__main__":
    main()
