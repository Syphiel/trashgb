#!/bin/bash
set -euo pipefail

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir ./www target/wasm32-unknown-unknown/release/trashgb.wasm
wasm-snip --snip-rust-panicking-code --snip-rust-fmt-code ./www/trashgb_bg.wasm -o ./www/trashgb_bg.wasm
wasm-strip ./www/trashgb_bg.wasm
wasm-opt -Oz ./www/trashgb_bg.wasm -o ./www/trashgb_bg.wasm
