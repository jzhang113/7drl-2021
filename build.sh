#!/bin/bash
NAME="tymerl"

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen "target/wasm32-unknown-unknown/release/$NAME.wasm" --out-dir wasm --no-modules --no-typescript

mv "wasm/$NAME.js" "wasm/myblob.js"
mv "wasm/${NAME}_bg.wasm" "wasm/myblob_bg.wasm"
