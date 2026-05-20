# CLAUDE.md

## Repo-specific rules

- Keep `vendor/alacritty-theme` as a **git submodule**. Do not flatten it into copied files.
- Preserve the current dual-license model: `Apache-2.0 OR MIT` with `LICENSE-APACHE` and `LICENSE-MIT` present at the repo root.
- Keep DMG claims honest: the repository can build a real `.app` bundle and DMG, but only call a release signed/notarized when the Apple certificate and notary secrets are actually configured.

## Runtime packaging assumptions

- Bundled preset themes are expected at runtime either from a `themes/` directory next to the executable or from `.app/Contents/Resources/themes`.
- Development mode may fall back to the checked-out submodule under `vendor/alacritty-theme/themes`.
- If you change theme discovery, keep both packaged-runtime paths and the local-development fallback working.

## Release workflow invariants

If you change `.github/workflows/publish-master.yml` or `scripts/package-macos-release.sh`, keep the release artifacts self-contained.

The tarball must continue to include:

- `alacritty-config-ui`
- `themes/`
- `LICENSE-APACHE`
- `LICENSE-MIT`

The `.app` bundle must continue to include:

- `Contents/MacOS/alacritty-config-ui`
- `Contents/Resources/themes/`
- `Contents/Resources/LICENSE-APACHE`
- `Contents/Resources/LICENSE-MIT`

`actions/checkout` must continue to fetch submodules recursively, or bundled preset themes will disappear from CI and published releases.

## Verification expectations

For code changes, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

For packaging changes, also verify the release archive contents locally before claiming success:

```bash
python3 scripts/generate_app_icon.py
bash scripts/package-macos-release.sh
```

## Scope reminders

- The editor is ownership-aware: root/base/theme documents should stay separate.
- Avoid flattening the user's layered Alacritty config into a single file.
- Prefer honest support boundaries over aspirational claims.
