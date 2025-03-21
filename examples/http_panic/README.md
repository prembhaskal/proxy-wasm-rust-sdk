## Proxy-Wasm plugin example: HTTP headers

Proxy-Wasm plugin that logs HTTP request/response headers.

### Building

```sh
$ rustup toolchain install nightly
$ rustup target add wasm32-wasip1
$ cargo build --target wasm32-wasip1 --release
$ cp target/wasm32-wasip1/release/proxy_wasm_example_http_panic.wasm /tmp/

$ # assuming envoy installed locally, Also tested from company fork of envoy.
$ envoy -c envoy-local.yaml --log-level info --concurrency 1
$ # for envoy 1.33 onwards, by default wasm is disabled, need to build locally.
$ /tmp/envoy_133 -c ./envoy-local-reload.yaml --concurrency 1


$ # make normal calls
$ curl -v http://localhost:10000/hello

$ # curl request to cause panic
$ curl -v http://localhost:10000/panic

$ # check wasm still works for other request
$ curl -v http://localhsot:10000/hello

```

### VM reload
- Needs envoy >= 1.33
- Add field - failure_policy: FAIL_RELOAD, this will by default reload VM, with backoff of 1s

### Using in Envoy

This example can be run with [`docker compose`](https://docs.docker.com/compose/install/)
and has a matching Envoy configuration.

```sh
$ docker compose up
```

Send HTTP request to `localhost:10000/hello`:

```sh
$ curl localhost:10000/hello
Hello, World!
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
