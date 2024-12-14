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
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
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
        info!("got response for call {}", token_id);
        let resp_headers = self.get_http_call_response_headers();

        let mut status_code = 0;
        for (name, value) in resp_headers.iter() {
            if name == ":status" {
                status_code = value.parse().unwrap_or(0);
            }
            info!(
                "#{} http call response {}: {}",
                self.context_id, name, value
            )
        }

        let response_body = self.get_http_call_response_body(0, body_size);
        if let Some(body) = &response_body {
            info!("Response body: {:?}", String::from_utf8_lossy(body));
        }

        if self.call_count > 2 {
            panic!("incorrect state - call_count {}", self.call_count);
        }

        if self.call_count == 2 {
            // final clusterb call
            info!("sending cluster b response back");
            self.send_http_response(
                status_code as u32,
                resp_headers
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect(),
                Some(response_body.as_deref().unwrap_or(&[])),
            );
        }

        if self.call_count == 1 {
            self.call_count += 1;

            let json_bytes = self.original_body.as_deref().unwrap_or(&[]);

            // let json_data = r#"{
            //     "data": "some data"
            // }"#;
            // let json_bytes: &[u8] = json_data.as_bytes();

            // make one more call
            info!(
                "#{} retrying original request to new cluster",
                self.context_id
            );
            self.dispatch_http_call(
                "clusterb",
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
                        ":authority",
                        self.request_headers
                            .get(":authority")
                            .unwrap_or(&"localhost:10000".to_string()),
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
                ],
                Some(json_bytes),
                vec![],
                Duration::from_secs(1),
            )
            .unwrap_or_else(|err| {
                info!("#{} HTTP call failed: {:?}", self.context_id, err);
                0
            });
        }
    }
}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        info!("#{} entered on_http_request_headers", self.context_id);

        for (name, value) in &self.get_http_request_headers() {
            info!("#{} -> {}: {}", self.context_id, name, value);
        }

        self.request_headers = self
            .get_http_request_headers()
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        // This will get the upstream cluster name where the request will be sent
        match self.get_property(vec!["cluster_name"]) {
            Some(cluster_name_bytes) => {
                if let Ok(cluster_name) = String::from_utf8(cluster_name_bytes) {
                    log::info!("Request will be sent to upstream cluster: {}", cluster_name);
                }
            }
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

    // refer https://tetrate.io/blog/validating-a-request-payload-with-wasm/
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.total_request_body_size += body_size;
        // TODO - add flag to check if this is a DR request, only then buffer body.
        if !end_of_stream {
            return Action::Pause; // wait until we get whole body.
        }

        self.original_body = self.get_http_request_body(0, self.total_request_body_size);
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        let mut status_code: u32 = 0;
        let mut upstream = String::from("defaultcluster");
        for (name, value) in &self.get_http_response_headers() {
            info!(
                "#{} response headers <- {}: {}",
                self.context_id, name, value
            );
            if name == ":status" {
                status_code = value.parse().unwrap_or(0);
            } else if name == ":freeform" {
                upstream = value.to_string();
            }
        }

        info!("cluster {} invoked", upstream);

        if status_code == 302 {
            info!("#{} calling gateway again", self.context_id);
            self.call_count += 1;
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
            )
            .unwrap_or_else(|err| {
                info!("#{} HTTP call failed: {:?}", self.context_id, err);
                0
            });
            Action::Pause // don't complete original request, make a new call.
        } else {
            Action::Continue
        }
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
