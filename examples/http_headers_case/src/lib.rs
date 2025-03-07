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

        match self.get_http_request_header(":path") {
            Some(path) if path == "/hello" => {
                self.send_http_response(
                    200,
                    vec![("Hello", "World"), ("Powered-BY", "proxy-wasM"), ("lower-case", "val1"), ("content-type", "application/json")],
                    Some(b"{\"Hello\", \"World!\"}"),
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
        self.set_http_response_header("lowercaseh1", Some("v1"));
        self.set_http_response_header("mixeDCasEH2", Some("v2"));
        self.set_http_response_header("UPPERCASEH3", Some("v3"));
        self.set_http_response_header("Access-Control-Expose-Headers", Some("intuit_consent_resource,content-length,intuit_*,x-b3-parentspanid,test-header,intuit_consent_purpose,origin,x-b3-sampled,accept,authorization,intuit-*,tracestate,foo*,x-b3-traceid,x-b3-spanid,traceparent,x-requested-with,x-csrf-token,content-type,location"));
        self.set_http_response_header("Access-Control-Allow-Credentials", Some("true"));
        self.set_http_response_header("Access-Control-Allow-Origin", Some("https://qa2.unit1.turbotaxonline.intuit.com"));
        // self.send_http_response(503, vec![], Some(b"Internal proxy error.\n"));
        Action::Continue
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
