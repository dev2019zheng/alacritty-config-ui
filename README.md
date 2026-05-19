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
2. `Publish master artifacts`
   - builds the macOS release binary
   - packages bundled preset themes and license files
   - uploads a workflow artifact
   - updates the rolling prerelease tag `master-latest`

The current release package is a tarball containing:

- `alacritty-config-ui`
- `themes/`
- `LICENSE-APACHE`
- `LICENSE-MIT`

## Why the release is `tar.gz`, not a `.dmg`

That is an intentional scope boundary in the current repository state.

Today the project publishes a standalone release binary plus runtime assets. The workflow does **not** build a macOS `.app` bundle, and it does **not** run any of the steps normally expected for a polished DMG installer flow:

- no `Info.plist` / bundle metadata
- no app icon packaging
- no `codesign`
- no notarization via `notarytool`
- no DMG assembly step such as `hdiutil`, `create-dmg`, or `appdmg`

A DMG without signing and notarization would still give users Gatekeeper friction, while also implying an installer-grade experience that the repo does not yet implement. Shipping a tarball is the honest current shape: a raw native binary plus the assets it needs at runtime.

If we want a real DMG path later, the missing work is clear:

1. produce a proper `.app` bundle
2. add bundle metadata and icon assets
3. sign the app with an Apple Developer identity
4. notarize the artifact
5. staple the notarization result
6. build and publish a DMG from that signed app bundle

Until those pieces exist, calling the current output a DMG installer would be packaging theater.

## License

Licensed under either of:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.
