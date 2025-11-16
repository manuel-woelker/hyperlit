#!/usr/bin/env bash

set -euo pipefail

(

cd "$(dirname "$0")/.."


PROJECT_NAME=hyperlit
VERSION=${1:-no-version}
TARGET=${RUSTTARGET:-x86_64-pc-windows-msvc}

echo "Building hyperlit binary"
echo "Target: $TARGET"
echo "Tag/Version: $VERSION"

case "$OSTYPE" in
  linux*)   EXE_EXT="" ;;         # Linux binaries have no extension
  darwin*)  EXE_EXT="" ;;         # macOS binaries have no extension
  cygwin*)  EXE_EXT=".exe" ;;     # Cygwin uses Windows executables
  msys*)    EXE_EXT=".exe" ;;     # Git Bash / MinGW on Windows
  win32*)   EXE_EXT=".exe" ;;     # Native Windows
  *)        EXE_EXT="" ;;         # Default (safe fallback)
esac

./tool-tool$EXE_EXT --download
# On linux make bun exe executable

if [[ $OSTYPE == linux* ]]; then
  echo "Making bun executable"
  chmod +x ./.tool-tool/v2/cache/bun-*-linux/bun
fi


# Build the UI
UI_ZIP_FILE="./target/ui.zip"
rm -f UI_ZIP_FILE
(cd ui && ../tool-tool$EXE_EXT bun run build)
(cd ui/dist && zip -r ../../$UI_ZIP_FILE "*")

RELEASE_BINARY="./target/release/$PROJECT_NAME$EXE_EXT"
rm -f RELEASE_BINARY
# Build the binary
cargo build -p $PROJECT_NAME --release --locked

# Append the ui zip to the binary
cat "./target/ui.zip" >> $RELEASE_BINARY

PACKAGE_NAME="$PROJECT_NAME-$VERSION-$TARGET"

if [[ $TARGET == *"windows"* ]]; then
  echo "Creating zip file"
  rm -f "$PACKAGE_NAME.zip"
  #zip -j -r "$PROJECT_NAME-$VERSION-$TARGET.zip" "./target/$TARGET/release/$PROJECT_NAME.exe"
  # Use power + Compressarchive instead for better windows compatibility
  powershell -command "Compress-Archive -Path '$RELEASE_BINARY' -DestinationPath $PACKAGE_NAME.zip -Force"
else
  echo "Creating tar.gz file"
  rm -f "$PACKAGE_NAME.tar.gz"
  tar -zcvf "$PACKAGE_NAME.tar.gz" $RELEASE_BINARY
fi

echo "✅ Built $RELEASE_BINARY"
)