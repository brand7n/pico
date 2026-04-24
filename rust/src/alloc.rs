use std::alloc::{Layout, alloc_zeroed};

/// Allocate a zeroed block of `size` bytes for a struct with the given type_id.
/// The caller stores type_id at offset 0.
///
/// Returns a pointer to the allocated memory. For now this is a simple
/// malloc-style allocator. A tracing GC can replace this later — the
/// interface stays the same.
#[no_mangle]
pub unsafe extern "C" fn picohp_object_alloc(size: u64, _type_id: i32) -> *mut u8 {
    let size = size.max(8) as usize; // minimum 8 bytes (one field)
    let align = 8; // 8-byte alignment for all structs

    let layout = match Layout::from_size_align(size, align) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };

    let ptr = alloc_zeroed(layout);
    if ptr.is_null() {
        // OOM — abort
        std::process::abort();
    }

    ptr
}

/// Runtime version — can be used by compiled programs to check compatibility.
#[no_mangle]
pub extern "C" fn pico_rt_version() -> i32 {
    2 // v0.2 — matches spec version
}
