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
use log::error;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

mod proxy_calls;

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

impl Context for HttpHeaders {}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        for (name, value) in &self.get_http_request_headers() {
            info!("#{} -> {}: {}", self.context_id, name, value);
        }

        for (name, value) in &self.get_http_request_headers_bytes() {
            info!("bytes #{} -> {}: {}", self.context_id, name, String::from_utf8_lossy(value).to_string());
        }

        if let Some(intuit_tid_header) = self.get_http_request_header_bytes("intuit_tid") {
            info!("normal check intuit_tid header present");
        } else {
            info!("normal check intuit_tid header missing")
        }

        match proxy_calls::get_header_value_bytes_empty_check("intuit_tid") {
            Ok(res) => {
                if let Some(intuit_tid_header) = res {
                    info!("new-check intuit_tid present");
                    info!("new-check intuit_tid value {}" , String::from_utf8_lossy(&intuit_tid_header).to_string());
                } else {
                    info!("new-check intuit_tid missing");
                }
                
            } Err(status) => {
                error!("new-check error in reading intuit_tid");
            }
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
        // for (name, value) in &self.get_http_response_headers() {
        //     info!("#{} <- {}: {}", self.context_id, name, value);
        // }
        info!("in response flow");
        // self.send_http_response(503, vec![], Some(b"Internal proxy error.\n"));
        Action::Continue
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
