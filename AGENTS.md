# AGENTS.md

## Project

Bandoo WebForge is a Tauri v2 desktop runtime for turning WebApps/PWAs into enhanced desktop applications. The product direction is:

- Keep the original web page as the core experience.
- Add desktop-grade capabilities around it.
- Support multiple platforms over time, with Linux as the current primary target.

The design source is `Bandoo_WebForge.md`. Use it as the product authority when implementation details are unclear.

## Stack

- Desktop runtime: Tauri v2
- Native layer: Rust
- Frontend: Vue 3, TypeScript, Pinia, Vue Router, SCSS, Vite
- Local storage: JSON now, SQLite later if the data model grows

## Architecture Guidelines

- Keep platform-specific behavior isolated in Rust modules such as `platform` and `runtime`.
- Keep shared domain types in `src-tauri/src/models.rs` and mirrored TypeScript types in `src/types/`.
- Do not pile new commands directly into a large `lib.rs`; add focused modules and expose only Tauri commands from `lib.rs`.
- Treat Linux as the first implementation target for desktop integration, but avoid hard-coding Linux paths in shared logic.
- Add Windows/macOS behavior behind explicit platform branches when needed.

## Current MVP Priorities

1. WebApp management: create, edit, delete, launch.
2. Window runtime: independent WebView windows, UserAgent, window state restore.
3. Linux desktop integration: `.desktop` files, tray, autostart.
4. Runtime enhancement: JS injection, `window.__BANDOO__`, URL/route detection.
5. Automation: Trigger, Condition, Action.
6. User scripts and script APIs.

## Development Commands

```bash
npm install
npm run build
. "$HOME/.cargo/env"
cd src-tauri
cargo fmt -- --check
cargo check
```

For local UI preview:

```bash
npm run dev -- --host 127.0.0.1
```

For desktop development:

```bash
. "$HOME/.cargo/env"
npm run tauri:dev
```

## Coding Rules

- Prefer existing patterns before adding new abstractions.
- Use TypeScript interfaces for frontend data contracts.
- Keep Rust structs serialized with `#[serde(rename_all = "camelCase")]` when they cross the Tauri boundary.
- Validate URLs before launching a WebApp window.
- Keep Shell/filesystem permissions disabled by default.
- Do not introduce unrelated refactors while implementing a feature.
- Do not commit generated directories or dependency folders.

## Generated Or Heavy Files

Do not edit or commit:

- `node_modules/`
- `dist/`
- `src-tauri/target/`
- `src-tauri/gen/`

## Verification

Before handing off meaningful code changes, run:

```bash
npm run build
. "$HOME/.cargo/env"
cd src-tauri
cargo fmt -- --check
cargo check
```

If Tauri desktop execution fails, first check Linux prerequisites: WebKitGTK, librsvg, Rust, Cargo, and rustup.
