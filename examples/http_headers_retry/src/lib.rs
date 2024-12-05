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
use std::iter;
use std::time::Duration;

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
        Some(Box::new(HttpHeaders { context_id }))
    }
}

struct HttpHeaders {
    context_id: u32,
}

impl Context for HttpHeaders {
    fn on_http_call_response(&mut self, token_id: u32, _: usize, _: usize, _: usize) {
        info!("got response for call {}", token_id);
        let resp_headers = self.get_http_call_response_headers();

        for (name, value) in resp_headers.iter() {
            info!("#{} http call response {}: {}", self.context_id, name, value)
        }

        self.resume_http_response();

        // if let Some(body) = self.get_http_call_response_body(0, body_size) {
        //     if !body.is_empty() && body[0] % 2 == 0 {
        //         info!("Access granted.");
        //         self.resume_http_request();
        //         return;
        //     }
        // }
    }
}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        info!("#{} entered on_http_request_headers", self.context_id);

        for (name, value) in &self.get_http_request_headers() {
            info!("#{} -> {}: {}", self.context_id, name, value);
        }

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

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        let mut status_code: u32 = 0;
        for (name, value) in &self.get_http_response_headers() {
            info!("#{} response headers <- {}: {}", self.context_id, name, value);
            if name == ":status" {
                status_code = value.parse().unwrap_or(0);
            }
        }

        // TODO - we cannot get request headers here, need to cache them during request flow.
        // let actual_headers = self.get_http_request_headers();
        // info!("len of http request headers: {}", actual_headers.len());
        // let mut retry_request_headers :Vec<(&str, &str)> = Vec::new();
        // for (name, value) in actual_headers.iter() {
        //     info!("#{} retry headers <- {}: {}", self.context_id, name, value);
        //     retry_request_headers.push((name, value));
        // }

        // self.dispatch_http_call(
        //     "ppp",
        //     retry_request_headers,
        //     None,
        //     vec![],
        //     Duration::from_secs(10),
        // )
        // .unwrap_or_else(|err| {
        //     info!("#{} HTTP call failed: {:?}", self.context_id, err);
        //     0 // Return a valid u32 value
        // });


        if status_code == 302 {
            info!("#{} retrying request", self.context_id);
            self.dispatch_http_call(
                "mycustomhttpbin",
                vec![
                    (":method", "GET"),
                    (":path", "/hello"),
                    (":authority", "localhost:10000"),
                    ],
                    None,
                    vec![],
                    Duration::from_secs(1),
                ).unwrap_or_else(|err| {
                    info!("#{} HTTP call failed: {:?}", self.context_id, err);
                    0
                });
                Action::Pause // don't complete original request, make a new call.
            }
             else {
                Action::Continue
             }
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
