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
        let mut call_count = 0;
        Some(Box::new(HttpHeaders { context_id, call_count}))
    }
}

struct HttpHeaders {
    context_id: u32,
    call_count: u32,
}

impl Context for HttpHeaders {
    fn on_http_call_response(&mut self, token_id: u32, _: usize, body_size: usize, _: usize) {
        info!("got response for call {}", token_id);
        let resp_headers = self.get_http_call_response_headers();

        let mut status_code = 0;
        for (name, value) in resp_headers.iter() {
            if name == ":status" {
                status_code = value.parse().unwrap_or(0);
            }
            info!("#{} http call response {}: {}", self.context_id, name, value)
        }

        let response_body = self.get_http_call_response_body(0, body_size);
        if let Some(body) = &response_body {
            info!("Response body: {:?}", String::from_utf8_lossy(body));
        }

        if self.call_count == 2 {
            // final clusterb call
            info!("sending cluster b response back");
            self.send_http_response(
                status_code as u32,
                resp_headers.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect(),
                Some(response_body.as_deref().unwrap_or(&[])),
            );

            // self.resume_http_response();
        }


        if self.call_count == 1 {
            self.call_count+=1;

            let json_data = r#"{
                "data": "some data"
            }"#;
            let json_bytes: &[u8] = json_data.as_bytes();

            // make one more call
            info!("#{} retrying request to new cluster", self.context_id);
            self.dispatch_http_call(
                "clusterb",
                vec![
                    (":method", "POST"),
                    (":path", "/post"),
                    (":authority", "localhost:10000"),
                    (":content-type", "application/json"),
                    (":accept", "application/json"),
                    ],
                    Some(json_bytes),
                    vec![],
                    Duration::from_secs(1),
                ).unwrap_or_else(|err| {
                    info!("#{} HTTP call failed: {:?}", self.context_id, err);
                    0
                });
            }

        // TODO overwrite response headers
        // TODO overwrite response body in another flow.
        // self.resume_http_response();

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

        // This will get the upstream cluster name where the request will be sent
        match self.get_property(vec!["cluster_name"]) {
            Some(cluster_name_bytes) => {
                if let Ok(cluster_name) = String::from_utf8(cluster_name_bytes) {
                    log::info!("Request will be sent to upstream cluster: {}", cluster_name);
                }
            },
        _ => log::error!("Failed to get upstream cluster name"),
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
        let mut upstream = String::from("defaultcluster");
        for (name, value) in &self.get_http_response_headers() {
            info!("#{} response headers <- {}: {}", self.context_id, name, value);
            if name == ":status" {
                status_code = value.parse().unwrap_or(0);
            } else if name == ":freeform" {
                upstream = value.to_string();
            }
        }

        // TODO - we cannot get request headers here, need to cache them during request flow. (Later)

        info!("cluster {} invoked", upstream);

        if status_code == 302 {
            info!("#{} calling gateway again", self.context_id);
            self.call_count+=1;
            self.dispatch_http_call(
                "clustera",
                vec![
                    (":method", "GET"),
                    (":path", "/status/200"),
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
