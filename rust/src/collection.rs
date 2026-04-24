use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Internal value storage. Every value is stored as 64 bits.
/// The caller (compiler-generated code) knows the type and calls
/// the appropriate get/set function.
///
/// With Pico int = i64, int and ptr are both 64 bits — from_int/as_int
/// are identity operations (no truncation or extension).
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Slot {
    bits: u64,
}

impl Slot {
    fn from_int(v: i64) -> Self {
        Slot { bits: v as u64 }
    }
    fn from_ptr(v: *mut u8) -> Self {
        Slot { bits: v as u64 }
    }
    fn as_int(self) -> i64 {
        self.bits as i64
    }
    fn as_ptr(self) -> *mut u8 {
        self.bits as *mut u8
    }
}

/// The collection: ordered entries with both integer and string key access.
///
/// Sequential entries (push/getAt/setAt) are stored in a Vec.
/// String-keyed entries (get/set/has) are stored in a HashMap + insertion-order Vec.
///
/// The two spaces are independent — a collection can have both sequential
/// and string-keyed entries simultaneously (like PHP arrays).
pub struct PicoCollection {
    /// Sequential storage (integer-keyed: 0, 1, 2, ...)
    seq: Vec<Slot>,

    /// String-keyed storage: maps key → slot
    map: HashMap<String, Slot>,

    /// String key insertion order (for keyAt)
    key_order: Vec<String>,
}

#[allow(dead_code)]
impl PicoCollection {
    pub fn new() -> Self {
        PicoCollection {
            seq: Vec::new(),
            map: HashMap::new(),
            key_order: Vec::new(),
        }
    }

    pub fn push_int(&mut self, val: i64) {
        self.seq.push(Slot::from_int(val));
    }

    pub fn push_ptr(&mut self, val: *mut u8) {
        self.seq.push(Slot::from_ptr(val));
    }
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Constructor / count
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn pico_collection_new() -> *mut PicoCollection {
    Box::into_raw(Box::new(PicoCollection::new()))
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_count(col: *mut PicoCollection) -> i64 {
    if col.is_null() {
        return 0;
    }
    ((*col).seq.len() + (*col).map.len()) as i64
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Sequential push
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_push_int(col: *mut PicoCollection, val: i64) {
    (*col).seq.push(Slot::from_int(val));
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_push_str(col: *mut PicoCollection, val: *mut c_char) {
    (*col).seq.push(Slot::from_ptr(val as *mut u8));
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_push_ptr(col: *mut PicoCollection, val: *mut u8) {
    (*col).seq.push(Slot::from_ptr(val));
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Sequential get (integer index)
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_get_int_at(col: *mut PicoCollection, index: i64) -> i64 {
    let c = &*col;
    let i = index as usize;
    if i < c.seq.len() {
        c.seq[i].as_int()
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_get_str_at(
    col: *mut PicoCollection,
    index: i64,
) -> *mut c_char {
    let c = &*col;
    let i = index as usize;
    if i < c.seq.len() {
        c.seq[i].as_ptr() as *mut c_char
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_get_ptr_at(
    col: *mut PicoCollection,
    index: i64,
) -> *mut u8 {
    let c = &*col;
    let i = index as usize;
    if i < c.seq.len() {
        c.seq[i].as_ptr()
    } else {
        std::ptr::null_mut()
    }
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Sequential set (integer index)
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_set_int_at(
    col: *mut PicoCollection,
    index: i64,
    val: i64,
) {
    let c = &mut *col;
    let i = index as usize;
    // Extend if necessary
    while c.seq.len() <= i {
        c.seq.push(Slot::default());
    }
    c.seq[i] = Slot::from_int(val);
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_set_str_at(
    col: *mut PicoCollection,
    index: i64,
    val: *mut c_char,
) {
    let c = &mut *col;
    let i = index as usize;
    while c.seq.len() <= i {
        c.seq.push(Slot::default());
    }
    c.seq[i] = Slot::from_ptr(val as *mut u8);
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_set_ptr_at(
    col: *mut PicoCollection,
    index: i64,
    val: *mut u8,
) {
    let c = &mut *col;
    let i = index as usize;
    while c.seq.len() <= i {
        c.seq.push(Slot::default());
    }
    c.seq[i] = Slot::from_ptr(val);
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Pop / last
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_pop_int(col: *mut PicoCollection) -> i64 {
    (*col).seq.pop().map(|s| s.as_int()).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_pop_str(col: *mut PicoCollection) -> *mut c_char {
    (*col)
        .seq
        .pop()
        .map(|s| s.as_ptr() as *mut c_char)
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_pop_ptr(col: *mut PicoCollection) -> *mut u8 {
    (*col)
        .seq
        .pop()
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_last_int(col: *mut PicoCollection) -> i64 {
    (*col).seq.last().map(|s| s.as_int()).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_last_str(col: *mut PicoCollection) -> *mut c_char {
    (*col)
        .seq
        .last()
        .map(|s| s.as_ptr() as *mut c_char)
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_last_ptr(col: *mut PicoCollection) -> *mut u8 {
    (*col)
        .seq
        .last()
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null_mut())
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Associative get (string key)
// ═══════════════════════════════════════════════════════════════════

unsafe fn key_str(key: *const c_char) -> String {
    CStr::from_ptr(key).to_string_lossy().into_owned()
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_get_int(
    col: *mut PicoCollection,
    key: *const c_char,
) -> i64 {
    let k = key_str(key);
    (*col).map.get(&k).map(|s| s.as_int()).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_get_str(
    col: *mut PicoCollection,
    key: *const c_char,
) -> *mut c_char {
    let k = key_str(key);
    (*col)
        .map
        .get(&k)
        .map(|s| s.as_ptr() as *mut c_char)
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_get_ptr(
    col: *mut PicoCollection,
    key: *const c_char,
) -> *mut u8 {
    let k = key_str(key);
    (*col)
        .map
        .get(&k)
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null_mut())
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Associative set (string key)
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_set_int(
    col: *mut PicoCollection,
    key: *const c_char,
    val: i64,
) {
    let k = key_str(key);
    if !(*col).map.contains_key(&k) {
        (*col).key_order.push(k.clone());
    }
    (*col).map.insert(k, Slot::from_int(val));
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_set_str(
    col: *mut PicoCollection,
    key: *const c_char,
    val: *mut c_char,
) {
    let k = key_str(key);
    if !(*col).map.contains_key(&k) {
        (*col).key_order.push(k.clone());
    }
    (*col).map.insert(k, Slot::from_ptr(val as *mut u8));
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_set_ptr(
    col: *mut PicoCollection,
    key: *const c_char,
    val: *mut u8,
) {
    let k = key_str(key);
    if !(*col).map.contains_key(&k) {
        (*col).key_order.push(k.clone());
    }
    (*col).map.insert(k, Slot::from_ptr(val));
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Associative query
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_has(
    col: *mut PicoCollection,
    key: *const c_char,
) -> i64 {
    let k = key_str(key);
    (*col).map.contains_key(&k) as i64
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_key_at(
    col: *mut PicoCollection,
    index: i64,
) -> *mut c_char {
    let i = index as usize;
    let c = &*col;
    if i < c.key_order.len() {
        std::ffi::CString::new(c.key_order[i].as_str())
            .unwrap_or_default()
            .into_raw()
    } else {
        std::ptr::null_mut()
    }
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Valid index / search / contains
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_valid_index(
    col: *mut PicoCollection,
    index: i64,
) -> i64 {
    let i = index as usize;
    (index >= 0 && i < (*col).seq.len()) as i64
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_index_of_int(
    col: *mut PicoCollection,
    needle: i64,
) -> i64 {
    for (i, slot) in (*col).seq.iter().enumerate() {
        if slot.as_int() == needle {
            return i as i64;
        }
    }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_index_of_str(
    col: *mut PicoCollection,
    needle: *const c_char,
) -> i64 {
    let needle_bytes = CStr::from_ptr(needle).to_bytes();
    for (i, slot) in (*col).seq.iter().enumerate() {
        let ptr = slot.as_ptr() as *const c_char;
        if !ptr.is_null() && CStr::from_ptr(ptr).to_bytes() == needle_bytes {
            return i as i64;
        }
    }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_contains_int(
    col: *mut PicoCollection,
    needle: i64,
) -> i64 {
    (pico_collection_index_of_int(col, needle) >= 0) as i64
}

#[no_mangle]
pub unsafe extern "C" fn pico_collection_contains_str(
    col: *mut PicoCollection,
    needle: *const c_char,
) -> i64 {
    (pico_collection_index_of_str(col, needle) >= 0) as i64
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Slice
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_slice(
    col: *mut PicoCollection,
    start: i64,
    length: i64,
) -> *mut PicoCollection {
    let result = Box::into_raw(Box::new(PicoCollection::new()));
    let s = start.max(0) as usize;
    let seq = &(*col).seq;

    let end = if length < 0 {
        seq.len()
    } else {
        (s + length as usize).min(seq.len())
    };

    if s < seq.len() {
        for slot in &seq[s..end] {
            (*result).seq.push(*slot);
        }
    }

    result
}

// ═══════════════════════════════════════════════════════════════════
// FFI: Join (for Collection<PicoString>)
// ═══════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn pico_collection_join(
    col: *mut PicoCollection,
    delimiter: *const c_char,
) -> *mut c_char {
    let delim = CStr::from_ptr(delimiter).to_str().unwrap_or("");
    let parts: Vec<&str> = (*col)
        .seq
        .iter()
        .filter_map(|slot| {
            let ptr = slot.as_ptr() as *const c_char;
            if ptr.is_null() {
                None
            } else {
                CStr::from_ptr(ptr).to_str().ok()
            }
        })
        .collect();

    let joined = parts.join(delim);
    std::ffi::CString::new(joined)
        .unwrap_or_default()
        .into_raw()
}
