# Base devcontainer
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

ARG DEBIAN_FRONTEND=noninteractive

# System dependencies for esp-idf builds and Rust crates
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl git ca-certificates pkg-config libssl-dev \
    cmake ninja-build libusb-1.0-0-dev \
    python3 python3-pip python3-venv \
    gperf bison flex ccache dfu-util wget xz-utils unzip git-lfs \
    libudev-dev \
 && rm -rf /var/lib/apt/lists/*

# Codespaces uses "vscode" user by default
USER vscode
ENV CARGO_HOME=/home/vscode/.cargo
ENV RUSTUP_HOME=/home/vscode/.rustup
ENV PATH=$CARGO_HOME/bin:$PATH

# Install rustup + cargo for the vscode user so `cargo` is available in the container
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
 && rustup default stable \
 && rustup component add clippy rust-src rustfmt || true \
 && cargo --version \
 && cargo +stable install cargo-espflash \
 && cargo +stable install cargo-espmonitor \
 && cargo +stable install ldproxy \
 && cargo +stable install espup --locked \
 && espup install
