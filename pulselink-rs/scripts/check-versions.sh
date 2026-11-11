#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

python3 - <<'PY'
import json
import pathlib
import subprocess
import sys
import tomllib

EXPECTED = "0.1.0"
root = pathlib.Path.cwd()
with (root / "Cargo.toml").open("rb") as handle:
    manifest = tomllib.load(handle)

workspace_version = manifest["workspace"]["package"]["version"]
if workspace_version != EXPECTED:
    sys.exit(f"workspace.package.version is {workspace_version}, expected {EXPECTED}")

for name, dependency in manifest["workspace"]["dependencies"].items():
    if name.startswith("pulselink-"):
        version = dependency.get("version") if isinstance(dependency, dict) else None
        if version != EXPECTED:
            sys.exit(f"workspace dependency {name} is {version}, expected {EXPECTED}")

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"], text=True
    )
)
for package in metadata["packages"]:
    if package["name"].startswith("pulselink") and package["version"] != EXPECTED:
        sys.exit(
            f"package {package['name']} is {package['version']}, expected {EXPECTED}"
        )

changelog = (root / "CHANGELOG.md").read_text()
if f"## [{EXPECTED}]" not in changelog:
    sys.exit(f"CHANGELOG.md has no ## [{EXPECTED}] section")

print(f"versions ok: {EXPECTED}")
PY
