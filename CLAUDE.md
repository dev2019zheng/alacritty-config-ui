# CLAUDE.md

## Repo-specific rules

- Keep `vendor/alacritty-theme` as a **git submodule**. Do not flatten it into copied files.
- Preserve the current dual-license model: `Apache-2.0 OR MIT` with `LICENSE-APACHE` and `LICENSE-MIT` present at the repo root.
- Do not claim DMG support unless the repository actually contains an `.app` bundle pipeline plus signing and notarization.

## Runtime packaging assumptions

- Bundled preset themes are expected at runtime from a `themes/` directory next to the executable.
- Development mode may fall back to the checked-out submodule under `vendor/alacritty-theme/themes`.
- If you change theme discovery, keep both the packaged-runtime path and the local-development fallback working.

## Release workflow invariants

If you change `.github/workflows/publish-master.yml`, keep the release artifact self-contained. The tarball must continue to include:

- `alacritty-config-ui`
- `themes/`
- `LICENSE-APACHE`
- `LICENSE-MIT`

`actions/checkout` must continue to fetch submodules recursively, or bundled preset themes will disappear from CI and published releases.

## Verification expectations

For code changes, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

For packaging changes, also verify the release archive contents locally before claiming success.

## Scope reminders

- The editor is ownership-aware: root/base/theme documents should stay separate.
- Avoid flattening the user's layered Alacritty config into a single file.
- Prefer honest support boundaries over aspirational claims.
