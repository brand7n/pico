use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn pico_file_read(path: *const c_char) -> *mut c_char {
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match std::fs::read_to_string(path_str) {
        Ok(contents) => CString::new(contents).unwrap_or_default().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pico_file_write(path: *const c_char, data: *const c_char) {
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let data_str = match CStr::from_ptr(data).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    let _ = std::fs::write(path_str, data_str);
}

#[no_mangle]
pub unsafe extern "C" fn pico_file_exists(path: *const c_char) -> i32 {
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    std::path::Path::new(path_str).exists() as i32
}

#[no_mangle]
pub unsafe extern "C" fn pico_file_is_file(path: *const c_char) -> i32 {
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    std::path::Path::new(path_str).is_file() as i32
}

#[no_mangle]
pub unsafe extern "C" fn pico_file_is_dir(path: *const c_char) -> i32 {
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    std::path::Path::new(path_str).is_dir() as i32
}
