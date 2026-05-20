# Alacritty Config UI

A standalone desktop editor for layered Alacritty configuration with live preview, preset theme browsing, and ownership-aware TOML saving.

## What it does

- Loads the active Alacritty config graph from `alacritty.toml` and its imports.
- Preserves layered ownership instead of flattening everything into one file.
- Edits the currently supported surface:
  - `general.import`
  - `general.live_config_reload`
  - `scrolling`
  - `cursor`
  - `selection`
  - `font`
  - `window`
  - `colors`
- Provides both an embedded preview pane and a detached preview window.
- Integrates official preset themes from [`alacritty-theme`](https://github.com/alacritty/alacritty-theme) via git submodule.
- Lets you copy a preset into a writable custom theme before editing.

## Repository layout

- `src/app.rs` — top-level app state, panels, dialogs, save/reload flow.
- `src/config_graph/` — layered load, merge, and ownership-aware save logic.
- `src/config_types/` — trimmed serde model for the supported Alacritty config surface.
- `src/theme_catalog/` — bundled/local/custom theme discovery.
- `src/preview/` — terminal mock preview renderer.
- `vendor/alacritty-theme` — official preset theme catalog as a git submodule.
- `.github/workflows/ci.yml` — format, clippy, test, and release-build validation.
- `.github/workflows/publish-master.yml` — rolling `master-latest` prerelease publishing.

## Clone and run

Clone with the submodule so the bundled preset themes are available locally:

```bash
git clone --recurse-submodules git@github.com:dev2019zheng/alacritty-config-ui.git
cd alacritty-config-ui
cargo run
```

If you already cloned the repository without submodules:

```bash
git submodule update --init --recursive
```

## Development verification

Run the same checks used by CI:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

## Release behavior

Pushes to `master` trigger two GitHub Actions workflows:

1. `CI`
   - formatting
   - clippy
   - tests
   - release build
   - macOS packaging verification (`.app` bundle + DMG assembly)
2. `Publish master artifacts`
   - builds the macOS release binary
   - generates the app icon asset
   - packages bundled preset themes and license files into both a raw tarball and a macOS `.app`
   - publishes either a signed notarized DMG or an explicitly unsigned DMG, depending on configured Apple secrets
   - uploads workflow artifacts from `dist/upload/`
   - updates the rolling prerelease tag `master-latest`

The release pipeline always produces a self-contained tarball containing:

- `alacritty-config-ui`
- `themes/`
- `LICENSE-APACHE`
- `LICENSE-MIT`

It also builds a real macOS `.app` bundle and then wraps that bundle in a DMG.

## Local macOS packaging

Reproduce the release artifacts locally with:

```bash
cargo build --release
python3 scripts/generate_app_icon.py
bash scripts/package-macos-release.sh
```

The packaging script writes artifacts to `dist/upload/`:

- `alacritty-config-ui-macos.tar.gz`
- `alacritty-config-ui-macos.tar.gz.sha256`
- `alacritty-config-ui-macos.dmg` + checksum when signing and notarization are configured
- `alacritty-config-ui-macos-unsigned.dmg` + checksum otherwise

The raw tarball keeps `themes/` next to the executable. The `.app` bundle stores runtime themes and licenses under `Contents/Resources/`.

## DMG support boundaries

The repository now contains a real `.app` bundle pipeline, icon generation, DMG assembly, optional signing, and optional notarization. The support boundary is still explicit:

- A signed and notarized DMG only happens when the Apple certificate and notary secrets are configured.
- Without those secrets, the workflow intentionally publishes an `-unsigned` DMG instead of pretending the result is installer-grade.
- CI validates packaging structure on every `master` push, but it cannot verify Apple signing or notarization unless those credentials exist in GitHub Actions.

For GitHub Actions signing and notarization, the workflow expects these secrets:

- `APPLE_CERTIFICATE_P12_BASE64`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_NOTARY_APPLE_ID`
- `APPLE_NOTARY_TEAM_ID`
- `APPLE_NOTARY_APP_SPECIFIC_PASSWORD`

## License

Licensed under either of:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.
