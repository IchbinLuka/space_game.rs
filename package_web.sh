#!/bin/bash

if [[ $1 == "release" ]] ; then
    echo "Building in release mode..."
    MODE="release"
    cargo build --target wasm32-unknown-unknown --release
elif [[ $1 == "debug" ]] ; then
    echo "Building in debug mode..."
    MODE="debug"
    cargo build --target wasm32-unknown-unknown
else 
    echo "Usage: $0 <release|debug>"
    exit 1
fi

echo "Generating wasm bindings..."
wasm-bindgen --no-typescript --out-dir ./out/ --out-name "space_game" --target web ./target/wasm32-unknown-unknown/$MODE/test_game.wasm

echo "Optimizing wasm..."
wasm-opt -O ./out/space_game_bg.wasm -o ./out/space_game_bg.wasm

echo "Copying assets..."
rm ./out/assets -r
cp ./assets ./out/ -r