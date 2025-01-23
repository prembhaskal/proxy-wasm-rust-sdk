## Proxy-Wasm plugin example: HTTP headers

Proxy-Wasm plugin that cached HTTP request body.
If we pause the processing, the request body is buffered and then we can copy whole body at the end.

### Building

```sh
% rustup toolchain install nightly
% rustup target add wasm32-wasip1
% cargo build --target wasm32-wasip1 --release
% cp target/wasm32-wasip1/release/proxy_wasm_example_http_request_body.wasm /tmp/
```

### Using in Envoy

This example can be run with [`docker compose`](https://docs.docker.com/compose/install/)
and has a matching Envoy configuration.

```sh
% envoy -c envoy-local.yaml --log-level info
```

Send a POST request with a temporary file
```sh
% dd if=/dev/urandom of=/tmp/random.bin bs=500K count=1
% cat /tmp/random.bin | base64  > /tmp/random.out            
% curl -X POST http://localhost:10000/post -d@/tmp/random.out
```


Expected Envoy logs:
```console
[info][wasm] [...] wasm log http_headers: In method on_http_request_body, body_size: 393060, end_of_stream: false, content_length: 393060
[info][wasm] [...] wasm log http_headers: In method on_http_request_body, body_size: 393060, end_of_stream: false, content_length: 786120
[info][wasm] [...] wasm log http_headers: In method on_http_request_body, body_size: 682668, end_of_stream: true, content_length: 786120
[info][wasm] [...] wasm log http_headers: Cached body length: 682668
[info][wasm] [...] wasm log http_headers: #2 completed.
```
