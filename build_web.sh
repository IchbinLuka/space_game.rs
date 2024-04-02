#!/bin/bash
cargo build --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/debug/test_game.wasm
cp ./assets ./out/ -r
python3.12 -m http.server 8080 --directory ./out/
