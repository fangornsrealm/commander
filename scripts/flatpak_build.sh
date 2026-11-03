#!/usr/bin/env bash
# Build script for the commander Flatpak sandbox.
# Copied into the flatpak build dir and run via `flatpak build <dir> bash /build.sh`.
# Mirrors the build-commands of flatpak/eu.fangornsrealm.commander.json.
set -eux
# $0 is the script's host path (the build dir is mounted at its host path)
cd "$(dirname "$0")/src"
tar xf vendor.tar
export CARGO_HOME="$(pwd)/.cargo"
export PATH="/usr/lib/sdk/rust-stable/bin:$PATH"
export RUSTUP_HOME="$CARGO_HOME"
cargo build --release --offline --locked
install -Dm0755 target/release/commander /app/bin/commander
install -Dm0644 res/eu.fangornsrealm.commander.desktop /app/share/applications/eu.fangornsrealm.commander.desktop
install -Dm0644 res/eu.fangornsrealm.commander.metainfo.xml /app/share/metainfo/eu.fangornsrealm.commander.metainfo.xml
install -Dm0644 res/eu.fangornsrealm.commander.svg /app/share/icons/hicolor/scalable/apps/eu.fangornsrealm.commander.svg
