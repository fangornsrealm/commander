name := 'commander'
export APPID := 'eu.fangornsrealm.commander'
rootdir := ''
prefix := '/usr'
base-dir := absolute_path(clean(rootdir / prefix))
export INSTALL_DIR := base-dir / 'share'
cargo-target-dir := env('CARGO_TARGET_DIR', 'target')
bin-src := cargo-target-dir / 'release' / name
bin-dst := base-dir / 'bin' / name
desktop := APPID + '.desktop'
desktop-src := 'res' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop
metainfo := APPID + '.metainfo.xml'
metainfo-src := 'res' / metainfo
metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo
icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor-snapshot vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Vendor dependencies locally (self-contained cargo home snapshot: works
# offline, and with private git dependencies that `cargo vendor` cannot)
vendor:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf vendor-snapshot vendor.tar
    python3 scripts/vendor_snapshot.py .
    # Bake the vergen version strings so builds without a git checkout work
    cfg="vendor-snapshot/.cargo/config.toml"
    : > "$cfg"
    echo '[env]' >> "$cfg"
    if [ -f .git ]
    then
        git_sha=$(git rev-parse HEAD)
        git_date=$(git log -1 --format=%cd --date=format:%Y-%m-%d)
    else
        git_sha=unknown
        git_date=$(date +%Y-%m-%d)
    fi
    echo "VERGEN_GIT_SHA = \"${git_sha}\"" >> "$cfg"
    echo "VERGEN_GIT_COMMIT_DATE = \"${git_date}\"" >> "$cfg"
    tar pcf vendor.tar -C vendor-snapshot .cargo
    rm -rf vendor-snapshot

# Compiles with debug profile
build-debug *args:
    cargo build {{ args }}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract
    CARGO_HOME=.cargo cargo build --release --frozen --offline {{ args }}

# Runs a clippy check
check *args:
    cargo clippy --all-features {{ args }} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Developer target
dev *args:
    cargo fmt
    just run {{ args }}

# Run with debug logs
run *args:
    cargo build --release
    env RUST_LOG=cosmic_files=info RUST_BACKTRACE=full {{ bin-src }} {{ args }}

# Run tests
test *args:
    cargo test {{ args }}

# Check i18n translation completeness
i18n-check:
    bash scripts/i18n/check_completeness.sh

flamegraph *args:
    cargo flamegraph --release --bin commander -- --no-daemon {{ args }}
    xdg-open flamegraph.svg

heaptrack *args:
    #!/usr/bin/env bash
    set -ex
    rm -fv heaptrack.commander.*
    cargo heaptrack --profile release-with-debug --bin commander -- --no-daemon {{ args }}
    zstd -dc < heaptrack.commander.*.raw.zst | /usr/lib/heaptrack/libexec/heaptrack_interpret | zstd -c > heaptrack.commander.zst
    heaptrack_gui heaptrack.commander.zst

# Installs files
install:
    install -Dm0755 {{ bin-src }} {{ bin-dst }}
    install -Dm0644 {{ desktop-src }} {{ desktop-dst }}
    install -Dm0644 {{ metainfo-src }} {{ metainfo-dst }}
    for size in `ls {{ icons-src }}`; do \
        install -Dm0644 "{{ icons-src }}/$size/apps/{{ APPID }}.svg" "{{ icons-dst }}/$size/apps/{{ APPID }}.svg"; \
    done

# Uninstalls installed files
uninstall:
    rm -f {{ bin-dst }}

# Installs files locally
[no-cd]
install-local:
    install -Dm0755 {{ bin-src }} ~/.local/bin/{{ bin-dst }}
    install -Dm0644 {{ desktop-src }} ${XDG_DATA_HOME:-~/.local/share}/{{ desktop-dst }}
    install -Dm0644 {{ metainfo-src }} ${XDG_DATA_HOME:-~/.local/share}/{{ metainfo-dst }}
    install -Dm0644 {{ icons-src }} ${XDG_DATA_HOME:-~/.local/share}/{{ icons-dst }}

# Uninstalls locally installed files
uninstall-local:
    rm ~/.local/bin/{{ bin-dst }}
    rm ${XDG_DATA_HOME:-~/.local/share}/{{ desktop-dst }}
    rm ${XDG_DATA_HOME:-~/.local/share}/{{ metainfo-dst }}
    rm ${XDG_DATA_HOME:-~/.local/share}/{{ icons-dst }}

# Compiles and packages deb with release profile
build-deb:
    command -v cargo-deb || cargo install cargo-deb
    cargo deb

[no-cd]
install-deb:
    apt install --reinstall ./target/debian/*.deb

# Compiles and packages rpm with release profile
[no-cd]
build-rpm: build-release
    command -v cargo-generate-rpm || cargo install cargo-generate-rpm
    strip -s {{ bin-src }}
    cargo generate-rpm

[no-cd]
install-rpm:
    dnf install ./target/generate-rpm/*.rpm

# Vendor dependencies locally (self-contained cargo home snapshot; see `vendor`)
vendor-extract:
    rm -rf .cargo
    tar pxf vendor.tar

# --- Distribution packages ---------------------------------------------------

# Prints the package version from Cargo.toml
[private]
app-version:
    @sed -nE 's/^version = "([0-9.]+)".*/\1/p' Cargo.toml | head -n1

# Regenerates the vendored dependency snapshot, then builds a Flatpak bundle
# Builds a Flatpak bundle. This flatpak build lacks `build-from-manifest`,
# so the recipe drives the build dir directly, following
# flatpak/eu.fangornsrealm.commander.json (keep both in sync).
flatpak: vendor
    #!/usr/bin/env bash
    set -euo pipefail
    if ! flatpak info --user org.freedesktop.Sdk//25.08 >/dev/null 2>&1
    then
        flatpak install --user -y org.freedesktop.Sdk//25.08 org.freedesktop.Sdk.Extension.rust-stable//25.08
    fi
    builddir="flatpak/build/eu.fangornsrealm.commander"
    if [ ! -d "$builddir" ]
    then
        flatpak build-init --arch x86_64 "$builddir" eu.fangornsrealm.commander org.freedesktop.Sdk org.freedesktop.Sdk 25.08
    fi
    # Manifest sources: the project (minus build artifacts) plus vendor.tar
    mkdir -p "$builddir/src"
    tar --exclude=.git --exclude=target --exclude=.cargo --exclude=vendor-snapshot \
        --exclude=flatpak/build --exclude=samples \
        --exclude=AppDir --exclude=Commander-x86_64.AppImage \
        -cf - . | tar xf - -C "$builddir/src"
    cp scripts/flatpak_build.sh "$builddir/build.sh"
    # This flatpak build does not auto-mount SDK extensions, so bind the
    # rust-stable extension into its canonical prefix (/usr/lib/sdk/rust-stable)
    rust_ext=$(readlink -f ~/.local/share/flatpak/runtime/org.freedesktop.Sdk.Extension.rust-stable/x86_64/25.08/active/files 2>/dev/null) \
        || rust_ext=$(ls -d ~/.local/share/flatpak/runtime/org.freedesktop.Sdk.Extension.rust-stable/x86_64/25.08/*/files | head -1)
    [ -n "$rust_ext" ] && [ -x "$rust_ext/bin/cargo" ]
    flatpak build --bind-mount=/usr/lib/sdk/rust-stable="$rust_ext" "$builddir" bash "$PWD/flatpak/build/eu.fangornsrealm.commander/build.sh"
    if ! grep -q '^\[Application\]' "$builddir/metadata"
    then
        flatpak build-finish "$builddir" \
            --command=commander \
            --metadata=Name=Commander \
            --metadata=Description="Dual panel file manager for the COSMIC desktop" \
            --share=ipc --share=network \
            --socket=x11 --socket=wayland \
            --device=dri \
            --filesystem=home \
            --talk-name=org.freedesktop.Notifications
    fi
    rm -rf flatpak/build/repo
    flatpak build-export "flatpak/build/repo" "$builddir"
    # build-init commits the app under the `master` branch
    flatpak build-bundle "flatpak/build/repo" "commander-$(just app-version).flatpak" eu.fangornsrealm.commander master

# Installs the built flatpak (user installation) and runs it
[no-cd]
flatpak-run:
    flatpak remote-add --user --if-not-exists --no-gpg-verify commander-local "file://$(pwd)/flatpak/build/repo"
    flatpak install --user commander-local eu.fangornsrealm.commander
    flatpak run eu.fangornsrealm.commander

# Builds a snap from snap/snapcraft.yaml (needs snapcraft, installed on demand).
# Creates a filtered source dir (snap-src/) first so the build container does
# not swallow local build artifacts (target/, flatpak/build/, ...).
[no-cd]
snap: vendor
    command -v snapcraft || sudo snap install snapcraft --classic
    rm -rf snap-src
    mkdir -p snap-src
    tar --exclude=.git --exclude=target --exclude=.cargo --exclude=vendor-snapshot \
        --exclude=flatpak/build --exclude=AppDir --exclude=tools --exclude=samples \
        --exclude=Commander-x86_64.AppImage \
        -cf - . | tar xf - -C snap-src
    snapcraft pack

# Builds an AppImage via linuxdeploy (downloaded to tools/ on demand)
appimage: build-release
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -x tools/linuxdeploy ]
    then
        mkdir -p tools
        curl -fsSL -o tools/linuxdeploy-x86_64.AppImage https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage
        chmod +x tools/linuxdeploy-x86_64.AppImage
        rm -rf tools/linuxdeploy-root tools/squashfs-root
        (cd tools && ./linuxdeploy-x86_64.AppImage --appimage-extract)
        mv tools/squashfs-root tools/linuxdeploy-root
        rm -f tools/linuxdeploy-x86_64.AppImage
        ln -sf linuxdeploy-root/usr/bin/linuxdeploy tools/linuxdeploy
    fi
    rm -rf AppDir
    mkdir -p AppDir/usr/bin AppDir/usr/share/applications AppDir/usr/share/metainfo AppDir/usr/share/icons/hicolor/256x256/apps
    install -Dm0755 target/release/commander AppDir/usr/bin/commander
    install -Dm0644 res/eu.fangornsrealm.commander.desktop AppDir/usr/share/applications/eu.fangornsrealm.commander.desktop
    install -Dm0644 res/eu.fangornsrealm.commander.metainfo.xml AppDir/usr/share/metainfo/eu.fangornsrealm.commander.metainfo.xml
    install -Dm0644 res/eu.fangornsrealm.commander.svg AppDir/usr/share/icons/hicolor/256x256/apps/eu.fangornsrealm.commander.svg
    tools/linuxdeploy --appdir AppDir --output appimage

# Builds the docker image (reproducible builder stage + headless runtime stage)
[no-cd]
docker-build: vendor
    docker build -f docker/Dockerfile -t commander:$(just app-version) .

# Runs the headless runtime image against this machine's X display
[no-cd]
docker-run: docker-build
    xhost +local:
    docker run --rm -it -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix -v ~/.Xauthority:$HOME/.Xauthority --device /dev/dri commander:$(just app-version)

# Builds every distribution format (deb, rpm, flatpak, snap, AppImage)
dist: build-deb build-rpm flatpak snap appimage
