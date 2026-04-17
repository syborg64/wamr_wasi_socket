# WAMR WASI Socket

Here are some examples for running network socket programs in wasm-micro-runtime. The applications are written in Rust.

## Prerequisites

You need to install [Rust](https://www.rust-lang.org/tools/install) and [iwasm](https://github.com/bytecodealliance/wasm-micro-runtime/tree/main/product-mini) before you try to compile and run the following examples.

## An HTTP client example

[See here](http_client/README.md)

## An non-blocking HTTP client example

[See here](nonblock_http_client)

## An HTTP server example

[See here](http_server/README.md)

## An HTTP server example with poll

[See here](poll_http_server)

## TCP Stream Example with WAMR

This is a example of using WAMR as a socket client.

```
cargo run --example tcp_stream
```

Set up a server on your localhost with [ncat](https://nmap.org/ncat).

```
ncat -kvlp 1234
```

run the wasm with WAMR's iwasm. iwasm would send message "hello" to a server at `localhost:1234`.

```
$ iwasm --env PORT=1234 --addr-pool=127.0.0.1/32 <path-to-wamr_wasi_socket>/target/wasm32-unknown-unknown/debug/examples/tcp_stream.wasm
connect to 127.0.0.1:1234
sending hello message...
```

The server should get the message "hello".

```
$ ncat -kvlp 1234 
Ncat: Version 7.91 ( https://nmap.org/ncat )
Ncat: Listening on :::1234
Ncat: Listening on 0.0.0.0:1234
Ncat: Connection from 127.0.0.1.
Ncat: Connection from 127.0.0.1:56366.
hello
```

## TCP Listener Example with WAMR

This is a example of using WAMR as a socket server.

```
cargo run --example tcp_listener
```

Set up a client on your localhost with [ncat](https://nmap.org/ncat).

Send any message, then send EOF with <ctrl+D>. The server would send back the reversed message.

For example, if the client send message "hello", the client would receive the response "olleh".

```
$ ncat -v 127.0.0.1 1234
Ncat: Version 7.91 ( https://nmap.org/ncat )
Ncat: Connected to 127.0.0.1:1234.
hello

olleh
```

