#!/bin/bash

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <PROJECT_NAME> <REMOTE_ADDR>"
    exit 1
fi

BINARY_NAME="$1"
PROJECT_DIR="$BINARY_NAME"

cd "$PROJECT_DIR" || { echo "Project directory not found"; exit 1; }
cargo build --release

REMOTE_USER="ubuntu"
REMOTE_HOST="$2"
REMOTE_PATH="/home/ubuntu/protohackers"
scp -i ~/.ssh/protohackers.pem "target/release/$BINARY_NAME" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH"

echo "Binary has been copied to the remote machine."
