## Proxy-Wasm plugin example: HTTP headers case

Checking if we can preserve header case for local replies.

### Building

```sh
$ cargo build --target wasm32-wasip1 --release
$ cp target/wasm32-wasip1/release/proxy_wasm_example_http_headers_case.wasm /tmp/
```

### Using in Envoy

This example can be run with [`docker compose`](https://docs.docker.com/compose/install/)
and has a matching Envoy configuration.

```sh
$ docker compose up
```

Send HTTP request to `localhost:10000/hello`:

```sh
$ curl -v http://localhost:10000/hello
Hello, World!

$ curl -v "http://localhost:10000/response-headers?Freeform=key1" -H "accept: application/json"
```

Expected Envoy logs:

```console
[...] wasm log http_headers: #2 -> :authority: localhost:10000
[...] wasm log http_headers: #2 -> :path: /hello
[...] wasm log http_headers: #2 -> :method: GET
[...] wasm log http_headers: #2 -> :scheme: http
[...] wasm log http_headers: #2 -> user-agent: curl/7.81.0
[...] wasm log http_headers: #2 -> accept: */*
[...] wasm log http_headers: #2 -> x-forwarded-proto: http
[...] wasm log http_headers: #2 -> x-request-id: 3ed6eb3b-ddce-4fdc-8862-ddb8f168d406
[...] wasm log http_headers: #2 <- :status: 200
[...] wasm log http_headers: #2 <- hello: World
[...] wasm log http_headers: #2 <- powered-by: proxy-wasm
[...] wasm log http_headers: #2 <- content-length: 14
[...] wasm log http_headers: #2 <- content-type: text/plain
[...] wasm log http_headers: #2 completed.
```
