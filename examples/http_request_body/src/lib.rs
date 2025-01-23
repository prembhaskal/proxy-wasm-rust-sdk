// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders { context_id, content_length: 0, cached_body: None }))
    }
}

struct HttpHeaders {
    context_id: u32,
    content_length: usize,
    cached_body: Option<Vec<u8>>,
}

impl Context for HttpHeaders {}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        // for (name, value) in &self.get_http_request_headers() {
        //     info!("#{} -> {}: {}", self.context_id, name, value);
        // }

        match self.get_http_request_header(":path") {
            Some(path) if path == "/hello" => {
                self.send_http_response(
                    200,
                    vec![("Hello", "World"), ("Powered-By", "proxy-wasm")],
                    Some(b"Hello, World!\n"),
                );
                Action::Pause
            }
            _ => Action::Continue,
        }
    }

    // When we pause before end_of_stream, then we don't have to accumulate body_size, envoy does this automatically.
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        // if !end_of_stream {
            self.content_length += body_size;
            // info!("In method on_http_request_body, body_size: {}, end_of_stream: {}, content_length: {}", body_size, end_of_stream, self.content_length);
            // self.cached_body = self.get_http_request_body(0, self.content_length);
            // return Action::Continue;
        // }
        info!("In method on_http_request_body, body_size: {}, end_of_stream: {}, content_length: {}", body_size, end_of_stream, self.content_length);

        // append to cached_body instead of overwriting
        if let Some(ref mut cached_body) = self.cached_body {
            if let Some(new_body) = self.get_http_request_body(self.content_length - body_size, body_size) {
                cached_body.extend_from_slice(&new_body);
            }
        } else {
            self.cached_body = self.get_http_request_body(0, self.content_length);
        }
        self.cached_body = self.get_http_request_body(0, body_size);
        // self.cached_body = self.get_http_request_body(0, self.content_length);

        if let Some(ref body) = self.cached_body {
            info!("Cached body length: {}", body.len());
        } else {
            info!("No body cached");
        }
        return Action::Continue;
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // for (name, value) in &self.get_http_response_headers() {
        //     info!("#{} <- {}: {}", self.context_id, name, value);
        // }
        Action::Continue
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
