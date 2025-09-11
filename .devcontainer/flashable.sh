#!/bin/bash
set -e

cargo clean -p esp32-thesis && cargo build
cargo espflash save-image --chip esp32s3 out/main.bin
