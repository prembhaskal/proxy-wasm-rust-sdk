#[no_mangle]
// pub extern "C" fn on_start(name_ptr: *const u8, name_len: usize, params_ptr: *const u8, params_len: usize) -> i32 {
pub extern "C" fn on_start() -> i32 {
    let name = "on_start";
    let params_len = 1;
    // Convert raw pointers to Rust strings
    // let name = unsafe {
    //     let slice = std::slice::from_raw_parts(name_ptr, name_len);
    //     std::str::from_utf8(slice).unwrap()
    // };
    
    // Dummy processing
    println!("Starting process for: {}", name);
    println!("Parameter length: {}", params_len);
    
    // Return some dummy value
    42
}

#[no_mangle]
// pub extern "C" fn on_stop(name_ptr: *const u8, name_len: usize, params_ptr: *const u8, params_len: usize) -> i32 {
pub extern "C" fn on_stop() -> i32 {
    let name = "on_start";
    let params_len = 1;

    // Convert raw pointers to Rust strings
    // let name = unsafe {
    //     let slice = std::slice::from_raw_parts(name_ptr, name_len);
    //     std::str::from_utf8(slice).unwrap()
    // };
    
    // Dummy processing
    println!("Stopping process for: {}", name);
    println!("Parameter length: {}", params_len);
    
    // Return some dummy value
    24
}