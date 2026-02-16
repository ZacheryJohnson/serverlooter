#!/usr/bin/bash

WORKING_DIR="${1:-.}"

OUT_DIR="$WORKING_DIR/wasm_out"
OUT_NAME="serverlooter"
CARGO_BUILD_PROFILE="wasm"

cargo build --profile $CARGO_BUILD_PROFILE --target wasm32-unknown-unknown
if [ -d "$OUT_DIR" ]; then rm -rf $OUT_DIR; fi
mkdir $OUT_DIR
wasm-bindgen --target web \
  --out-dir $OUT_DIR \
  --out-name $OUT_NAME \
  $WORKING_DIR/target/wasm32-unknown-unknown/$CARGO_BUILD_PROFILE/serverlooter.wasm

wasm-opt -Oz -o $OUT_DIR/${OUT_NAME}_opt.wasm $OUT_DIR/${OUT_NAME}_bg.wasm
gzip -kf $OUT_DIR/${OUT_NAME}_opt.wasm

rm $OUT_DIR/serverlooter.d.ts
rm $OUT_DIR/serverlooter_bg.wasm
rm $OUT_DIR/serverlooter_bg.wasm.d.ts

GIT_HASH=$(git rev-parse --short HEAD)
export GIT_HASH
mv $OUT_DIR/serverlooter.js $OUT_DIR/serverlooter-${GIT_HASH}.js
mv $OUT_DIR/serverlooter_opt.wasm $OUT_DIR/serverlooter_opt-${GIT_HASH}.wasm
mv $OUT_DIR/serverlooter_opt.wasm.gz $OUT_DIR/serverlooter_opt-${GIT_HASH}.wasm.gz
envsubst < $WORKING_DIR/scripts/index.html.template > $OUT_DIR/index.html
