#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACKAGE_JSON_PATH = ROOT / "package.json"
TAURI_CARGO_PATH = ROOT / "src-tauri" / "Cargo.toml"
TAURI_CONFIG_PATH = ROOT / "src-tauri" / "tauri.conf.json"


def load_version() -> str:
    package_version = json.loads(PACKAGE_JSON_PATH.read_text(encoding="utf-8"))["version"]
    cargo_version = tomllib.loads(TAURI_CARGO_PATH.read_text(encoding="utf-8"))["package"]["version"]
    tauri_version = json.loads(TAURI_CONFIG_PATH.read_text(encoding="utf-8"))["version"]

    versions = {
        "package.json": package_version,
        "src-tauri/Cargo.toml": cargo_version,
        "src-tauri/tauri.conf.json": tauri_version,
    }
    if len(set(versions.values())) != 1:
        mismatch = " ".join(f"{name}={version}" for name, version in versions.items())
        raise SystemExit(f"version mismatch: {mismatch}")

    return package_version


def build_metadata(version: str, tag: str | None, rolling: bool) -> dict[str, str]:
    if rolling:
        if tag:
            raise SystemExit("--rolling cannot be combined with --tag")
        return {
            "RELEASE_VERSION": version,
            "RELEASE_TAG": "master-latest",
            "RELEASE_TITLE": "master-latest",
            "RELEASE_IS_PRERELEASE": "true",
            "RELEASE_ASSET_PREFIX": "alacritty-config-ui",
        }

    expected_tag = f"v{version}"
    if tag and tag != expected_tag:
        raise SystemExit(f"tag mismatch: expected {expected_tag} but got {tag}")

    release_tag = expected_tag
    return {
        "RELEASE_VERSION": version,
        "RELEASE_TAG": release_tag,
        "RELEASE_TITLE": f"Alacritty Config UI {release_tag}",
        "RELEASE_IS_PRERELEASE": "true" if "-" in version else "false",
        "RELEASE_ASSET_PREFIX": f"alacritty-config-ui-{release_tag}",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", help="Validate the pushed git tag against the manifest version")
    parser.add_argument("--rolling", action="store_true", help="Derive metadata for the master-latest rolling release")
    parser.add_argument("--github-env", help="Append derived metadata to the GitHub Actions env file")
    args = parser.parse_args()

    metadata = build_metadata(load_version(), args.tag, args.rolling)
    lines = [f"{key}={value}" for key, value in metadata.items()]

    if args.github_env:
        env_path = Path(args.github_env)
        env_path.parent.mkdir(parents=True, exist_ok=True)
        with env_path.open("a", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")

    print("\n".join(lines))


if __name__ == "__main__":
    main()
