#!/usr/bin/bash

ITCHIO_USERNAME="determinismdev"
ITCHIO_GAME="serverlooter"
ITCHIO_CHANNEL="web"

WORKING_DIR="${1:-$(pwd)}"
OUT_DIR="$WORKING_DIR/wasm_out"
OUT_NAME="serverlooter"
GIT_HASH=$(git rev-parse --short HEAD)
OUT_ZIP=$OUT_DIR/$OUT_NAME-itch-$GIT_HASH.zip

cp -r $WORKING_DIR/assets $OUT_DIR/assets

cd $OUT_DIR

zip -r $OUT_ZIP \
  serverlooter-${GIT_HASH}.js \
  serverlooter_opt-${GIT_HASH}.wasm \
  index.html \
  assets

cd $WORKING_DIR
rm -r $OUT_DIR/assets

butler push $OUT_ZIP $ITCHIO_USERNAME/$ITCHIO_GAME:$ITCHIO_CHANNEL

rm $OUT_ZIP
