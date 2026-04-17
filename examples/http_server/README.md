# WAMR WASI Socket Http Server Demo

This demo runs an echo server on `localhost`.

## Build

```shell
cargo build --target wasm32-wasi --release
```

## Run

```shell
iwasm --addr_pool=0.0.0.0/0 target/wasm32-wasi/release/http_server.wasm
```

## Test

In another terminal window, do the following.

```shell
curl -X POST http://127.0.0.1:1234 -d "name=iwasm"
echo: name=iwasm
```
