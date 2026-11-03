# Contributing to Commander

Thank you for your interest in contributing! This document covers how to set
up a development environment, the project's feature flags, and the
conventions we follow.

## Prerequisites

Commander targets Linux (COSMIC, GNOME, KDE, …). System dependencies:

- **Rust** — see [MSRV](#msrv) below. Install via [rustup](https://rustup.rs).
- **`just`** — task runner used by the `justfile` ([just.systems](https://just.systems)).
- **GTK 3 / WebKitGTK** development headers, required by iced/wgpu via
  libcosmic:
  ```sh
  # Debian / Ubuntu
  sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libglib2.0-dev
  # Fedora
  sudo dnf install gtk3-devel webkit2gtk4.1-devel glib2-devel
  ```
- No C libraries are required for compression: `.bz2` and `.xz`/`.lzma`
  extraction use the pure-Rust crates `bzip2-rs` and `lzma-rs`, so there is
  no `libbz2-dev`/`liblzma-dev` build dependency.
- Optional, for the GVFS / desktop integrations (`gio`, `glib`):
  `libgio2.0-dev` / `glib2-devel` are covered by the `glib2-devel` package
  above on most distros.

## MSRV

The minimum supported Rust version is pinned in `rust-toolchain.toml`
(`1.88.0`) and `Cargo.toml` (`rust-version = "1.88"`). Install that toolchain
via rustup before building; CI enforces it in a dedicated MSRV job.

## Getting started

```sh
git clone https://github.com/fangornsrealm/commander
cd commander
just build-release   # cargo build --release
just run             # build (debug) and run
```

`just` with no arguments runs the default recipe (`build-release`).
Run `just --list` for all recipes.

## Feature flags

The default feature set is `["bzip2", "desktop", "gvfs", "liblzma", "notify", "wgpu"]`.
Non-default / optional features:

| Feature      | Default | Notes                                                        |
| ------------ | ------- | ------------------------------------------------------------ |
| `desktop`    | yes     | libcosmic desktop integration, cosmic-mime-apps, xdg          |
| `gvfs`       | yes     | GVFS remote volume mounting (`gio`, `glib`)                   |
| `notify`     | yes     | desktop notifications (`notify-rust`)                          |
| `bzip2`      | yes     | `.bz2` extraction (pure-Rust `bzip2-rs`)                      |
| `liblzma`    | yes     | `.xz`/`.lzma` extraction (pure-Rust `lzma-rs`)                |
| `wgpu`       | yes     | GPU renderer backend                                           |
| `wayland`    | no      | Wayland backend (mutually exclusive-ish with `wgpu`; test both) |
| `jemalloc`   | no      | tikv-jemallocator instead of the glibc `mallopt` fallback      |

CI runs the full matrix (`--all-features` and `--no-default-features`), so
please make sure your changes compile under both. When adding `#[cfg]`-gated
code, check it against the relevant feature combinations locally:

```sh
just check -- --no-default-features
just check -- --all-features
```

## Testing

```sh
cargo test                       # default features
cargo test --all-features
cargo test --no-default-features
```

Notes on writing tests:

- New UI strings **must** use `fl!("…")` so they are translatable; tests that
  compare generated strings should use `fl!` rather than hard-coded English
  (see the existing tests in `src/operation/mod.rs`).
- File-system tests should create isolated fixtures with `tempfile` and clean
  up (the `tempfile::TempDir` pattern is used throughout).
- Prefer testing pure logic (e.g. `copy_unique_path`, `get_directory_name`,
  config round-trips) over full UI flows; the iced runtime makes headless UI
  tests expensive.

## Linting & formatting

CI enforces the following — keep them green locally before pushing:

```sh
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo clippy --no-default-features -- -D warnings
```

The project's `just check` recipe runs clippy with the pedantic set; use it
as a stricter local pass:

```sh
just check
```

## Local dev workflow (patched dependencies)

Several dependencies are tracked upstream in git (`libcosmic`,
`cosmic-text`, …). For local development against an adjacent checkout, the
`[patch.'https://github.com/pop-os/…']` blocks in `Cargo.toml` are provided
in commented form:

```toml
# [patch.'https://github.com/pop-os/libcosmic']
# libcosmic = { path = "../libcosmic" }
# cosmic-config = { path = "../libcosmic/cosmic-config" }
# cosmic-theme = { path = "../libcosmic/cosmic-theme" }
```

Uncomment the block(s) you need, `cargo build`, and **revert before
committing** — patches to path deps do not publish. A suggested layout is a
sibling checkout: `…/libcosmic` next to `…/commander`.

## Code style

- Edition 2021, current MSRV idioms. Avoid stabilisation-recent features that
  would break the MSRV job.
- Avoid `unsafe`. The one intentional `unsafe` block (glibc `mallopt` in
  `src/main.rs`) is annotated with `#[allow(unsafe_code)]` and a comment
  explaining the invariant; keep it that way.
- Prefer `?` / `match` over `.unwrap()`. If you keep an `unwrap()` or
  `expect()`, state the invariant it relies on.
- `TODO` comments should reference a GitHub issue (`TODO: #123`) or state a
  concrete, actionable follow-up; vague TODOs get removed in review.

## Translations

- All user-visible strings go through `fl!("…")` (`i18n-embed` + fluent).
- Add new strings to the source locale `i18n/en/*.ftl`; translation
  completeness is checked per-locale (see `i18n/` for `en`, `de`, `sv`).
- Date/time display should honour the active locale
  (`LANGUAGE_CHRONO` / `chrono` locales) — don't hard-code `"%b %-d …"`
  formats for new UI.

## Packaging

Distribution formats and their entry points (all are wired into the
`justfile`; see the README's Packaging section for usage):

| Format   | Recipe          | Tooling                                        | Output                          |
| -------- | --------------- | ---------------------------------------------- | ------------------------------- |
| deb      | `just build-deb`   | `cargo-deb` (installed on demand)              | `target/debian/*.deb`           |
| rpm      | `just build-rpm`   | `cargo-generate-rpm` (installed on demand)     | `target/generate-rpm/*.rpm`     |
| AppImage | `just appimage`    | `linuxdeploy` (auto-downloaded into `tools/`)  | `Commander-x86_64.AppImage`     |
| flatpak  | `just flatpak`     | `flatpak` + Sdk 25.08 + rust-stable extension  | `commander-<ver>.flatpak`       |
| snap     | `just snap`        | `snapcraft` + LXD/KVM                          | `commander_<ver>_amd64.snap`    |
| docker   | `just docker-build`| `docker`                                       | image `commander:<ver>`         |

Rules of thumb:

- **Resource files**: when adding files under `res/` (icons, metainfo,
  desktop entries), update **all** of: `[package.metadata.deb]` and
  `[package.metadata.generate-rpm]` in `Cargo.toml`, the `install` recipe in
  the `justfile`, the build steps in `scripts/flatpak_build.sh`, the build
  commands in `snap/snapcraft.yaml`, the `COPY` lines in `docker/Dockerfile`,
  and the AppImage recipe. Keep the names lowercase — the Flatpak ID and the
  desktop file name must match.
- **Versions**: `snap/snapcraft.yaml` carries a hard-coded `version:` that
  must be kept in sync with `Cargo.toml` (the other formats read it from
  `Cargo.toml` at build time).
- The `flatpak/` manifest and the `just flatpak` recipe describe the same
  build (the local flatpak lacks `build-from-manifest`); update both
  together.
- `just vendor` regenerates `vendor.tar`, the self-contained dependency
  snapshot consumed by the flatpak, snap and docker builds (it also bakes
  the `vergen` git metadata). Run it whenever `Cargo.toml`/`Cargo.lock`
  change.
- The desktop file must keep valid `MimeType=` entries if Commander is to be
  selectable as a default file manager; run `appstreamcli validate` on the
  metainfo when editing it.

## Submitting changes

1. Fork and branch from `main`.
2. Make a focused change; keep PRs reviewable (one concern per PR).
3. Run `just check`, `cargo test --all-features`, and `cargo fmt --all`.
4. Update `CHANGELOG.md` (or note it in the PR description) for user-visible
   changes.
5. Open a PR against `main`; CI (fmt, clippy, tests, MSRV, build) must pass.

## License

By contributing, you agree that your contributions will be licensed under the
project's [GPL-3.0](LICENSE) license. Files that are minor modifications of
Iced (MIT) code carry their own SPDX headers — preserve them.
