# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-04-04

### Added

1. Tauri + React cross-platform project scaffold
2. Main control page for Home Assistant climate state, power, refresh, and temperature control
3. Config page for base URL, entity ID, startup settings, climate defaults, and token management
4. Rust config store with validation and legacy `appsettings.json` import support
5. Rust Home Assistant client for state fetch, turn on, turn off, and set temperature
6. Secure token storage abstraction via system keychain APIs
7. Tray menu with open, config, and quit actions
8. Close-to-tray and minimize-to-tray behavior
9. Cross-platform startup integration via `auto-launch`
10. CI workflow, release workflow, release checklist, and icon asset guidance

### Verified

1. `npm run build`
2. `cargo check --manifest-path src-tauri/Cargo.toml`
3. `xvfb-run -a sh -lc '. "$HOME/.cargo/env" && npm exec tauri dev'`
