#!/usr/bin/env bash

set -euo pipefail



PROJECT_NAME=hyperlit
VERSION=${1:-no-version}
TARGET=${RUSTTARGET:-x86_64-pc-windows-msvc}

echo "Building hyperlit binary"
echo "Target: $TARGET"
echo "Tag/Version: $VERSION"

cargo build -p $PROJECT_NAME --release --target $TARGET --locked

PACKAGE_NAME="$PROJECT_NAME-$VERSION-$TARGET"

if [[ $TARGET == *"windows"* ]]; then
  echo "Creating zip file"
  rm -f "$PACKAGE_NAME.zip"
  #zip -j -r "$PROJECT_NAME-$VERSION-$TARGET.zip" "./target/$TARGET/release/$PROJECT_NAME.exe"
  # Use power + Compressarchive instead for better windows compatibility
  powershell -command "Compress-Archive -Path './target/$TARGET/release/$PROJECT_NAME.exe' -DestinationPath $PACKAGE_NAME.zip -Force"
else
  echo "Creating tar.gz file"
  rm -f "$PACKAGE_NAME.tar.gz"
  tar -zcvf "$PACKAGE_NAME.tar.gz" "./target/$TARGET/release/$PROJECT_NAME"
fi
