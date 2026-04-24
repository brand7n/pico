/**
 * FFI smoke test: calls every category of runtime function through the
 * same C ABI that compiled picohp programs use.
 *
 * Build:
 *   cargo build --release
 *   cc -o test_ffi test_ffi.c -Ltarget/release -lpico_runtime -lpthread -ldl -lm
 *   ./test_ffi
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <stdint.h>
#include <math.h>

/* ── String functions ─────────────────────────────────────────────── */
extern int64_t pico_string_length(const char *s);
extern int64_t pico_string_char_at(const char *s, int64_t index);
extern char   *pico_string_substring(const char *s, int64_t start, int64_t length);
extern int64_t pico_string_index_of(const char *s, const char *needle, int64_t offset);
extern int64_t pico_string_starts_with(const char *s, const char *prefix);
extern int64_t pico_string_ends_with(const char *s, const char *suffix);
extern int64_t pico_string_contains(const char *s, const char *needle);
extern char   *pico_string_concat(const char *a, const char *b);
extern int64_t pico_string_equals(const char *a, const char *b);
extern char   *pico_string_trim(const char *s);
extern char   *pico_string_to_lower(const char *s);
extern char   *pico_string_to_upper(const char *s);
extern char   *pico_string_replace(const char *s, const char *search, const char *replace);
extern int64_t pico_string_to_int(const char *s);
extern char   *pico_int_to_string(int64_t val);
extern char   *pico_float_to_string(double val);

/* ── Collection functions ─────────────────────────────────────────── */
typedef void PicoCollection;
extern PicoCollection *pico_collection_new(void);
extern int64_t pico_collection_count(PicoCollection *col);
extern void    pico_collection_push_int(PicoCollection *col, int64_t val);
extern void    pico_collection_push_str(PicoCollection *col, char *val);
extern void    pico_collection_push_ptr(PicoCollection *col, void *val);
extern int64_t pico_collection_get_int_at(PicoCollection *col, int64_t index);
extern char   *pico_collection_get_str_at(PicoCollection *col, int64_t index);
extern void   *pico_collection_get_ptr_at(PicoCollection *col, int64_t index);
extern void    pico_collection_set_int_at(PicoCollection *col, int64_t index, int64_t val);
extern int64_t pico_collection_pop_int(PicoCollection *col);
extern int64_t pico_collection_last_int(PicoCollection *col);
extern void    pico_collection_set_int(PicoCollection *col, const char *key, int64_t val);
extern int64_t pico_collection_get_int(PicoCollection *col, const char *key);
extern int64_t pico_collection_has(PicoCollection *col, const char *key);
extern char   *pico_collection_key_at(PicoCollection *col, int64_t index);
extern int64_t pico_collection_valid_index(PicoCollection *col, int64_t index);
extern int64_t pico_collection_index_of_int(PicoCollection *col, int64_t needle);
extern int64_t pico_collection_contains_int(PicoCollection *col, int64_t needle);
extern PicoCollection *pico_collection_slice(PicoCollection *col, int64_t start, int64_t len);
extern char   *pico_collection_join(PicoCollection *col, const char *delimiter);

/* ── Regex functions ──────────────────────────────────────────────── */
typedef void CompiledRegex;
extern CompiledRegex *pico_regex_compile(const char *pattern);
extern void    pico_regex_free(CompiledRegex *compiled);
extern int64_t pico_regex_exec(CompiledRegex *compiled, const char *subject, int64_t offset);
extern char   *pico_regex_exec_str(CompiledRegex *compiled, const char *subject, int64_t offset);
extern int64_t pico_regex_match(const char *pattern, const char *subject, int64_t offset);

/* ── Value functions ──────────────────────────────────────────────── */
typedef void PicoValue;
extern PicoValue *pico_value_none(void);
extern PicoValue *pico_value_from_int(int64_t val);
extern PicoValue *pico_value_from_bool(int64_t val);
extern PicoValue *pico_value_from_float(double val);
extern PicoValue *pico_value_from_string(char *val);
extern PicoValue *pico_value_from_object(void *val);
extern int64_t    pico_value_tag(const PicoValue *val);
extern int64_t    pico_value_as_int(const PicoValue *val);
extern int64_t    pico_value_as_bool(const PicoValue *val);
extern double     pico_value_as_float(const PicoValue *val);
extern char      *pico_value_as_string(const PicoValue *val);
extern void      *pico_value_as_object(const PicoValue *val);

/* ── File functions ───────────────────────────────────────────────── */
extern int64_t pico_file_exists(const char *path);
extern int64_t pico_file_is_dir(const char *path);

/* ── Alloc ────────────────────────────────────────────────────────── */
extern void   *picohp_object_alloc(uint64_t size, int64_t type_id);
extern int64_t pico_rt_version(void);

/* ── Test harness ─────────────────────────────────────────────────── */
static int passed = 0;
static int failed = 0;

static void check(const char *name, int condition) {
    if (condition) {
        printf("  \033[32m✓\033[0m %s\n", name);
        passed++;
    } else {
        printf("  \033[31m✗\033[0m %s\n", name);
        failed++;
    }
}

int main(void) {
    printf("═══ Runtime version ═══\n\n");
    check("pico_rt_version() == 3", pico_rt_version() == 3);

    printf("\n═══ String: length / charAt ═══\n\n");
    check("length('hello') == 5", pico_string_length("hello") == 5);
    check("length('') == 0", pico_string_length("") == 0);
    check("charAt('hello', 0) == 'h'", pico_string_char_at("hello", 0) == 'h');
    check("charAt('hello', 4) == 'o'", pico_string_char_at("hello", 4) == 'o');
    check("charAt OOB == -1", pico_string_char_at("hi", 5) == -1);

    printf("\n═══ String: substring ═══\n\n");
    char *sub1 = pico_string_substring("hello world", 0, 5);
    check("substring(0, 5) == 'hello'", strcmp(sub1, "hello") == 0);
    char *sub2 = pico_string_substring("hello world", 6, -1);
    check("substring(6, -1) == 'world'", strcmp(sub2, "world") == 0);

    printf("\n═══ String: search ═══\n\n");
    check("indexOf('hello', 'llo') == 2", pico_string_index_of("hello", "llo", 0) == 2);
    check("indexOf('hello', 'xyz') == -1", pico_string_index_of("hello", "xyz", 0) == -1);
    check("indexOf with offset", pico_string_index_of("abab", "ab", 1) == 2);
    check("startsWith('hello', 'hel')", pico_string_starts_with("hello", "hel") == 1);
    check("!startsWith('hello', 'world')", pico_string_starts_with("hello", "world") == 0);
    check("endsWith('hello', 'llo')", pico_string_ends_with("hello", "llo") == 1);
    check("contains('hello world', 'world')", pico_string_contains("hello world", "world") == 1);
    check("!contains('hello', 'xyz')", pico_string_contains("hello", "xyz") == 0);

    printf("\n═══ String: transform ═══\n\n");
    char *cat = pico_string_concat("hello", " world");
    check("concat", strcmp(cat, "hello world") == 0);
    check("equals same", pico_string_equals("abc", "abc") == 1);
    check("equals diff", pico_string_equals("abc", "xyz") == 0);
    char *trimmed = pico_string_trim("  hi  ");
    check("trim", strcmp(trimmed, "hi") == 0);
    char *lower = pico_string_to_lower("Hello World");
    check("toLower", strcmp(lower, "hello world") == 0);
    char *upper = pico_string_to_upper("Hello World");
    check("toUpper", strcmp(upper, "HELLO WORLD") == 0);
    char *replaced = pico_string_replace("hello world", "world", "pico");
    check("replace", strcmp(replaced, "hello pico") == 0);

    printf("\n═══ String: conversion ═══\n\n");
    check("toInt('42') == 42", pico_string_to_int("42") == 42);
    check("toInt('-7') == -7", pico_string_to_int("-7") == -7);
    char *s42 = pico_int_to_string(42);
    check("intToString(42) == '42'", strcmp(s42, "42") == 0);
    char *sf = pico_float_to_string(3.14);
    check("floatToString(3.14) starts with '3.14'", strncmp(sf, "3.14", 4) == 0);

    printf("\n═══ Collection: sequential ═══\n\n");
    PicoCollection *c = pico_collection_new();
    check("new collection count == 0", pico_collection_count(c) == 0);
    pico_collection_push_int(c, 10);
    pico_collection_push_int(c, 20);
    pico_collection_push_int(c, 30);
    check("push 3 items, count == 3", pico_collection_count(c) == 3);
    check("getAt(0) == 10", pico_collection_get_int_at(c, 0) == 10);
    check("getAt(1) == 20", pico_collection_get_int_at(c, 1) == 20);
    check("getAt(2) == 30", pico_collection_get_int_at(c, 2) == 30);
    pico_collection_set_int_at(c, 1, 99);
    check("setAt(1, 99), getAt(1) == 99", pico_collection_get_int_at(c, 1) == 99);
    check("last == 30", pico_collection_last_int(c) == 30);
    int64_t popped = pico_collection_pop_int(c);
    check("pop == 30", popped == 30);
    check("after pop, count == 2", pico_collection_count(c) == 2);

    printf("\n═══ Collection: associative ═══\n\n");
    PicoCollection *m = pico_collection_new();
    pico_collection_set_int(m, "width", 1920);
    pico_collection_set_int(m, "height", 1080);
    check("get('width') == 1920", pico_collection_get_int(m, "width") == 1920);
    check("get('height') == 1080", pico_collection_get_int(m, "height") == 1080);
    check("has('width')", pico_collection_has(m, "width") == 1);
    check("!has('depth')", pico_collection_has(m, "depth") == 0);
    char *k0 = pico_collection_key_at(m, 0);
    check("keyAt(0) == 'width'", k0 != NULL && strcmp(k0, "width") == 0);

    printf("\n═══ Collection: search ═══\n\n");
    PicoCollection *s2 = pico_collection_new();
    pico_collection_push_int(s2, 5);
    pico_collection_push_int(s2, 10);
    pico_collection_push_int(s2, 15);
    check("indexOf(10) == 1", pico_collection_index_of_int(s2, 10) == 1);
    check("indexOf(99) == -1", pico_collection_index_of_int(s2, 99) == -1);
    check("contains(15)", pico_collection_contains_int(s2, 15) == 1);
    check("!contains(99)", pico_collection_contains_int(s2, 99) == 0);
    check("validIndex(0)", pico_collection_valid_index(s2, 0) == 1);
    check("!validIndex(99)", pico_collection_valid_index(s2, 99) == 0);

    printf("\n═══ Collection: slice / join ═══\n\n");
    PicoCollection *sl = pico_collection_slice(s2, 1, 2);
    check("slice(1,2) count == 2", pico_collection_count(sl) == 2);
    check("slice[0] == 10", pico_collection_get_int_at(sl, 0) == 10);
    check("slice[1] == 15", pico_collection_get_int_at(sl, 1) == 15);

    PicoCollection *parts = pico_collection_new();
    pico_collection_push_str(parts, "a");
    pico_collection_push_str(parts, "b");
    pico_collection_push_str(parts, "c");
    char *joined = pico_collection_join(parts, ",");
    check("join(',') == 'a,b,c'", strcmp(joined, "a,b,c") == 0);

    printf("\n═══ Regex ═══\n\n");
    check("one-shot match digits", pico_regex_match("[0-9]+", "42abc", 0) == 2);
    check("one-shot no match", pico_regex_match("[0-9]+", "abc", 0) == -1);
    check("match at offset", pico_regex_match("[a-z]+", "123abc", 3) == 3);

    CompiledRegex *re = pico_regex_compile("[a-zA-Z_][a-zA-Z0-9_]*");
    check("compile succeeds", re != NULL);
    check("exec identifier", pico_regex_exec(re, "hello world", 0) == 5);
    check("exec at offset", pico_regex_exec(re, "  foo", 2) == 3);
    check("exec no match", pico_regex_exec(re, "123", 0) == -1);

    char *matched = pico_regex_exec_str(re, "myVar = 42", 0);
    check("execStr returns 'myVar'", matched != NULL && strcmp(matched, "myVar") == 0);
    pico_regex_free(re);

    printf("\n═══ Regex: tokenizer pattern ═══\n\n");
    const char *source = "function main() { return 42; }";
    int64_t cursor = 0;

    CompiledRegex *re_id = pico_regex_compile("[a-zA-Z_][a-zA-Z0-9_]*");
    CompiledRegex *re_num = pico_regex_compile("[0-9]+");
    CompiledRegex *re_ws = pico_regex_compile("[ \\t\\n]+");
    CompiledRegex *re_sym = pico_regex_compile("[(){}=;]");

    int token_count = 0;
    int64_t len = strlen(source);
    while (cursor < len) {
        int64_t mt;
        if ((mt = pico_regex_exec(re_ws, source, cursor)) > 0) {
            cursor += mt;
        } else if ((mt = pico_regex_exec(re_id, source, cursor)) > 0) {
            cursor += mt; token_count++;
        } else if ((mt = pico_regex_exec(re_num, source, cursor)) > 0) {
            cursor += mt; token_count++;
        } else if ((mt = pico_regex_exec(re_sym, source, cursor)) > 0) {
            cursor += mt; token_count++;
        } else {
            cursor++;
        }
    }
    check("tokenized 'function main() { return 42; }' -> 9 tokens", token_count == 9);
    pico_regex_free(re_id);
    pico_regex_free(re_num);
    pico_regex_free(re_ws);
    pico_regex_free(re_sym);

    printf("\n═══ Value ═══\n\n");
    PicoValue *vn = pico_value_none();
    check("value_none tag == 0", pico_value_tag(vn) == 0);

    PicoValue *vi = pico_value_from_int(42);
    check("value_from_int tag == 1", pico_value_tag(vi) == 1);
    check("value_as_int == 42", pico_value_as_int(vi) == 42);

    PicoValue *vb = pico_value_from_bool(1);
    check("value_from_bool tag == 2", pico_value_tag(vb) == 2);
    check("value_as_bool == 1", pico_value_as_bool(vb) == 1);

    PicoValue *vf = pico_value_from_float(3.14);
    check("value_from_float tag == 3", pico_value_tag(vf) == 3);
    check("value_as_float ~= 3.14", fabs(pico_value_as_float(vf) - 3.14) < 0.001);

    PicoValue *vs = pico_value_from_string("hello");
    check("value_from_string tag == 4", pico_value_tag(vs) == 4);
    check("value_as_string == 'hello'", strcmp(pico_value_as_string(vs), "hello") == 0);

    /* Use a collection as an opaque "object" pointer */
    PicoCollection *obj = pico_collection_new();
    PicoValue *vo = pico_value_from_object(obj);
    check("value_from_object tag == 5", pico_value_tag(vo) == 5);
    check("value_as_object round-trips", pico_value_as_object(vo) == obj);

    /* Value in a collection (parser stack pattern) */
    PicoCollection *stack = pico_collection_new();
    pico_collection_push_ptr(stack, vi);
    pico_collection_push_ptr(stack, vs);
    pico_collection_push_ptr(stack, vn);
    check("stack count == 3", pico_collection_count(stack) == 3);
    PicoValue *top = (PicoValue *)pico_collection_get_ptr_at(stack, 0);
    check("stack[0] tag == INT", pico_value_tag(top) == 1);
    check("stack[0] as_int == 42", pico_value_as_int(top) == 42);

    printf("\n═══ Object allocation ═══\n\n");
    int64_t *point = (int64_t *)picohp_object_alloc(24, 1);
    check("alloc returns non-null", point != NULL);
    check("zeroed memory", point[0] == 0 && point[1] == 0 && point[2] == 0);
    point[0] = 1;   /* type_id */
    point[1] = 10;  /* x */
    point[2] = 20;  /* y */
    check("write/read type_id", point[0] == 1);
    check("write/read x", point[1] == 10);
    check("write/read y", point[2] == 20);

    printf("\n═══ File ═══\n\n");
    check("file_exists('.')", pico_file_exists(".") == 1);
    check("file_is_dir('.')", pico_file_is_dir(".") == 1);
    check("!file_exists('nonexistent_xyz')", pico_file_exists("nonexistent_xyz") == 0);

    /* ═══ Summary ═══ */
    printf("\n══════════════════════════════════════════════════\n");
    printf("Results: \033[32m%d passed\033[0m", passed);
    if (failed > 0) printf(", \033[31m%d failed\033[0m", failed);
    printf(" out of %d tests\n", passed + failed);
    return failed > 0 ? 1 : 0;
}
