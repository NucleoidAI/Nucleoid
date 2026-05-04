#!/usr/bin/env bash
# Build script for Nucleoid C++
# Requires: cmake >= 3.20, clang++ or g++ (C++20), git

set -e

# Install cmake if missing (macOS via Homebrew)
if ! command -v cmake &>/dev/null; then
  if command -v brew &>/dev/null; then
    echo "Installing cmake via Homebrew..."
    brew install cmake
  else
    echo "ERROR: cmake not found. Install it via your package manager."
    exit 1
  fi
fi

BUILD_TYPE="${1:-Release}"
BUILD_DIR="build"

echo "Configuring (${BUILD_TYPE})..."
cmake -B "$BUILD_DIR"         \
      -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
      -DCMAKE_EXPORT_COMPILE_COMMANDS=ON

echo "Building..."
cmake --build "$BUILD_DIR" --parallel "$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"

echo ""
echo "Done. Binary: ${BUILD_DIR}/nucleoid"
echo "Usage: ./${BUILD_DIR}/nucleoid start --port 8448"
