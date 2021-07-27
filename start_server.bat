cargo build --target wasm32-unknown-unknown --release
copy .\target\wasm32-unknown-unknown\release\cybersub.wasm .\docs\cybersub.wasm
basic-http-server docs
