#!/bin/bash

echo ">> Building contract"

set -e

rustup target add wasm32-unknown-unknown
RUSTFLAGS='-C link-arg=-s' cargo build --package "$1" --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/*.wasm ./res/