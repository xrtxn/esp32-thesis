#!/bin/bash

set -e

source .devcontainer/.bash_esp_init || echo "You are in the wrong directory"
cargo build
cargo espflash save-image --chip esp32s3 out/main.bin --partition-table=partitions.csv
