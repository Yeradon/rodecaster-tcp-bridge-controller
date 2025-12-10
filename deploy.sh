#!/bin/bash
set -e

REMOTE_HOST="root@192.168.7.22"
REMOTE_TMP="/tmp"
TARGET="aarch64-unknown-linux-musl"

# Define project root (assuming this script is in root)
PROJECT_ROOT=$(pwd)

# Default paths for Release Mode (Root of tarball)
BIN_TCP_BRIDGE="$PROJECT_ROOT/tcp-bridge"
BIN_BRIDGE_CTL="$PROJECT_ROOT/bridge-ctl"

# Check Mode
if [ -f "$PROJECT_ROOT/tcp-bridge/Cargo.toml" ]; then
    echo " -> detected Dev Environment (Cargo.toml found)."
    echo " -> Building binaries..."
    
    cd "$PROJECT_ROOT/tcp-bridge"
    cross build --release --target $TARGET
    if [ $? -ne 0 ]; then
        echo "Build failed!"
        exit 1
    fi
    cd "$PROJECT_ROOT"
    
    # Update paths to point to target dir
    BIN_TCP_BRIDGE="$PROJECT_ROOT/tcp-bridge/target/$TARGET/release/tcp-bridge"
    BIN_BRIDGE_CTL="$PROJECT_ROOT/tcp-bridge/target/$TARGET/release/bridge-ctl"
else
    echo " -> Detected Release Mode (No Cargo.toml)."
    echo " -> Expecting binaries in root..."
fi

# Verify Binaries Exist
if [ ! -f "$BIN_TCP_BRIDGE" ]; then
    echo "Error: tcp-bridge binary not found at $BIN_TCP_BRIDGE"
    exit 1
fi
if [ ! -f "$BIN_BRIDGE_CTL" ]; then
    echo "Error: bridge-ctl binary not found at $BIN_BRIDGE_CTL"
    exit 1
fi

echo " -> Deploying to $REMOTE_HOST..."

# Kill existing processes
ssh "$REMOTE_HOST" "pkill tcp-bridge || true"

# SCP Files
scp "$BIN_TCP_BRIDGE" \
    "$BIN_BRIDGE_CTL" \
    "$PROJECT_ROOT/scripts/run-proxy.sh" \
    "$PROJECT_ROOT/scripts/run-sniffer.sh" \
    "$PROJECT_ROOT/scripts/test_mappings.sh" \
    "$REMOTE_HOST:$REMOTE_TMP/"

# Make executable
ssh "$REMOTE_HOST" "chmod +x $REMOTE_TMP/tcp-bridge $REMOTE_TMP/bridge-ctl $REMOTE_TMP/*.sh"

echo " -> Done. Run '/tmp/run-proxy.sh' on device to start."
