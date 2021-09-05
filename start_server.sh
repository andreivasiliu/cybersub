#!/bin/bash
set -eu

# Starts a local web-server that serves the contents of the `doc/` folder,
# which is the folder to where the web version is compiled.

# cargo install basic-http-server

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/cybersub.wasm docs/cybersub.wasm

echo "Open: http://localhost:8080"

~/.cargo/bin/basic-http-server -a 0.0.0.0:8080 docs
