#!/usr/bin/bash

ITCHIO_USERNAME="determinismdev"
ITCHIO_GAME="serverlooter"
ITCHIO_CHANNEL="web"

WORKING_DIR="${1:-$(pwd)}"
OUT_DIR="$WORKING_DIR/wasm_out"
OUT_NAME="serverlooter"
GIT_HASH=$(git rev-parse --short HEAD)
OUT_ZIP=$OUT_NAME-itch-$GIT_HASH.zip
BUTLER_EXE="${2:-butler}"

cp -r $WORKING_DIR/assets $OUT_DIR/assets

echo "Verifying OUT_DIR before zip:"
ls -l $OUT_DIR

zip -r $OUT_DIR/$OUT_ZIP \
  $OUT_DIR/serverlooter-${GIT_HASH}.js \
  $OUT_DIR/serverlooter_opt-${GIT_HASH}.wasm \
  $OUT_DIR/index.html \
  $OUT_DIR/assets

if [ $? -ne 0 ]; then
  echo "failed to zip all requested files (errno=$?); aborting."
  exit 1
fi

rm -r $OUT_DIR/assets

$BUTLER_EXE push $OUT_DIR/$OUT_ZIP $ITCHIO_USERNAME/$ITCHIO_GAME:$ITCHIO_CHANNEL

rm $OUT_DIR/$OUT_ZIP
