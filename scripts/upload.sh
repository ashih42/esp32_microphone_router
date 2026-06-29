#!/usr/bin/env bash

# ---------------------------------------------------------------------------------------
# This script serves as an alternative to running:
# espflash flash --monitor --chip esp32 --baud 115200
#
# The upload step is slow, but this way always works, whereas `espflash` fails all the time.
#
# Run this script by simply running:
# cargo run --bin <bin_name>
#
# `cargo` will pass 1 argument to this script: a relative path to my Rust program's compiled
# and linked binary, such as "target/xtensa-esp32-espidf/debug/<bin_name>".
#
# This is configured in `.cargo/config.toml` by setting `runner` to this script.
# ---------------------------------------------------------------------------------------

# Check there is exactly 1 argument.
if [ $# -ne 1 ]; then
    echo "Usage: $0 <bin_path>"
    exit 1
fi

BIN_PATH=$1
BIN_NAME=$(basename $BIN_PATH)

PROJECT_DIR=$(dirname $(dirname $(readlink -f $0)))
IMAGE_PATH="$PROJECT_DIR/_images/$BIN_NAME.bin"

BAUD=115200
CHIP=esp32
SERIAL='/dev/cu.usbserial-0001'
# Note: May need to find serial by running `espflash list-ports`.

# 1. Create an image.
espflash save-image --chip $CHIP --merge $BIN_PATH $IMAGE_PATH

# 2. Upload the image.
esptool --chip $CHIP --baud $BAUD write_flash 0x0 $IMAGE_PATH

# Check if upload finished successfully.
read -n 1 -s -r -p 'Press any key to continue to serial monitor...'
echo

# 3. Connect to serial monitor.
TERM=xterm screen $SERIAL $BAUD

# Note: To exit, press `Ctrl`+`A`, then `K`, and then `Y`.
