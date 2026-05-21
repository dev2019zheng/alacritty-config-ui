#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACKAGE_JSON_PATH = ROOT / "package.json"
TAURI_CARGO_PATH = ROOT / "src-tauri" / "Cargo.toml"


def load_version() -> str:
    package_version = json.loads(PACKAGE_JSON_PATH.read_text(encoding="utf-8"))["version"]
    cargo_version = tomllib.loads(TAURI_CARGO_PATH.read_text(encoding="utf-8"))["package"]["version"]

    if package_version != cargo_version:
        raise SystemExit(
            f"version mismatch: package.json={package_version} src-tauri/Cargo.toml={cargo_version}"
        )

    return cargo_version


def build_metadata(version: str, tag: str | None) -> dict[str, str]:
    expected_tag = f"v{version}"
    if tag and tag != expected_tag:
        raise SystemExit(f"tag mismatch: expected {expected_tag} but got {tag}")

    release_tag = expected_tag
    return {
        "RELEASE_VERSION": version,
        "RELEASE_TAG": release_tag,
        "RELEASE_TITLE": f"Alacritty Config UI {release_tag}",
        "RELEASE_IS_PRERELEASE": "true" if "-" in version else "false",
        "RELEASE_ASSET_BASE_NAME": f"alacritty-config-ui-{release_tag}-macos",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", help="Validate the pushed git tag against the manifest version")
    parser.add_argument("--github-env", help="Append derived metadata to the GitHub Actions env file")
    args = parser.parse_args()

    metadata = build_metadata(load_version(), args.tag)
    lines = [f"{key}={value}" for key, value in metadata.items()]

    if args.github_env:
        env_path = Path(args.github_env)
        env_path.parent.mkdir(parents=True, exist_ok=True)
        with env_path.open("a", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")

    print("\n".join(lines))


if __name__ == "__main__":
    main()
