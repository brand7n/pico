use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Helper: C string pointer → Rust &str (panics on null or invalid UTF-8)
unsafe fn cstr_to_str<'a>(s: *const c_char) -> &'a str {
    assert!(!s.is_null(), "pico_string: null pointer");
    CStr::from_ptr(s).to_str().unwrap_or("")
}

/// Helper: Rust String → heap-allocated C string pointer (caller owns)
fn str_to_cstr(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

// ── Length / char access ───────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn pico_string_length(s: *const c_char) -> i32 {
    if s.is_null() {
        return 0;
    }
    CStr::from_ptr(s).to_bytes().len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_char_at(s: *const c_char, index: i32) -> i32 {
    let bytes = CStr::from_ptr(s).to_bytes();
    let i = index as usize;
    if i < bytes.len() {
        bytes[i] as i32
    } else {
        -1
    }
}

// ── Substring ──────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn pico_string_substring(
    s: *const c_char,
    start: i32,
    length: i32,
) -> *mut c_char {
    let src = cstr_to_str(s);
    let start = start.max(0) as usize;

    let result = if length < 0 {
        if start < src.len() {
            &src[start..]
        } else {
            ""
        }
    } else {
        let end = (start + length as usize).min(src.len());
        if start < src.len() {
            &src[start..end]
        } else {
            ""
        }
    };

    str_to_cstr(result)
}

// ── Search ─────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn pico_string_index_of(
    s: *const c_char,
    needle: *const c_char,
    offset: i32,
) -> i32 {
    let haystack = cstr_to_str(s);
    let needle = cstr_to_str(needle);
    let offset = offset.max(0) as usize;

    if offset >= haystack.len() {
        return -1;
    }

    match haystack[offset..].find(needle) {
        Some(pos) => (pos + offset) as i32,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_starts_with(
    s: *const c_char,
    prefix: *const c_char,
) -> i32 {
    let s = cstr_to_str(s);
    let prefix = cstr_to_str(prefix);
    s.starts_with(prefix) as i32
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_ends_with(
    s: *const c_char,
    suffix: *const c_char,
) -> i32 {
    let s = cstr_to_str(s);
    let suffix = cstr_to_str(suffix);
    s.ends_with(suffix) as i32
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_contains(
    s: *const c_char,
    needle: *const c_char,
) -> i32 {
    let s = cstr_to_str(s);
    let needle = cstr_to_str(needle);
    s.contains(needle) as i32
}

// ── Concat / Transform ────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn pico_string_concat(
    a: *const c_char,
    b: *const c_char,
) -> *mut c_char {
    let a = cstr_to_str(a);
    let b = cstr_to_str(b);
    let mut result = String::with_capacity(a.len() + b.len());
    result.push_str(a);
    result.push_str(b);
    str_to_cstr(&result)
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_equals(
    a: *const c_char,
    b: *const c_char,
) -> i32 {
    if a.is_null() && b.is_null() {
        return 1;
    }
    if a.is_null() || b.is_null() {
        return 0;
    }
    let a = CStr::from_ptr(a);
    let b = CStr::from_ptr(b);
    (a == b) as i32
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_trim(s: *const c_char) -> *mut c_char {
    let s = cstr_to_str(s);
    str_to_cstr(s.trim())
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_to_lower(s: *const c_char) -> *mut c_char {
    let s = cstr_to_str(s);
    str_to_cstr(&s.to_lowercase())
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_to_upper(s: *const c_char) -> *mut c_char {
    let s = cstr_to_str(s);
    str_to_cstr(&s.to_uppercase())
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_replace(
    s: *const c_char,
    search: *const c_char,
    replace: *const c_char,
) -> *mut c_char {
    let s = cstr_to_str(s);
    let search = cstr_to_str(search);
    let replace = cstr_to_str(replace);
    str_to_cstr(&s.replace(search, replace))
}

#[no_mangle]
pub unsafe extern "C" fn pico_string_split(
    s: *const c_char,
    delimiter: *const c_char,
) -> *mut crate::collection::PicoCollection {
    let s = cstr_to_str(s);
    let delim = cstr_to_str(delimiter);
    let col = crate::collection::PicoCollection::new();
    let col_ptr = Box::into_raw(Box::new(col));

    for part in s.split(delim) {
        let part_cstr = str_to_cstr(part);
        (*col_ptr).push_ptr(part_cstr as *mut u8);
    }

    col_ptr
}

// ── Conversion ─────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn pico_string_to_int(s: *const c_char) -> i32 {
    let s = cstr_to_str(s);
    s.trim().parse::<i32>().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn pico_int_to_string(val: i32) -> *mut c_char {
    str_to_cstr(&val.to_string())
}

#[no_mangle]
pub extern "C" fn pico_float_to_string(val: f64) -> *mut c_char {
    // Match PHP's default float formatting
    if val == val.floor() && val.abs() < 1e15 {
        str_to_cstr(&format!("{:.0}", val))
    } else {
        str_to_cstr(&format!("{}", val))
    }
}

// ── Format ─────────────────────────────────────────────────────────

/// Simple {} placeholder format. Arguments are passed as an array of
/// C string pointers (each pre-converted to string by the caller).
#[no_mangle]
pub unsafe extern "C" fn pico_string_format(
    template: *const c_char,
    args: *const *const c_char,
    arg_count: i32,
) -> *mut c_char {
    let template = cstr_to_str(template);
    let mut result = String::with_capacity(template.len() + 64);
    let mut arg_idx = 0usize;
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'}') {
            chars.next(); // consume '}'
            if arg_idx < arg_count as usize && !args.is_null() {
                let arg_ptr = *args.add(arg_idx);
                if !arg_ptr.is_null() {
                    result.push_str(cstr_to_str(arg_ptr));
                } else {
                    result.push_str("null");
                }
            } else {
                result.push_str("{}");
            }
            arg_idx += 1;
        } else {
            result.push(ch);
        }
    }

    str_to_cstr(&result)
}
