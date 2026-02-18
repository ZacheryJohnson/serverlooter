#!/usr/bin/bash

ITCHIO_USERNAME="determinismdev"
ITCHIO_GAME="serverlooter"
ITCHIO_CHANNEL="web"

WORKING_DIR="${1:-.}"
OUT_DIR="$WORKING_DIR/wasm_out"
OUT_NAME="serverlooter"
GIT_HASH=$(git rev-parse --short HEAD)
OUT_ZIP=$OUT_DIR/$OUT_NAME-itch-$GIT_HASH.zip

zip $OUT_ZIP \
  $OUT_DIR/serverlooter-${GIT_HASH}.js \
  $OUT_DIR/serverlooter_opt-${GIT_HASH}.wasm \
  $OUT_DIR/index.html

butler push $OUT_ZIP $ITCHIO_USERNAME/$ITCHIO_GAME:$ITCHIO_CHANNEL

rm $OUT_ZIP
