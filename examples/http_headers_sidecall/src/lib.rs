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
use std::collections::HashMap;
use std::time::Duration;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
    info!("main invoked");
}}
// // #[no_mangle]
// pub fn _start() {
//     proxy_wasm::set_log_level(LogLevel::Trace);
//     proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
//     info!("_started invoked");
// }

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        info!("create_http_context invoked for id: {}", context_id);
        Some(Box::new(HttpHeaders {
            context_id,
            call_count: 0,
            total_request_body_size: 0,
            request_headers: HashMap::new(),
            original_body: None,
        }))
    }
}

struct HttpHeaders {
    context_id: u32,
    call_count: u32,
    total_request_body_size: usize,
    request_headers: HashMap<String, String>,
    original_body: Option<Vec<u8>>,
}

impl Context for HttpHeaders {
    fn on_http_call_response(&mut self, token_id: u32, _: usize, body_size: usize, _: usize) {
        info!("on_http_call_response {}", token_id);
        let resp_headers = self.get_http_call_response_headers();
        info!("on_http_call_response headers: {:?}", resp_headers);

        self.resume_http_request();
    }
}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        info!("#{} entered on_http_request_headers", self.context_id);

        for (name, value) in &self.get_http_request_headers() {
            info!("#{} -> {}: {}", self.context_id, name, value);
        }
        match self.dispatch_http_call(
            "defaultcluster",
            vec![
                (
                    ":method",
                    self.request_headers
                        .get(":method")
                        .unwrap_or(&"GET".to_string()),
                ),
                (
                    ":path",
                    self.request_headers
                        .get(":path")
                        .unwrap_or(&"/post".to_string()),
                ),
                (
                    ":content-type",
                    self.request_headers
                        .get(":content-type")
                        .unwrap_or(&"".to_string()),
                ),
                (
                    ":accept",
                    self.request_headers
                        .get(":accept")
                        .unwrap_or(&"".to_string()),
                ),
                (
                    ":authority",
                    self.request_headers
                        .get(":authority")
                        .unwrap_or(&"localhost:10000".to_string()),
                ),
            ],
            None,
            vec![],
            Duration::from_secs(1),
        ) {
            Ok(b) => {
                info!("#{} HTTP call dispatched: {}", self.context_id, b);
            }
            Err(err) => {
                info!("#{} HTTP call dispatch failed: {:?}", self.context_id, err);
                return Action::Continue;
            }
        };

        Action::Pause
        
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
