# commander

Dual Panel File manager for the COSMIC desktop environment. Think of a less cluttered Krusader.

![commander](assets/commander.png)

This project is based on [COSMIC files](https://github.com/pop-os/cosmic-files) and the Terminal on [COSMIC term](https://github.com/pop-os/cosmic-term).

> [!NOTE]
> The basic functionality is working. Two panels, copying or moving files or tabs between them. A Terminal to run commands. And all the features of COSMIC Files that it inherits. Good enough to close krusader and use commander instead.



## Prerequisites

- **Rust** (MSRV 1.88, pinned in `rust-toolchain.toml`) — install via [rustup](https://rustup.rs)
- **`just`** task runner — see [just.systems](https://just.systems)
- **System libraries** (Debian/Ubuntu):
  ```sh
  sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libglib2.0-dev
  ```
- No C libraries are required for compression: `.bz2` and `.xz`/`.lzma`
  extraction use the pure-Rust crates `bzip2-rs` and `lzma-rs`, so there is
  no `libbz2-dev`/`liblzma-dev` build dependency.

## Build & Install

```sh
git clone https://github.com/fangornsrealm/commander
cd commander
just build-release   # optimized build (this may take a while)
sudo just install
# or just run it:
cargo run --release
```

## Packaging

All formats build from the same release binary and resources, and the
vendor-based formats (flatpak, snap, docker) work offline and with private
git dependencies because `just vendor` bakes a self-contained dependency
snapshot (`vendor.tar`) into the build.

```sh
just dist          # builds everything below (except docker)

just build-deb     # Debian / Ubuntu .deb        -> target/debian/
just build-rpm     # Red Hat / Fedora / SUSE     -> target/generate-rpm/
just appimage      # portable AppImage           -> Commander-x86_64.AppImage
just flatpak       # Flatpak bundle + repo       -> commander-0.1.0.flatpak
just snap          # snap package                -> commander_0.1.0_amd64.snap
just docker-build  # OCI image                   -> docker tag commander:0.1.0
```

Per-format notes:

- **deb / rpm** — `cargo-deb` / `cargo-generate-rpm` are installed on demand;
  the file lists live in `Cargo.toml` under `[package.metadata.deb]` and
  `[package.metadata.generate-rpm]`.
- **AppImage** — built with `linuxdeploy` (downloaded into `tools/` on first
  use). Result is a type-2 AppImage, run it directly:
  `./Commander-x86_64.AppImage`.
- **flatpak** — needs `flatpak` with the `org.freedesktop.Sdk//25.08` and
  `org.freedesktop.Sdk.Extension.rust-stable//25.08` runtimes (installed by
  the recipe on demand). Produces `commander-0.1.0.flatpak` plus a local repo
  in `flatpak/build/repo`. Try it with:
  ```sh
  just flatpak-run   # installs from the local repo and launches the app
  ```
- **snap** — needs `snapcraft` (installed on demand) and an LXD/KVM backend.
  The build bootstraps a Rust toolchain inside the core22 container, so the
  host needs no special setup beyond `just vendor`.
- **docker** — multi-stage build (`docker/Dockerfile`): a reproducible `rust`
  builder stage plus a slim Ubuntu 24.04 runtime stage with lavapipe (so it
  also works without a GPU). Run it against your X display:
  ```sh
  just docker-run
  ```

The Flatpak manifest (`flatpak/eu.fangornsrealm.commander.json`) is kept in
sync with the `just flatpak` recipe for portability to full flatpak builds /
Flathub.

## Resources

- [Contributing guide](CONTRIBUTING.md) — development workflow, feature flags, linting
- [Changelog](CHANGELOG.md) — release history
- [`justfile`](justfile) — all available build/install/package recipes (`just --list`)

## License

This project is licensed under [GPL-3.0](LICENSE).

Parts derived from the Iced project are modified and carry their own MIT license headers.
