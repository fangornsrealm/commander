# Reproducible build environment for commander.
#
# All dependencies come from vendor.tar (produced by `just vendor`), so the
# build works offline and with private git dependencies.
#
#   just vendor
#   docker build -f docker/Dockerfile -t commander .

ARG RUST_VERSION=1.93
FROM rust:${RUST_VERSION} AS builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libglib2.0-dev libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN tar xf vendor.tar \
    && cargo build --release --offline --locked

# Headless runtime stage. The app is a Wayland/X11 GUI: run it against a
# forwarded X display (e.g. `xhost +local; docker run --rm -it \
# -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix -v ~/.Xauthority:$HOME/.Xauthority \
# --device /dev/dri commander`) or mount a host Wayland socket and a Vulkan
# ICD from a driver container. mesa-vulkan-drivers (software lavapipe) is
# included so it also works without any host GPU.
FROM ubuntu:24.04
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libatomic1 \
        libblkid1 \
        libbz2-1.0 \
        libglib2.0-0t64 \
        liblzma5 \
        libmount1 \
        libpcre2-8-0 \
        libselinux1 \
        libxkbcommon0 \
        libz1 \
        mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/commander /usr/local/bin/commander
COPY res/eu.fangornsrealm.commander.desktop /usr/share/applications/eu.fangornsrealm.commander.desktop
COPY res/eu.fangornsrealm.commander.metainfo.xml /usr/share/metainfo/eu.fangornsrealm.commander.metainfo.xml
COPY res/eu.fangornsrealm.commander.svg /usr/share/icons/hicolor/scalable/apps/eu.fangornsrealm.commander.svg
ENTRYPOINT ["/usr/local/bin/commander"]
