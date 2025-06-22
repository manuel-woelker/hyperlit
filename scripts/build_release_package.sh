#!/usr/bin/env bash

set -euo pipefail



PROJECT_NAME=hyperlit
VERSION=${1:-no-version}
TARGET=${RUSTTARGET:-x86_64-pc-windows-msvc}

echo "Building hyperlit binary"
echo "Target: $TARGET"
echo "Tag/Version: $VERSION"

cargo build -p $PROJECT_NAME --release --target $TARGET --frozen

if [[ $TARGET == *"windows"* ]]; then
  echo "Creating zip file"
  rm -f "$PROJECT_NAME-$TARGET.zip"
  zip -j -r "$PROJECT_NAME-$VERSION-$TARGET.zip" "./target/$TARGET/release/$PROJECT_NAME.exe"
else
  echo "Creating tar.gz file"
  rm -f "$PROJECT_NAME-$TARGET.tar.gz"
  tar -zcvf "$PROJECT_NAME-$VERSION-$TARGET.tar.gz" "./target/$TARGET/release/$PROJECT_NAME"
fi
