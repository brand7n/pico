use regex::Regex;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Compiled regex pattern — opaque to the caller.
pub struct CompiledRegex {
    re: Regex,
}

// ═══════════════════════════════════════════════════════════════════
// Pattern compilation
// ═══════════════════════════════════════════════════════════════════

/// Compile a regex pattern. Returns an opaque pointer, or null on error.
/// The caller should compile patterns once and reuse them.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_compile(pattern: *const c_char) -> *mut CompiledRegex {
    let pat = match CStr::from_ptr(pattern).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match Regex::new(pat) {
        Ok(re) => Box::into_raw(Box::new(CompiledRegex { re })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a compiled regex.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_free(compiled: *mut CompiledRegex) {
    if !compiled.is_null() {
        drop(Box::from_raw(compiled));
    }
}

// ═══════════════════════════════════════════════════════════════════
// Matching with compiled patterns
// ═══════════════════════════════════════════════════════════════════

/// Match a compiled pattern against subject at offset.
/// Returns the length of the match, or -1 if no match.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_exec(
    compiled: *mut CompiledRegex,
    subject: *const c_char,
    offset: i64,
) -> i64 {
    if compiled.is_null() || subject.is_null() {
        return -1;
    }

    let subject_str = match CStr::from_ptr(subject).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let start = offset.max(0) as usize;
    if start >= subject_str.len() {
        return -1;
    }

    let haystack = &subject_str[start..];
    match (*compiled).re.find(haystack) {
        Some(m) if m.start() == 0 => m.len() as i64,
        _ => -1,
    }
}

/// Match a compiled pattern and return the matched string, or null.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_exec_str(
    compiled: *mut CompiledRegex,
    subject: *const c_char,
    offset: i64,
) -> *mut c_char {
    if compiled.is_null() || subject.is_null() {
        return std::ptr::null_mut();
    }

    let subject_str = match CStr::from_ptr(subject).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let start = offset.max(0) as usize;
    if start >= subject_str.len() {
        return std::ptr::null_mut();
    }

    let haystack = &subject_str[start..];
    match (*compiled).re.find(haystack) {
        Some(m) if m.start() == 0 => CString::new(m.as_str())
            .unwrap_or_default()
            .into_raw(),
        _ => std::ptr::null_mut(),
    }
}

/// Match with capture groups. Returns a Collection of strings (group 0 = full match).
/// Returns null if no match.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_exec_groups(
    compiled: *mut CompiledRegex,
    subject: *const c_char,
    offset: i64,
) -> *mut crate::collection::PicoCollection {
    if compiled.is_null() || subject.is_null() {
        return std::ptr::null_mut();
    }

    let subject_str = match CStr::from_ptr(subject).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let start = offset.max(0) as usize;
    if start >= subject_str.len() {
        return std::ptr::null_mut();
    }

    let haystack = &subject_str[start..];
    let caps = match (*compiled).re.captures(haystack) {
        Some(c) => c,
        None => return std::ptr::null_mut(),
    };

    // Verify the match starts at position 0 (anchored behavior)
    if let Some(m) = caps.get(0) {
        if m.start() != 0 {
            return std::ptr::null_mut();
        }
    }

    let col = crate::collection::PicoCollection::new();
    let col_ptr = Box::into_raw(Box::new(col));

    for i in 0..caps.len() {
        match caps.get(i) {
            Some(m) => {
                let s = CString::new(m.as_str()).unwrap_or_default().into_raw();
                (*col_ptr).push_ptr(s as *mut u8);
            }
            None => {
                (*col_ptr).push_ptr(std::ptr::null_mut());
            }
        }
    }

    col_ptr
}

// ═══════════════════════════════════════════════════════════════════
// Convenience: one-shot matching (compiles pattern each time)
// ═══════════════════════════════════════════════════════════════════

/// One-shot match: compile + match + return length. Convenience for
/// simple cases where pattern reuse isn't needed.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_match(
    pattern: *const c_char,
    subject: *const c_char,
    offset: i64,
) -> i64 {
    let compiled = pico_regex_compile(pattern);
    if compiled.is_null() {
        return -1;
    }
    let result = pico_regex_exec(compiled, subject, offset);
    pico_regex_free(compiled);
    result
}

/// One-shot match returning the matched string, or null.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_match_str(
    pattern: *const c_char,
    subject: *const c_char,
    offset: i64,
) -> *mut c_char {
    let compiled = pico_regex_compile(pattern);
    if compiled.is_null() {
        return std::ptr::null_mut();
    }
    let result = pico_regex_exec_str(compiled, subject, offset);
    pico_regex_free(compiled);
    result
}

/// One-shot match with capture groups.
#[no_mangle]
pub unsafe extern "C" fn pico_regex_match_groups(
    pattern: *const c_char,
    subject: *const c_char,
    offset: i64,
) -> *mut crate::collection::PicoCollection {
    let compiled = pico_regex_compile(pattern);
    if compiled.is_null() {
        return std::ptr::null_mut();
    }
    let result = pico_regex_exec_groups(compiled, subject, offset);
    pico_regex_free(compiled);
    result
}
