# WAMR WASI Socket Http Client Demo

## Build

```shell
cargo build --target wasm32-wasi --release
```

## Run

```shell
iwasm --addr_pool=0.0.0.0/0 target/wasm32-wasi/release/http_client.wasm
```
