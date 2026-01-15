#!/usr/bin/env bash

set -euo pipefail

(

cd "$(dirname "$0")/../ui"


echo "Building hyperlit UI"

case "$OSTYPE" in
  linux*)   EXE_EXT="" ;;         # Linux binaries have no extension
  darwin*)  EXE_EXT="" ;;         # macOS binaries have no extension
  cygwin*)  EXE_EXT=".exe" ;;     # Cygwin uses Windows executables
  msys*)    EXE_EXT=".exe" ;;     # Git Bash / MinGW on Windows
  win32*)   EXE_EXT=".exe" ;;     # Native Windows
  *)        EXE_EXT="" ;;         # Default (safe fallback)
esac

../tool-tool$EXE_EXT --download

# Build the UI
../tool-tool$EXE_EXT npm install
../tool-tool$EXE_EXT npm run build
)