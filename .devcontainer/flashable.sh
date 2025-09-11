#!/bin/bash

cargo espflash save-image --chip esp32s3 $CODESPACE_VSCODE_FOLDER/out/main.bin
