#!/bin/bash
# Build libclawguard.so — LD_PRELOAD syscall interception library
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "[BUILD] Compiling libclawguard.so..."
gcc -shared -fPIC -O2 -Wall -Wextra -Wno-nonnull-compare \
    -o "$PROJECT_DIR/libclawguard.so" \
    "$PROJECT_DIR/src/preload/interpose.c" \
    -ldl

echo "[BUILD] Built: $PROJECT_DIR/libclawguard.so"
