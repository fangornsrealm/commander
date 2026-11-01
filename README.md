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

```sh
just build-release
just build-deb       # Debian / Ubuntu .deb
just build-rpm       # Redhat / Fedora / SUSE .rpm
```

## Resources

- [Contributing guide](CONTRIBUTING.md) — development workflow, feature flags, linting
- [Changelog](CHANGELOG.md) — release history
- [`justfile`](justfile) — all available build/install/package recipes (`just --list`)

## License

This project is licensed under [GPL-3.0](LICENSE).

Parts derived from the Iced project are modified and carry their own MIT license headers.
