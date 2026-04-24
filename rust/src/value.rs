use std::os::raw::c_char;

/// Tagged value for heterogeneous storage.
///
/// Layout: { i64 tag, u64 bits } = 16 bytes
/// Tags: 0=null, 1=int, 2=bool, 3=float, 4=string(ptr), 5=object(ptr)
///
/// Accessors panic on tag mismatch to catch type errors immediately.
#[repr(C)]
pub struct PicoValue {
    tag: i64,
    bits: u64,
}

const TAG_NULL: i64 = 0;
const TAG_INT: i64 = 1;
const TAG_BOOL: i64 = 2;
const TAG_FLOAT: i64 = 3;
const TAG_STRING: i64 = 4;
const TAG_OBJECT: i64 = 5;

fn tag_name(tag: i64) -> &'static str {
    match tag {
        TAG_NULL => "NULL",
        TAG_INT => "INT",
        TAG_BOOL => "BOOL",
        TAG_FLOAT => "FLOAT",
        TAG_STRING => "STRING",
        TAG_OBJECT => "OBJECT",
        _ => "UNKNOWN",
    }
}

impl PicoValue {
    fn boxed(tag: i64, bits: u64) -> *mut PicoValue {
        Box::into_raw(Box::new(PicoValue { tag, bits }))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Constructors
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn pico_value_none() -> *mut PicoValue {
    PicoValue::boxed(TAG_NULL, 0)
}

#[no_mangle]
pub extern "C" fn pico_value_from_int(val: i64) -> *mut PicoValue {
    PicoValue::boxed(TAG_INT, val as u64)
}

#[no_mangle]
pub extern "C" fn pico_value_from_bool(val: i64) -> *mut PicoValue {
    PicoValue::boxed(TAG_BOOL, (val != 0) as u64)
}

#[no_mangle]
pub extern "C" fn pico_value_from_float(val: f64) -> *mut PicoValue {
    PicoValue::boxed(TAG_FLOAT, val.to_bits())
}

#[no_mangle]
pub extern "C" fn pico_value_from_string(val: *mut c_char) -> *mut PicoValue {
    PicoValue::boxed(TAG_STRING, val as u64)
}

#[no_mangle]
pub extern "C" fn pico_value_from_object(val: *mut u8) -> *mut PicoValue {
    PicoValue::boxed(TAG_OBJECT, val as u64)
}

// ═══════════════════════════════════════════════════════════════════
// Tag query
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_value_tag(val: *const PicoValue) -> i64 {
    if val.is_null() {
        return TAG_NULL;
    }
    (*val).tag
}

// ═══════════════════════════════════════════════════════════════════
// Accessors (panic on tag mismatch)
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_value_as_int(val: *const PicoValue) -> i64 {
    assert!(
        !val.is_null() && (*val).tag == TAG_INT,
        "pico_value_as_int: expected INT, got {}",
        tag_name(if val.is_null() { -1 } else { (*val).tag })
    );
    (*val).bits as i64
}

#[no_mangle]
pub unsafe extern "C" fn pico_value_as_bool(val: *const PicoValue) -> i64 {
    assert!(
        !val.is_null() && (*val).tag == TAG_BOOL,
        "pico_value_as_bool: expected BOOL, got {}",
        tag_name(if val.is_null() { -1 } else { (*val).tag })
    );
    (*val).bits as i64
}

#[no_mangle]
pub unsafe extern "C" fn pico_value_as_float(val: *const PicoValue) -> f64 {
    assert!(
        !val.is_null() && (*val).tag == TAG_FLOAT,
        "pico_value_as_float: expected FLOAT, got {}",
        tag_name(if val.is_null() { -1 } else { (*val).tag })
    );
    f64::from_bits((*val).bits)
}

#[no_mangle]
pub unsafe extern "C" fn pico_value_as_string(val: *const PicoValue) -> *mut c_char {
    assert!(
        !val.is_null() && (*val).tag == TAG_STRING,
        "pico_value_as_string: expected STRING, got {}",
        tag_name(if val.is_null() { -1 } else { (*val).tag })
    );
    (*val).bits as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn pico_value_as_object(val: *const PicoValue) -> *mut u8 {
    assert!(
        !val.is_null() && (*val).tag == TAG_OBJECT,
        "pico_value_as_object: expected OBJECT, got {}",
        tag_name(if val.is_null() { -1 } else { (*val).tag })
    );
    (*val).bits as *mut u8
}
