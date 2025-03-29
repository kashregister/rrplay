#!/bin/bash

cargo build --release
if [ $? -eq 0 ]; then
    BINARY_NAME=$(basename $(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].targets[0].name'))
    sudo mv target/release/$BINARY_NAME /usr/bin/

    if [ $? -eq 0 ]; then
        echo "Binary moved to /usr/bin successfully."
    else
        echo "Failed to move the binary to /usr/bin."
    fi
else
    echo "Cargo build failed."
fi
