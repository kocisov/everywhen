default:
  @just --list

build-wasm mode="debug":
  cargo build --{{mode}} --target wasm32-wasi

run-wasmtime mode="debug":
  wasmtime target/wasm32-wasi/{{mode}}/everywhen.wasm
