#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Starting pairings solver..."
time DYLD_LIBRARY_PATH="$SCRIPT_DIR/target/release/build/scip-sys-0c5a5fa69df64301/out/scip_install/lib" "$SCRIPT_DIR/target/release/pairings" "$@"
