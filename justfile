default:
  @just --list

build *FLAGS:
  cargo build {{FLAGS}} --target wasm32-wasi

run mode="debug":
  wasmtime target/wasm32-wasi/{{mode}}/everywhen.wasm

run-example:
  cat example.js | just run release

build-run:
  just build --release && just run-example
