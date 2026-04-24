# Pico Runtime Specification v0.2

## Overview

Pico is the minimal runtime and language dialect for the picohp self-hosting compiler. It defines the contract between PHP source code and the compiled native binary: what types exist, what operations are available, and how the compiler lowers them.

The runtime is designed to be **sufficient for self-compilation** — the picohp compiler, its tokenizer, and a port of nikic's PHP parser should be expressible entirely within this dialect. Anything outside the dialect requires porting before compilation.

The Pico dialect is **standard PHP** — not a custom syntax or preprocessor. Pico classes are real PHP classes that run under the standard PHP interpreter. PHPStan validates them at the highest level. The compiler happens to compile them to efficient native code.

## Design Principles

1. **No mixed.** Every value has a concrete type known at compile time. The PHP `mixed` type exists only in method signatures for PHPStan compatibility; the compiler resolves it to the actual type via `@template` annotations.

2. **Standard PHP syntax.** Array bracket operators (`$a[0]`, `$a['key']`, `$a[] = $v`) work on `Collection` via PHP's `ArrayAccess` interface. Array literals (`[1, 2, 3]`) work via `Collection::from()`. The code is valid PHP that runs under any PHP 8.2+ interpreter.

3. **No PHP stdlib.** Functions like `substr`, `count`, `array_push`, `in_array`, `str_starts_with` are not available. Equivalent operations are methods on `Remodulate\Pico\PicoString` or `Remodulate\Pico\Collection`.

4. **Two implementations per class.** A PHP reference implementation (for the interpreter and differential testing) and a Rust native implementation (linked as a static `.a` for production). Both must be behaviorally identical.

5. **Compile-time type dispatch.** The compiler chooses which runtime function to call based on the resolved type. A `Collection<int>::push(42)` emits `pico_collection_push_int`. A `Collection<Node>::push($node)` emits `pico_collection_push_ptr`. The runtime never inspects types at runtime.

## Namespace

```
Remodulate\Pico\           — runtime classes (this package)
Remodulate\PicoIR\         — intermediate representation and compiler backend
Remodulate\PicoHP\         — compiler frontend (semantic analysis, AST transforms)
```

## Runtime Classes

### Remodulate\Pico\PicoString

Immutable byte string. All string operations produce new strings; the original is never modified. Supports bracket syntax for character access.

```php
namespace Remodulate\Pico;

class PicoString
{
    public function length(): int;
    public function charAt(int $index): int;
    public function substring(int $start, int $length = -1): PicoString;
    public function indexOf(PicoString $needle, int $offset = 0): int;
    public function startsWith(PicoString $prefix): bool;
    public function endsWith(PicoString $suffix): bool;
    public function contains(PicoString $needle): bool;
    public function concat(PicoString $other): PicoString;
    public function equals(PicoString $other): bool;
    public function trim(): PicoString;
    public function toLower(): PicoString;
    public function toUpper(): PicoString;
    public function replace(PicoString $search, PicoString $replace): PicoString;
    public function split(PicoString $delimiter): Collection;
    public function toInt(): int;
}
```

**Bracket access:** `$s[0]` is lowered by the compiler to `$s->charAt(0)`. The compiler recognizes `ArrayDimFetch` on a `PicoString`-typed value and emits `pico_string_char_at`. Read-only — `$s[0] = 'x'` is a compile error (strings are immutable).

**LLVM representation:** `ptr` (pointer to null-terminated UTF-8 byte buffer).

**Escape analysis:** Most methods return a new string. `$this` and arguments are `NoEscape` (not captured). `concat`, `replace`, `split` allocate new strings that may escape via return.

**Replaces:** `substr`, `strpos`, `str_starts_with`, `str_ends_with`, `str_contains`, `strtolower`, `strtoupper`, `str_replace`, `explode`, `trim`, `intval`, `strlen`.

### Remodulate\Pico\StringBuilder

Mutable string builder for constructing output (LLVM IR text, assembly, source code).

```php
namespace Remodulate\Pico;

class StringBuilder
{
    public function append(PicoString $s): void;
    public function appendChar(int $byte): void;
    public function appendInt(int $val): void;
    public function toString(): PicoString;
    public function length(): int;
    public function clear(): void;
}
```

**LLVM representation:** `ptr` (pointer to growable byte buffer struct).

**Escape analysis:** `append*` methods capture `$this` (mutate it) but do not capture arguments. `toString` returns a new `PicoString` that escapes.

**Replaces:** String concatenation via `.` and `.=` operators for output building. Source code can still use `.` for simple cases; `StringBuilder` is for hot loops and large outputs.

### Remodulate\Pico\Collection

Unified ordered map. Serves as both sequential array (integer-keyed) and associative array (string-keyed). One data structure internally — a hash map that preserves insertion order.

Implements `ArrayAccess` for bracket syntax and `Countable` for `count()` compatibility.

```php
namespace Remodulate\Pico;

/**
 * @template T
 * @implements \ArrayAccess<int|string, T>
 */
class Collection implements \ArrayAccess, \Countable
{
    // -- Sequential (array-like) access --

    /**
     * Append a value. Key is the next sequential integer.
     * @param T $val
     */
    public function push(mixed $val): void;

    /**
     * Remove and return the last element.
     * @return T
     */
    public function pop(): mixed;

    /**
     * Get value at integer index.
     * @return T
     */
    public function getAt(int $index): mixed;

    /**
     * Set value at integer index.
     * @param T $val
     */
    public function setAt(int $index, mixed $val): void;

    /**
     * Return the last element without removing it.
     * @return T
     */
    public function last(): mixed;

    /**
     * @return Collection<T>
     */
    public function slice(int $start, int $length = -1): Collection;

    // -- Associative (map-like) access --

    /**
     * Get value by string key.
     * @return T
     */
    public function get(string $key): mixed;

    /**
     * Set value by string key.
     * @param T $val
     */
    public function set(string $key, mixed $val): void;

    /**
     * Check if a string key exists.
     */
    public function has(string $key): bool;

    /**
     * Get the string key at a positional index.
     */
    public function keyAt(int $index): string;

    // -- Common --

    public function count(): int;

    /**
     * Check if integer index is within bounds.
     */
    public function validIndex(int $index): bool;

    /**
     * Search for a value, return its index or -1.
     * @param T $needle
     */
    public function indexOf(mixed $needle): int;

    /**
     * Check if collection contains a value.
     * @param T $needle
     */
    public function containsValue(mixed $needle): bool;

    // -- ArrayAccess implementation --

    /**
     * @param int|string $offset
     */
    public function offsetExists(mixed $offset): bool;

    /**
     * @param int|string $offset
     * @return T
     */
    public function offsetGet(mixed $offset): mixed;

    /**
     * @param int|string|null $offset null for push ($a[] = $v)
     * @param T $value
     */
    public function offsetSet(mixed $offset, mixed $value): void;

    /**
     * @param int|string $offset
     */
    public function offsetUnset(mixed $offset): void;

    // -- Factory --

    /**
     * Create a collection from a PHP array literal.
     * Under regular PHP: copies elements from the array.
     * Under compilation: the array literal is never allocated; the compiler
     * inlines it as new Collection() + push() for each element.
     *
     * @param array<T> $items
     * @return self<T>
     */
    public static function from(array $items): self;

    /**
     * Create a collection from variadic arguments.
     * @param T ...$values
     * @return self<T>
     */
    public static function of(mixed ...$values): self;
}
```

**Bracket syntax lowering:** The compiler recognizes `ArrayDimFetch` and `Assign` on `Collection`-typed values and lowers them to method calls. The `ArrayAccess` methods exist in the PHP reference implementation for runtime compatibility, but the compiler bypasses them and emits direct typed runtime calls:

| PHP syntax | Compiler lowering | Runtime call (T=int) |
|---|---|---|
| `$c[0]` (int key, read) | `$c->getAt(0)` | `pico_collection_get_int_at` |
| `$c[0] = 42` (int key, write) | `$c->setAt(0, 42)` | `pico_collection_set_int_at` |
| `$c['k']` (string key, read) | `$c->get('k')` | `pico_collection_get_int` |
| `$c['k'] = 42` (string key, write) | `$c->set('k', 42)` | `pico_collection_set_int` |
| `$c[] = 42` (push) | `$c->push(42)` | `pico_collection_push_int` |
| `isset($c[0])` | `$c->validIndex(0)` | `pico_collection_valid_index` |
| `isset($c['k'])` | `$c->has('k')` | `pico_collection_has` |
| `count($c)` | `$c->count()` | `pico_collection_count` |

**`Collection::from()` lowering:** When the compiler sees `Collection::from([1, 2, 3])`, it recognizes the `Array_` literal argument and inlines the construction. No PHP array is ever allocated at runtime:

```php
// Source
$ids = Collection::from([1, 2, 3]);

// Compiler emits (IR level)
%c = call ptr @pico_collection_new()
call void @pico_collection_push_int(ptr %c, i32 1)
call void @pico_collection_push_int(ptr %c, i32 2)
call void @pico_collection_push_int(ptr %c, i32 3)
```

`Collection::from()` with a non-literal argument (a variable) is a compile error. For constructing from existing data, use the constructor and `push()`.

**LLVM representation:** `ptr` (pointer to `PicoMap` struct). All elements stored as their concrete LLVM type — `i32` for `Collection<int>`, `ptr` for `Collection<Node>`.

**Escape analysis:** `push`, `set`, `setAt` capture their value argument into `$this` (`ArgEscape` — value escapes as far as the collection does). `get`, `getAt`, `pop`, `last` return values that escape via return. `$this` is `ArgEscape` on mutating methods.

**Replaces:** All PHP array operations (`array_push`, `array_pop`, `count`, `in_array`, `array_key_exists`, `array_search`, `array_slice`, `end`), all associative array operations.

### Remodulate\Pico\File

Static file I/O. No file handles, no streams, no buffering. Read the whole file or write the whole file.

```php
namespace Remodulate\Pico;

class File
{
    public static function readContents(PicoString $path): PicoString;
    public static function writeContents(PicoString $path, PicoString $data): void;
    public static function exists(PicoString $path): bool;
    public static function isFile(PicoString $path): bool;
    public static function isDir(PicoString $path): bool;
}
```

**LLVM representation:** No instances — all static methods. Calls lower to direct runtime function calls.

**Escape analysis:** `readContents` returns a new string that escapes. Arguments are `NoEscape` (paths are read, not captured).

**Replaces:** `file_get_contents`, `file_put_contents`, `file_exists`, `is_file`, `is_dir`.

## Usage Examples

### Natural PHP with full type safety

```php
use Remodulate\Pico\{Collection, PicoString, StringBuilder, File};

// Array literals via from() — PHPStan infers Collection<int>
$ids = Collection::from([268, 301, 302]);

// Bracket syntax — PHPStan validates through ArrayAccess<int|string, T>
$first = $ids[0];                    // PHPStan: int
$ids[1] = 999;                      // PHPStan: ✓ int matches T=int
$ids[] = 42;                        // push via bracket

// String-keyed — PHPStan infers Collection<ClassMetadata>
/** @var Collection<ClassMetadata> */
$registry = new Collection();
$registry['Foo\\Bar'] = $meta;      // set by string key
$m = $registry['Foo\\Bar'];         // PHPStan: ClassMetadata

// String character access
$src = new PicoString("hello");
$ch = $src[0];                      // int (104 = 'h')

// Method calls where needed
$last = $ids->pop();                // no bracket equivalent
$has = $registry->has('Baz');       // explicit key check
$len = $ids->count();               // or count($ids) via Countable

// StringBuilder for output
$sb = new StringBuilder();
$sb->append(new PicoString("define i32 @main() {\n"));
$sb->appendInt(42);
$output = $sb->toString();

// File I/O
$source = File::readContents(new PicoString("input.php"));
File::writeContents(new PicoString("output.ll"), $output);
```

### How the compiler sees it

The same code above, from the compiler's perspective:

```php
// $ids = Collection::from([268, 301, 302]);
// Compiler sees: StaticCall to Collection::from with Array_ argument
// Lowers to: new + push x3 (array literal never allocated)

// $first = $ids[0];
// Compiler sees: ArrayDimFetch on Collection<int> with int dim
// Resolves element type: int (from @template T = int)
// Emits: pico_collection_get_int_at(ptr, i32) → i32

// $registry['Foo\\Bar'] = $meta;
// Compiler sees: Assign with ArrayDimFetch on Collection<ClassMetadata> with string dim
// Emits: pico_collection_set_ptr(ptr, ptr, ptr)

// $m = $registry['Foo\\Bar'];
// Compiler sees: ArrayDimFetch on Collection<ClassMetadata> with string dim
// Resolves element type: ClassMetadata (from @var annotation)
// Emits: pico_collection_get_ptr(ptr, ptr) → ptr
// Downstream: $m typed as ClassMetadata for GEP/method dispatch
```

## Type System

### Scalar types

| Pico type | PHP type | LLVM type |
|---|---|---|
| `int` | `int` | `i32` |
| `float` | `float` | `double` |
| `bool` | `bool` | `i1` |
| `string` | `Remodulate\Pico\PicoString` | `ptr` |
| `void` | `void` | `void` |

### Object types

All classes compile to LLVM structs. Field 0 is always `i32 type_id` for runtime type identification (instanceof, catch dispatch).

```llvm
%struct.Point = type { i32, i32, i32 }     ; type_id, x, y
%struct.Token = type { i32, i32, ptr }     ; type_id, kind, value
```

### Nullable types

`?T` is represented as `ptr` regardless of the underlying type. Null is the zero pointer. For nullable scalars (`?int`), the value is stored as a pointer-sized integer via `inttoptr`/`ptrtoint`.

### Collection types (via @template)

PHPStan's `@template` system tracks the element type. The compiler's `DocTypeParser` resolves `T` at each use site. The concrete type determines which runtime function is called.

```php
/** @var Collection<int> */
$ids = Collection::from([1, 2, 3]);     // T = int

/** @var Collection<Node> */
$children = new Collection();            // T = Node

/** @var Collection<Collection<string>> */
$grid = new Collection();                // T = Collection<string>

// Type inference via from() — no @var needed
$nums = Collection::from([10, 20]);      // PHPStan infers Collection<int>
```

## Language Subset

### Supported constructs

- Classes with typed properties, methods, constructors
- Constructor promotion (`public readonly int $x`)
- Interfaces (for type hierarchy; methods resolved via virtual dispatch)
- `ArrayAccess` and `Countable` interfaces (for bracket syntax and `count()`)
- Abstract classes
- Enums (int-backed and string-backed)
- Traits (inlined at compile time)
- Static methods and properties
- Typed parameters and return types
- Nullable types (`?int`, `?Node`)
- PHPDoc `@template`, `@var`, `@param`, `@return` for generic type resolution
- String concatenation (`.` operator)
- Arithmetic, comparison, logical operators
- Array bracket syntax on `Collection` (`$c[0]`, `$c['key']`, `$c[] = $v`)
- String bracket syntax on `PicoString` (`$s[0]` for character access)
- Array literals via `Collection::from([...])` (compile-time inlined)
- `if`/`elseif`/`else`, `while`, `do`/`while`, `for`, `foreach`
- `switch`/`case`, `match`
- `try`/`catch` (implemented as value-based error returns, not stack unwinding)
- `throw` (new exception or re-throw)
- `instanceof`
- `count()` on `Collection` (via `Countable` interface)
- Closures (future — modeled as allocation with captured fields)
- Namespaces

### Not supported

- Raw PHP arrays as standalone values (only as literals in `Collection::from()`)
- PHP stdlib functions — use `Remodulate\Pico\*` class methods
- References (`&$var`)
- Variable variables (`$$name`)
- Dynamic property access (`$obj->$prop`)
- `eval`, `include`/`require` (runtime code loading)
- Generators, fibers
- Attributes
- Union types (except `?T` nullable)
- Intersection types
- `mixed` as a runtime type (only as PHPStan compatibility in signatures)

## Compilation Pipeline

```
PHP source (Pico dialect — standard PHP with Remodulate\Pico\* classes)
    │
    ▼
Parser (nikic/PHP-Parser or picohp hand-written parser)
    │
    ▼
AST
    │
    ▼
Semantic Analysis Pass
    │  - Name resolution, type checking
    │  - Class/enum/trait registration
    │  - PHPDoc @template resolution
    │  - ArrayDimFetch on Collection → resolve T, dispatch to getAt/get
    │  - ArrayDimFetch on PicoString → resolve to charAt
    │  - Collection::from(Array_) → recognize as intrinsic
    │  - canThrow propagation
    ▼
Decorated AST (PicoType + Symbol on every node)
    │
    ▼
IR Construction (AST → PicoIR)
    │  - Alloca for locals, load/store for access
    │  - Collection bracket ops → typed runtime calls
    │  - Collection::from([...]) → inline new + push sequence
    │  - PicoString bracket → pico_string_char_at
    │  - Object construction → picohp_object_alloc + constructor call
    ▼
PicoIR (structured instruction graph)
    │
    ├─── Escape Analysis Pass
    │      Interprocedural function summaries
    │      Allocation site classification: NoEscape / ArgEscape / GlobalEscape
    │
    ├─── Stack Promotion Pass
    │      Rewrite heap allocs → alloca for NoEscape sites
    │
    ├─── Store-to-Load Forwarding Pass
    │      Eliminate alloca→store→load chains
    │      Params flow directly to uses
    │
    ├─── Dead Instruction Elimination Pass
    │      Remove unused pure instructions
    │      Cascading deletion to fixpoint
    │
    ▼
Optimized PicoIR
    │
    ├─── LLVM Text Emitter → .ll file → clang → native binary
    ├─── ARM64 Emitter → .s file → as + ld → native binary
    └─── Interpreter → direct execution in PHP (testing, bootstrap)
```

## Runtime ABI

### Object allocation

```
ptr picohp_object_alloc(i64 size, i32 type_id)
```

Allocates `size` bytes on the GC heap, returns a pointer. The caller stores `type_id` at field 0. Escape analysis may replace this with `alloca` (stack allocation) when the object does not escape.

### Collection runtime functions

Naming convention: `pico_collection_{op}_{valuetype}[_at]`

Sequential access (integer key):
```
void pico_collection_push_int(ptr collection, i32 value)
void pico_collection_push_str(ptr collection, ptr value)
void pico_collection_push_ptr(ptr collection, ptr value)
i32  pico_collection_get_int_at(ptr collection, i32 index)
ptr  pico_collection_get_str_at(ptr collection, i32 index)
ptr  pico_collection_get_ptr_at(ptr collection, i32 index)
void pico_collection_set_int_at(ptr collection, i32 index, i32 value)
void pico_collection_set_str_at(ptr collection, i32 index, ptr value)
void pico_collection_set_ptr_at(ptr collection, i32 index, ptr value)
```

Associative access (string key):
```
i32  pico_collection_get_int(ptr collection, ptr key)
ptr  pico_collection_get_str(ptr collection, ptr key)
ptr  pico_collection_get_ptr(ptr collection, ptr key)
void pico_collection_set_int(ptr collection, ptr key, i32 value)
void pico_collection_set_str(ptr collection, ptr key, ptr value)
void pico_collection_set_ptr(ptr collection, ptr key, ptr value)
i32  pico_collection_has(ptr collection, ptr key) → bool as i32
```

Common:
```
ptr  pico_collection_new()
i32  pico_collection_count(ptr collection)
i32  pico_collection_valid_index(ptr collection, i32 index) → bool as i32
i32  pico_collection_pop_int(ptr collection)
ptr  pico_collection_pop_str(ptr collection)
ptr  pico_collection_pop_ptr(ptr collection)
ptr  pico_collection_last_int(ptr collection) → i32
ptr  pico_collection_last_str(ptr collection) → ptr
ptr  pico_collection_last_ptr(ptr collection) → ptr
ptr  pico_collection_key_at(ptr collection, i32 index)
ptr  pico_collection_slice(ptr collection, i32 start, i32 length)
i32  pico_collection_index_of_int(ptr collection, i32 needle) → i32
i32  pico_collection_index_of_str(ptr collection, ptr needle) → i32
i32  pico_collection_contains_int(ptr collection, i32 needle) → bool as i32
i32  pico_collection_contains_str(ptr collection, ptr needle) → bool as i32
```

### String runtime functions

```
ptr  pico_string_new(ptr data, i32 length)
i32  pico_string_length(ptr s)
i32  pico_string_char_at(ptr s, i32 index)
ptr  pico_string_substring(ptr s, i32 start, i32 length)
i32  pico_string_index_of(ptr s, ptr needle, i32 offset)
i32  pico_string_starts_with(ptr s, ptr prefix)
i32  pico_string_ends_with(ptr s, ptr suffix)
i32  pico_string_contains(ptr s, ptr needle)
ptr  pico_string_concat(ptr a, ptr b)
i32  pico_string_equals(ptr a, ptr b)
ptr  pico_string_trim(ptr s)
ptr  pico_string_to_lower(ptr s)
ptr  pico_string_to_upper(ptr s)
ptr  pico_string_replace(ptr s, ptr search, ptr replace)
ptr  pico_string_split(ptr s, ptr delimiter)
i32  pico_string_to_int(ptr s)
ptr  pico_int_to_string(i32 val)
ptr  pico_float_to_string(double val)
```

### StringBuilder runtime functions

```
ptr  pico_sb_new()
void pico_sb_append(ptr sb, ptr s)
void pico_sb_append_char(ptr sb, i32 byte)
void pico_sb_append_int(ptr sb, i32 val)
ptr  pico_sb_to_string(ptr sb)
i32  pico_sb_length(ptr sb)
void pico_sb_clear(ptr sb)
```

### File runtime functions

```
ptr  pico_file_read(ptr path)
void pico_file_write(ptr path, ptr data)
i32  pico_file_exists(ptr path)
i32  pico_file_is_file(ptr path)
i32  pico_file_is_dir(ptr path)
```

## Escape Analysis Summaries for Runtime

The escape analysis pass maintains hardcoded summaries for all runtime functions. These are the interprocedural facts the analysis uses at call sites:

| Method pattern | $this | value args | return |
|---|---|---|---|
| `Collection::push` | ArgEscape (receives value) | ArgEscape via $this | — |
| `Collection::get*` | NoEscape (read-only) | NoEscape | GlobalEscape (returned) |
| `Collection::set*` | ArgEscape (receives value) | ArgEscape via $this | — |
| `Collection::from` | — (static) | NoEscape (copied into new collection) | GlobalEscape (returned) |
| `PicoString::*` | NoEscape (immutable) | NoEscape | GlobalEscape (new string returned) |
| `PicoString::charAt` | NoEscape | NoEscape | NoEscape (returns int, not ptr) |
| `StringBuilder::append` | ArgEscape (mutated) | NoEscape (data copied) | — |
| `StringBuilder::toString` | NoEscape | — | GlobalEscape (new string) |
| `File::*` | — (static) | NoEscape | GlobalEscape (returned) |

## Self-Hosting Path

1. **Implement the Pico runtime** in Rust (native) and PHP (reference). The PHP implementation is a real working library — `Collection` implements `ArrayAccess`, bracket syntax works, code runs under `php` directly.
2. **Port the tokenizer** to use `Remodulate\Pico\PicoString` and `Remodulate\Pico\Collection` — no `token_get_all`. Array syntax works naturally via `ArrayAccess`.
3. **Port the parser** (nikic's, or the hand-written grammar output) to use `Remodulate\Pico\Collection` for AST nodes, token streams, and internal state. Bracket syntax means minimal code changes from the original.
4. **Port the compiler** (`SemanticAnalysisPass`, `IRGenerationPass`, `Builder`) to use `Remodulate\Pico\*` classes for symbol tables, class registries, and output generation.
5. **Bootstrap test:** compile the compiler with itself, run the compiled compiler on its own source, verify IR output matches (fixed-point test).
6. **Remove Rust runtime dependency** (optional, long-term): rewrite the Pico runtime in the Pico dialect itself, compile it with picohp. The compiler and its runtime are fully self-contained.

## Versioning

This is version 0.2 of the Pico specification.

Changes from v0.1:
- `Collection` now implements `ArrayAccess` and `Countable` — bracket syntax is standard PHP, not a custom lowering
- Added `Collection::from(array)` factory for array literal initialization
- `PicoString` supports bracket syntax for character access (`$s[0]`)
- Added `count()` support via `Countable` interface
- Removed "no array syntax" design principle — the dialect is now standard PHP
- Added usage examples showing how source code and compiler lowering relate
- Added `pico_collection_valid_index`, `pico_collection_last_*`, `pico_collection_index_of_*`, `pico_collection_contains_*` runtime functions

The runtime surface is intentionally minimal. Classes may be added (regex, process execution, etc.) but the core four — `PicoString`, `StringBuilder`, `Collection`, `File` — are the stable foundation. Specialized collection types (`IntArray` with contiguous `i32` storage) may be added as performance optimizations when profiling identifies hot paths, but `Collection` remains the default.
