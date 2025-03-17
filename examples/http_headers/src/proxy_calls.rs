
use proxy_wasm::hostcalls;
use proxy_wasm::types::*;
use std::ptr::{null, null_mut};

// Compared to get_http_request_header_bytes, below method will return an empty Vec<u8> when header value is empty, instead of None,
// so we can differentiate between a missing header and empty header.
pub fn get_header_value_bytes_empty_check(key: &str) -> Result<Option<Bytes>, Status> {
    let map_type: proxy_wasm::types::MapType = MapType::HttpRequestHeaders;
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match proxy_get_header_map_value(
            map_type,
            key.as_ptr(),
            key.len(),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => {
                if !return_data.is_null() {
                    Ok(Some(Vec::from_raw_parts(
                        return_data,
                        return_size,
                        return_size,
                    )))
                } else {
                    Ok(Some(Vec::new()))
                }
            }
            Status::NotFound => Ok(None),
            status => panic!("unexpected status: {}", status as u32),
        }
    }
}


pub fn get_map_value_empty_check(key: &str) -> Result<Option<String>, Status> {
    let map_type: proxy_wasm::types::MapType = MapType::HttpRequestHeaders;
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match proxy_get_header_map_value(
            map_type,
            key.as_ptr(),
            key.len(),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => {
                if !return_data.is_null() {
                    Ok(Some(
                        String::from_utf8(Vec::from_raw_parts(
                            return_data,
                            return_size,
                            return_size,
                        ))
                        .unwrap(),
                    ))
                } else {
                    Ok(Some(String::new()))
                }
            }
            Status::NotFound => Ok(None),
            status => panic!("unexpected status: {}", status as u32),
        }
    }
}

extern "C" {
    fn proxy_get_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
}