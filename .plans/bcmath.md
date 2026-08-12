# BCMath Procedural Functions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the 14 PHP 8.4 `bcmath` procedural functions on AOT and Magician through one `elephc-bcmath` crate.

**Architecture:** Pure-Rust PHP-compatible decimal engine in `crates/elephc-bcmath` (`staticlib` + `rlib`). AOT lowers `RuntimeFnId::Bc*` onto C ABI slots. Magician calls the same ABI. Process scale lives in the crate. No `BcMath\Number`, no operator overloading, no `ini bcmath.scale`.

**Tech Stack:** Rust workspace crate, `builtin!` / `eval_builtin!` registries, EIR `RuntimeFnId`, existing ValueError / DivisionByZeroError throw paths, `BRIDGES` table.

**Spec:** `.plans/bcmath-design.md`

## Global Constraints

- Plans and this file are English.
- All three supported targets (`macos-aarch64`, `linux-aarch64`, `linux-x86_64`) in the same change.
- EIR path only. No AST-optimizer special cases. No assembly decimal math.
- Every new Rust file starts with a `//!` preamble. Every function has a `///` docblock.
- Never run `cargo fmt`. Never run the full local suite. Focused tests only.
- Do not edit `Cargo.toml` / `Cargo.lock` package versions. Do not add ROADMAP rows.
- PHP messages and string forms are locked against `php -r` / php-src, not invented.
- `git diff --check` before every commit. No `Co-Authored-By`.
- Assembly comments on every `emitter.instruction(...)` aligned to column 81.

## Task list

- [ ] Task 1: Crate skeleton + parse / format / scale / add / sub / mul / div / mod / comp
- [ ] Task 2: Crate pow / powmod / sqrt / ceil / floor / round / divmod + fixture table
- [ ] Task 3: Workspace membership + `BRIDGES` + `--with-bcmath`
- [ ] Task 4: `RuntimeFnId::Bc*` contracts (effects, requirements, ownership, eir names)
- [ ] Task 5: AOT `builtin!` homes for all 14 names
- [ ] Task 6: AOT slots, `__rt_*` helpers, lowering, dispatch group
- [ ] Task 7: AOT codegen + error tests
- [ ] Task 8: Magician homes, hooks, extension_loaded
- [ ] Task 9: Magician tests + mixed AOT/eval scale
- [ ] Task 10: Example, docs, generated builtin pages, README

---

### Task 1: Crate skeleton and core arithmetic

**Files:**
- Create: `crates/elephc-bcmath/Cargo.toml`
- Create: `crates/elephc-bcmath/src/lib.rs`
- Create: `crates/elephc-bcmath/src/error.rs`
- Create: `crates/elephc-bcmath/src/parse.rs`
- Create: `crates/elephc-bcmath/src/num.rs`
- Create: `crates/elephc-bcmath/src/format.rs`
- Create: `crates/elephc-bcmath/src/scale.rs`
- Create: `crates/elephc-bcmath/src/ops.rs`

**Interfaces:**
- Produces: `BcNum`, `parse_bcmath_number`, `format_bcmath_number`, `bc_add`/`bc_sub`/`bc_mul`/`bc_div`/`bc_mod`/`bc_comp`, `get_scale`/`set_scale`, `BcError` + `php_message()`, C ABI stubs for the ops implemented in this task.

- [ ] **Step 1: Create the crate package**

```toml
# crates/elephc-bcmath/Cargo.toml
[package]
name = "elephc-bcmath"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Pure-Rust PHP bcmath engine and C ABI for elephc"
publish = false

[lib]
crate-type = ["staticlib", "rlib"]
```

No third-party decimal crate. Optional later: `num-bigint` only inside `bcpowmod` (Task 2).

Do **not** add the crate to the workspace yet (Task 3). Develop it as a path crate via `cargo test --manifest-path crates/elephc-bcmath/Cargo.toml`.

- [ ] **Step 2: Write failing parse/format/add tests in the crate**

Put unit tests in each module (or `src/ops.rs` `#[cfg(test)]`) that encode PHP 8.4 behavior:

```rust
#[test]
fn parse_accepts_signed_decimal() {
    let n = parse_bcmath_number("-003.50").unwrap();
    assert_eq!(format_bcmath_number(&n, 2).unwrap(), "-3.50");
}

#[test]
fn parse_accepts_digitless_zero_forms() {
    for value in ["", "+", "-", ".", "+.", "-."] {
        let n = parse_bcmath_number(value).unwrap();
        assert_eq!(format_bcmath_number(&n, 2).unwrap(), "0.00");
    }
}

#[test]
fn parse_rejects_whitespace_and_scientific_notation() {
    assert!(parse_bcmath_number(" 0").is_err());
    assert!(parse_bcmath_number("0 ").is_err());
    assert!(parse_bcmath_number("1e2").is_err());
}

#[test]
fn add_truncates_to_scale_and_pads() {
    // PHP: bcadd('1.234', '5', 4) === '6.2340'
    // PHP: bcadd('1.234', '5') === '6'   (default scale 0)
    assert_eq!(bc_add("1.234", "5", Some(4)).unwrap(), "6.2340");
    assert_eq!(bc_add("1.234", "5", Some(0)).unwrap(), "6");
}

#[test]
fn div_truncates_does_not_round() {
    // PHP: bcdiv('10', '3', 2) === '3.33'  (not 3.34)
    assert_eq!(bc_div("10", "3", Some(2)).unwrap(), "3.33");
}

#[test]
fn div_by_zero_is_div_zero_error() {
    assert!(matches!(bc_div("1", "0", Some(0)), Err(BcError::DivisionByZero { .. })));
}

#[test]
fn scale_get_set_round_trips() {
    let old = set_scale(4).unwrap();
    assert_eq!(old, 0);
    assert_eq!(get_scale(), 4);
    assert_eq!(bc_add("1", "1", None).unwrap(), "2.0000");
    set_scale(0).unwrap();
}
```

Cross-check any doubtful string with `php -r` if `php` is installed. If it is not, use the PHP manuals plus the seeds in the spec.

- [ ] **Step 3: Run the crate tests and confirm they fail**

```bash
cargo test --manifest-path crates/elephc-bcmath/Cargo.toml
```

Expected: compile error or failing assertions (`parse_bcmath_number` not defined).

- [ ] **Step 4: Implement the engine**

`BcNum`:

```rust
pub struct BcNum {
    pub negative: bool,
    pub digits: Vec<u8>, // base-10, most significant digit first, no leading zeros except the value 0
    pub scale: i32,      // how many digits are after the decimal point
}
```

`parse.rs`: scan input verbatim, normalize syntactically valid digitless forms to zero, and reject whitespace, scientific notation, and other junk.

`format.rs`: emit PHP strings for a requested result scale (pad or truncate, never round). Normalize `-0` to `0` / `0.000`.

`ops.rs`: implement add/sub/mul by aligning scales, then truncate to the requested scale. Division and modulo follow PHP’s truncate-toward-zero / remainder-sign rules; lock remainder signs against:

```php
bcmod('5', '3')    // 2
bcmod('-5', '3')   // -2
bcmod('5', '-3')   // 2
bcmod('-5', '-3')  // -2
```

`scale.rs`: `AtomicI32`, default `0`, reject values outside `0..=2147483647`.

`error.rs`:

```rust
pub enum BcError {
    Malformed { func: &'static str, arg_pos: u32, arg_name: &'static str },
    ScaleRange { func: &'static str, arg_pos: u32 },
    DivisionByZero { func: &'static str },
    // variants used in Task 2 may be added now as unused or added later
}

impl BcError {
    pub fn status_code(&self) -> i32 { /* spec table 1..=8 */ }
    pub fn php_message(&self) -> String { /* php-src wording */ }
}
```

Copy messages from php-src / `php -r`. Typical shapes:

```text
bcadd(): Argument #1 ($num1) is not well-formed
bcadd(): Argument #3 ($scale) must be between 0 and 2147483647
```

`lib.rs`: `//!` preamble; `mod` declarations; `#[no_mangle] pub extern "C"` wrappers that never panic (`catch_unwind` → status). Allocate result bytes with `Vec<u8>` / `Box<[u8]>`, return pointer+len, free via `elephc_bcmath_free`.

C ABI for a binary op (scale optional):

```rust
/// Adds two BCMath numeric strings.
///
/// # Safety
/// Pointers must be valid for the given lengths. `out_ptr`/`out_len` must be writable.
#[no_mangle]
pub unsafe extern "C" fn elephc_bcmath_add(
    a_ptr: *const u8,
    a_len: usize,
    b_ptr: *const u8,
    b_len: usize,
    scale: i64,
    scale_is_null: i32,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> i32
```

Same shape for `sub`/`mul`/`div`/`mod`. `elephc_bcmath_comp` writes `*out_cmp: i32` (`-1`/`0`/`1`). `elephc_bcmath_get_scale` / `elephc_bcmath_set_scale` return/set the `i32` scale (`set` writes the previous value through an out pointer).

- [ ] **Step 5: Re-run crate tests**

```bash
cargo test --manifest-path crates/elephc-bcmath/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/elephc-bcmath
git commit -m "feat: add elephc-bcmath core decimal engine"
```

---

### Task 2: Remaining engine operations and fixture table

**Files:**
- Create: `crates/elephc-bcmath/src/pow.rs`
- Create: `crates/elephc-bcmath/src/round.rs`
- Create: `crates/elephc-bcmath/tests/php_fixtures.rs`
- Create: `crates/elephc-bcmath/tests/gen_bcmath_fixtures.php` (optional generator)
- Modify: `crates/elephc-bcmath/src/lib.rs` (mods + C ABI)
- Modify: `crates/elephc-bcmath/src/ops.rs` (`bc_divmod`)
- Modify: `crates/elephc-bcmath/src/error.rs` (remaining variants)

**Interfaces:**
- Consumes: `BcNum`, parse/format, scale, `BcError` from Task 1.
- Produces: `bc_pow`, `bc_powmod`, `bc_sqrt`, `bc_ceil`, `bc_floor`, `bc_round`, `bc_divmod`, matching C exports.

- [ ] **Step 1: Write failing fixture tests**

`tests/php_fixtures.rs` must include at least:

```text
bcadd('1.234','5')           → 6
bcadd('1.234','5',4)         → 6.2340
bcdiv('105','6.55957',3)     → 16.007
bcdiv('10','3',2)            → 3.33
bcmul('1.20','1.20',2)       → lock vs php -r (truncate)
bccomp('1','2')              → -1
bcsqrt('2',3)                → 1.414
bcpow('4.2','3',2)           → 74.08
bcpow('5','2',2)             → 25.00          (PHP 7.3+ pads scale)
bcpow('0','-1',0)            → DivisionByZeroError
bcceil('1.1')                → 2
bcfloor('1.9')               → 1
bcfloor('-1.1')              → -2
bcround('3.5')               → 4
bcround('5.045',2)           → 5.05
bcround('345',-2)            → 300
bcround('9.5',0,1..=8)       → PHP manual table
bcdivmod('5','3')            → [1, 2]
bcdivmod('5','-3')           → [-1, 2]
bcdivmod('-5','3')           → [-1, -2]
bcdivmod('-5','-3')          → [1, -2]
bcdivmod('5.7','1.3',1)      → [4, 0.5]
bcpowmod: integral args, modulus 0 → DivisionByZeroError
negative scale               → ValueError
"1e2"                        → ValueError
```

- [ ] **Step 2: Run fixtures, confirm fail**

```bash
cargo test --manifest-path crates/elephc-bcmath/Cargo.toml php_fixtures
```

Expected: FAIL (missing functions).

- [ ] **Step 3: Implement**

- `bcpow`: exponent must be a well-formed integer string (no fractional part). Range at least `i32::MIN..=i32::MAX`. Negative exponent of `0` is `DivisionByZero`. Result honors requested scale (pad zeros).
- `bcpowmod`: num, exponent, modulus must be integral; exponent ≥ 0; modulus ≠ 0. `num-bigint` is allowed here only.
- `bcsqrt`: negative → `ValueError`. Truncate to scale (PHP `bcsqrt('2', 3)` is `1.414`).
- `bcceil` / `bcfloor`: no scale argument; result scale 0.
- `bcround`: precision may be negative; mode `1..=8` matching existing `round()` (`PHP_ROUND_HALF_UP` … `AwayFromZero`). Invalid mode → `BcError::RoundMode`.
- `bcdivmod`: quotient scale 0; remainder uses requested scale; signs match the PHP manual table.

Export matching `elephc_bcmath_pow`, `_powmod`, `_sqrt`, `_ceil`, `_floor`, `_round`, `_divmod` C functions. `divmod` takes two out buffers.

- [ ] **Step 4: Run crate tests**

```bash
cargo test --manifest-path crates/elephc-bcmath/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/elephc-bcmath
git commit -m "feat: complete elephc-bcmath ops and PHP fixtures"
```

---

### Task 3: Workspace and bridge registration

**Files:**
- Modify: `Cargo.toml` (workspace `members`, `default-members`, `[workspace.dependencies]`, `[dev-dependencies]`)
- Modify: `src/linker/bridges.rs` (`BRIDGES` + existing `php_extension_for_lib` test)

**Interfaces:**
- Consumes: crate name `elephc-bcmath` / lib `elephc_bcmath`.
- Produces: `--with-bcmath` accepted by CLI; `php_extension_for_lib("elephc_bcmath") == Some("bcmath")`.

- [ ] **Step 1: Add workspace membership**

In root `Cargo.toml`:

- append `"crates/elephc-bcmath"` to **both** `members` and `default-members`
- add a matching dev-dependency (this repo has no `[workspace.dependencies]` table):

```toml
[dev-dependencies]
elephc-bcmath = { path = "crates/elephc-bcmath" }
```

Update the comment above `[workspace]` that lists which `libelephc_*.a` a plain `cargo build` materializes.

- [ ] **Step 2: Register the bridge**

In `src/linker/bridges.rs`, after the crypto entry (keep table order stable and documented):

```rust
BridgeStaticlib {
    lib_name: "elephc_bcmath",
    env_var: "ELEPHC_BCMATH_LIB_DIR",
    crate_name: "elephc-bcmath",
    flag_name: "bcmath",
    whole_archive: false,
    macos_frameworks: &[],
    needs_libdl: true,
    php_extension: Some("bcmath"),
},
```

Extend the existing unit test that asserts `php_extension_for_lib("elephc_crypto") == Some("hash")` with:

```rust
assert_eq!(php_extension_for_lib("elephc_bcmath"), Some("bcmath"));
```

`crate_flag_names()` length assertion already follows `BRIDGES.len()`; no extra update if it iterates the table.

- [ ] **Step 3: Verify compile and bridge tests**

```bash
cargo build
cargo test --lib php_extension_for_lib -- --nocapture
```

If the test filter is too narrow, run the module:

```bash
cargo test --lib bridges
```

Expected: warning-free build; `--with-bcmath` listed by an unknown `--with-nope` error (optional smoke: `cargo run -- --help` does not need to mention every crate). Confirm `target/debug/libelephc_bcmath.a` exists after `cargo build`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml crates/elephc-bcmath src/linker/bridges.rs
git commit -m "feat: register elephc-bcmath bridge and --with-bcmath"
```

---

### Task 4: `RuntimeFnId` contracts

**Files:**
- Modify: `src/ir/runtime_fn.rs`

**Interfaces:**
- Produces: `RuntimeFnId::BcAdd`, `BcSub`, `BcMul`, `BcDiv`, `BcMod`, `BcDivmod`, `BcPow`, `BcPowmod`, `BcSqrt`, `BcComp`, `BcScale`, `BcCeil`, `BcFloor`, `BcRound`.
- Each `requirements()` → `&[BuiltinRequirement::Bridge("elephc_bcmath")]`.
- `as_eir()` names: `bcadd`, `bcsub`, `bcmul`, `bcdiv`, `bcmod`, `bcdivmod`, `bcpow`, `bcpowmod`, `bcsqrt`, `bccomp`, `bcscale`, `bcceil`, `bcfloor`, `bcround`.

- [ ] **Step 1: Add the 14 enum variants**

Place them together near the other math IDs (`Abs`/`Round`/`Pow`) or at the end of the math cluster. Keep the enum compileable: update every exhaustive match this file owns (`effects`, `result_ownership`, `requirements`, `as_eir`, and any callable-support match).

- [ ] **Step 2: Set effects explicitly**

Do **not** fall through the default “almost all bits” arm.

```rust
RuntimeFnId::BcScale => Effects::from_bits_retain(
    Effects::READS_PROCESS.bits()
        | Effects::WRITES_PROCESS.bits()
        | Effects::MAY_THROW.bits(),
),
RuntimeFnId::BcComp => Effects::from_bits_retain(
    Effects::READS_PROCESS.bits() | Effects::MAY_THROW.bits(),
),
RuntimeFnId::BcAdd | RuntimeFnId::BcSub | /* all other Bc* string/array results */ => {
    Effects::from_bits_retain(
        Effects::READS_PROCESS.bits()
            | Effects::ALLOC_HEAP.bits()
            | Effects::MAY_THROW.bits(),
    )
}
```

- [ ] **Step 3: Ownership**

- `BcComp`, `BcScale` → `NonHeap`
- `BcDivmod` and every string-returning `Bc*` → `Fresh`

Add them to the existing `result_ownership()` Fresh list (same comment style as `Base64Decode`: the helper persists a new string / builds a new array).

- [ ] **Step 4: Requirements**

```rust
RuntimeFnId::BcAdd
| RuntimeFnId::BcSub
| RuntimeFnId::BcMul
| RuntimeFnId::BcDiv
| RuntimeFnId::BcMod
| RuntimeFnId::BcDivmod
| RuntimeFnId::BcPow
| RuntimeFnId::BcPowmod
| RuntimeFnId::BcSqrt
| RuntimeFnId::BcComp
| RuntimeFnId::BcScale
| RuntimeFnId::BcCeil
| RuntimeFnId::BcFloor
| RuntimeFnId::BcRound => &[BuiltinRequirement::Bridge("elephc_bcmath")],
```

- [ ] **Step 5: Compile**

```bash
cargo build
```

Expected: PASS. If an exhaustive match outside this file breaks, fix it in this task only when it is a compile error; lowering arms come in Task 6.

- [ ] **Step 6: Commit**

```bash
git add src/ir/runtime_fn.rs
git commit -m "feat: add RuntimeFnId contracts for bcmath"
```

---

### Task 5: AOT registry homes

**Files:**
- Create: `src/builtins/math/bcadd.rs`
- Create: `src/builtins/math/bcsub.rs`
- Create: `src/builtins/math/bcmul.rs`
- Create: `src/builtins/math/bcdiv.rs`
- Create: `src/builtins/math/bcmod.rs`
- Create: `src/builtins/math/bcdivmod.rs`
- Create: `src/builtins/math/bcpow.rs`
- Create: `src/builtins/math/bcpowmod.rs`
- Create: `src/builtins/math/bcsqrt.rs`
- Create: `src/builtins/math/bccomp.rs`
- Create: `src/builtins/math/bcscale.rs`
- Create: `src/builtins/math/bcceil.rs`
- Create: `src/builtins/math/bcfloor.rs`
- Create: `src/builtins/math/bcround.rs`
- Modify: `src/builtins/math/mod.rs` (alphabetical `pub mod`)

**Interfaces:**
- Consumes: `RuntimeFnId::Bc*` from Task 4.
- Produces: catalog names, signatures, `function_exists`, named-arg keys matching PHP (`num1`, `num2`, `scale`, `num`, `exponent`, `modulus`, `precision`, `mode`).

- [ ] **Step 1: Add one home per function**

Every file: `//!` preamble + `builtin!`. Binary ops use `DefaultSpec::Null` for `$scale`. Examples:

`bcadd.rs` / `bcsub.rs` / `bcmul.rs` / `bcdiv.rs` / `bcmod.rs`:

```rust
params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
returns: Str,
semantics: runtime_fn_semantics(RuntimeFnId::BcAdd), // matching id
```

`bcpow.rs`: `params: [num: Str, exponent: Str, scale: Int = DefaultSpec::Null], returns: Str`

`bcpowmod.rs`: `params: [num: Str, exponent: Str, modulus: Str, scale: Int = DefaultSpec::Null], returns: Str`

`bcsqrt.rs`: `params: [num: Str, scale: Int = DefaultSpec::Null], returns: Str`

`bccomp.rs`: same params as `bcadd`, `returns: Int`

`bcscale.rs`: `params: [scale: Int = DefaultSpec::Null], returns: Int`

`bcceil.rs` / `bcfloor.rs`: `params: [num: Str], returns: Str`

`bcround.rs`:

```rust
params: [
    num: Str,
    precision: Int = DefaultSpec::Int(0),
    mode: Int = DefaultSpec::Int(1)
],
returns: Str,
```

`bcdivmod.rs`: `returns: Mixed` plus a `check` hook that returns `PhpType::Array(Box::new(PhpType::Str))` after inferring arguments (so the checker sees an array of strings). If `TypeSpec` cannot express that array, keep `Mixed` and document it in the preamble; do not invent a new `TypeSpec` variant unless one already exists.

`php_manual` URLs: `https://www.php.net/manual/en/function.<name>.php`.

- [ ] **Step 2: Register modules alphabetically**

In `src/builtins/math/mod.rs` add `pub mod bcadd;` … immediately after the file preamble / existing `use`s, keeping the `pub mod` list alphabetical (`bcadd` before `ceil`, etc.).

- [ ] **Step 3: Build**

```bash
cargo build
```

Expected: PASS. Registry inventory picks the homes up automatically.

- [ ] **Step 4: Commit**

```bash
git add src/builtins/math
git commit -m "feat: register PHP bcmath builtins"
```

---

### Task 6: AOT lowering and runtime helpers

**Files:**
- Create: `src/codegen_support/bcmath.rs` (slot publisher)
- Create: `src/codegen/lower_inst/builtins/bcmath.rs`
- Create: `src/codegen_support/runtime/bcmath/mod.rs` (and split if >500 LoC / mixed responsibility)
- Create: `src/codegen/lower_inst/runtime_functions/group_13.rs`
- Modify: `src/codegen/lower_inst/runtime_functions.rs` (mod + dispatch)
- Modify: `src/codegen/lower_inst/builtins.rs` (`pub(crate) mod bcmath`)
- Modify: `src/codegen_support/runtime/mod.rs` (emit the new helpers)
- Modify: runtime data section if ValueError message symbols are interned there

**Interfaces:**
- Consumes: C ABI from Tasks 1–2, `RuntimeFnId` from Task 4.
- Produces: `__rt_bcadd` … `__rt_bcround` that every supported target can call.

- [ ] **Step 1: Slot publisher**

Mirror `src/codegen_support/hash_crypto.rs`:

```rust
pub(crate) fn publish_elephc_bcmath_function_pointers(emitter: &mut Emitter) {
    const ENTRIES: &[(&str, &str)] = &[
        ("elephc_bcmath_add", "_elephc_bcmath_add_fn"),
        ("elephc_bcmath_sub", "_elephc_bcmath_sub_fn"),
        // … every C export including get/set scale and free
    ];
    // AArch64: adr slot, str fn ptr
    // x86_64: store via emit_store_reg_to_symbol
}
```

Emit BSS slots next to the hash slots (same pattern as `_elephc_crypto_hash_fn`).

- [ ] **Step 2: Runtime helpers**

Each `__rt_bc*` (or one parameterized `__rt_bcmath_binop` plus specialists):

1. Receive PHP strings (ptr/len) and an optional scale (null flag + i64).
2. Publish is **not** done here; the call-site lowerer publishes so the staticlib is only referenced when used.
3. Call the slot.
4. On `BCMATH_ERR_DIV_ZERO`, throw catchable `DivisionByZeroError` using the existing `_spl_division_by_zero_error_class_id` path. Message from `BcError::php_message` baked as a data string, or a small family of fixed messages (`bcdiv(): Division by zero`, etc.).
5. On other non-zero statuses, throw catchable `ValueError` with the matching php-src message.
6. On success, persist the returned bytes as a PHP string (`__rt_str_persist` or the current equivalent) and free the crate buffer via `elephc_bcmath_free`.
7. `bcdivmod`: allocate an indexed array of two strings.
8. `bccomp` / `bcscale`: return `i64` in the integer result register.
9. Support **both** AArch64 and x86_64 in the same helper file via `emitter.target` / existing ABI helpers. No ARM64-only path.

Every `emitter.instruction(...)` needs a `//` comment at column 81. Group with `// -- description --`.

- [ ] **Step 3: Lowering**

`src/codegen/lower_inst/builtins/bcmath.rs`:

- `ensure_arg_count_between` per signature.
- Load string operands through existing string-arg helpers.
- Omitted scale / PHP `null` → `scale_is_null = 1`. Explicit int → `scale_is_null = 0`.
- `bcround` `$mode`: either pass through to the crate (crate rejects out of `1..=8`) or reuse the `round_mode.rs` `1..=8` guard with the `bcround(): Argument #3 ($mode) must be a valid rounding mode (RoundingMode::*)` message. Prefer **one** place: the crate, so Magician matches.
- Call `publish_elephc_bcmath_function_pointers`, then `abi::emit_call_label(..., "__rt_bcadd")` (etc.).
- `store_if_result`.

- [ ] **Step 4: Dispatch group 13**

New `group_13.rs` matching the group_00 preamble. Match all 14 `RuntimeFnId::Bc*` onto `builtins::bcmath::lower_*`. Wire it in `runtime_functions.rs` after `group_12`.

- [ ] **Step 5: Build and comment check**

```bash
cargo build
./scripts/check_asm_comments.py src/codegen/lower_inst/builtins/bcmath.rs src/codegen_support/runtime/bcmath src/codegen_support/bcmath.rs
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/codegen src/codegen_support
git commit -m "feat: lower bcmath builtins through elephc-bcmath"
```

---

### Task 7: AOT codegen and error tests

**Files:**
- Create: `tests/codegen/math/bcmath.rs`
- Modify: `tests/codegen/math.rs` (`#[path = "math/bcmath.rs"] mod bcmath;`)
- Modify: `tests/error_tests/math_builtins.rs`
- Modify: `tests/extension_loaded_tests.rs` (or a new focused test next to it)

**Interfaces:**
- Consumes: compiled `bc*` builtins from Tasks 5–6.

- [ ] **Step 1: Write failing codegen tests**

```rust
//! Purpose:
//! End-to-end codegen coverage for the PHP bcmath procedural functions.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through Rust's test harness.
//!
//! Key details:
//! - Fixtures compile PHP to a native binary and assert stdout.
//! - Runtime-unknown `$argc` keeps values off the AST folder where needed.

#[test]
fn test_bcadd_explicit_scale() {
    let out = compile_and_run(r#"<?php echo bcadd('1.234', '5', 4);"#);
    assert_eq!(out, "6.2340");
}

#[test]
fn test_bcscale_then_omitted_scale() {
    let out = compile_and_run(
        r#"<?php
bcscale(3);
echo bcdiv('105', '6.55957');
"#,
    );
    assert_eq!(out, "16.007");
}

#[test]
fn test_bcdiv_by_zero_is_catchable() {
    let out = compile_and_run(
        r#"<?php
try { echo bcdiv('1', '0'); } catch (\DivisionByZeroError $e) { echo get_class($e); }
"#,
    );
    assert_eq!(out, "DivisionByZeroError");
}

#[test]
fn test_malformed_is_value_error() {
    let out = compile_and_run(
        r#"<?php
$s = '1e2';
try { echo bcadd($s, '1'); } catch (\ValueError $e) { echo get_class($e); }
"#,
    );
    assert_eq!(out, "ValueError");
}

#[test]
fn test_bcmath_case_insensitive_and_named_args() {
    let out = compile_and_run(
        r#"<?php echo BCADD(num1: '1', num2: '2', scale: 0);"#,
    );
    assert_eq!(out, "3");
}

#[test]
fn test_bcdivmod_signs() {
    let out = compile_and_run(
        r#"<?php
[$q, $r] = bcdivmod('-5', '3');
echo $q, '|', $r;
"#,
    );
    assert_eq!(out, "-1|-2");
}

#[test]
fn test_bcround_modes() {
    let out = compile_and_run(
        r#"<?php echo bcround('9.5', 0, 1), '|', bcround('9.5', 0, 2);"#,
    );
    assert_eq!(out, "10|9");
}

#[test]
fn test_function_exists_bcadd() {
    let out = compile_and_run(
        r#"<?php echo function_exists('bcadd') ? 'yes' : 'no';"#,
    );
    assert_eq!(out, "yes");
}
```

Also cover `bcsub`, `bcmul`, `bcmod`, `bcpow`, `bcsqrt`, `bccomp`, `bcceil`, `bcfloor`, `bcscale()` getter, and `bcpowmod` with a small integral triple.

Error tests:

```rust
#[test]
fn test_error_bcadd_too_few_args() {
    expect_error("<?php bcadd('1');", "bcadd() takes");
}
```

Match the registry’s actual arity wording (`takes 2 to 3 arguments` etc.).

Extension-loaded:

```php
// no bc* use
echo extension_loaded('bcmath') ? 'yes' : 'no';  // no
```

```php
echo bcadd('1','2');
echo extension_loaded('bcmath') ? 'yes' : 'no';  // yes
```

`--with-bcmath` on a program that only calls `extension_loaded('bcmath')` must print `yes`.

- [ ] **Step 2: Run tests, confirm they fail for the right reason**

```bash
cargo test --test codegen_tests test_bcadd_explicit_scale
```

Expected: FAIL until Task 6 is complete; after Task 6 they should start passing as they are filled in. If Task 6 already landed, this step is write-and-run-to-green.

- [ ] **Step 3: Make them pass**

Fix lowering, messages, or scale-null handling until the focused tests pass. Do not weaken fixtures to match a wrong engine.

```bash
cargo test --test codegen_tests bcmath
cargo test --test error_tests bcadd
```

- [ ] **Step 4: Commit**

```bash
git add tests/codegen/math tests/codegen/math.rs tests/error_tests/math_builtins.rs tests/extension_loaded_tests.rs
git commit -m "test: cover AOT bcmath builtins"
```

---

### Task 8: Magician registry and implementations

**Files:**
- Create: `crates/elephc-magician/src/interpreter/builtins/math/bcadd.rs` … `bcround.rs` (14 files)
- Modify: `crates/elephc-magician/src/interpreter/builtins/math/mod.rs`
- Modify: `crates/elephc-magician/src/interpreter/builtins/hooks/direct.rs`
- Modify: `crates/elephc-magician/src/interpreter/builtins/hooks/values.rs`
- Modify: `crates/elephc-magician/src/interpreter/builtins/hooks/arity.rs` if arity tables are derived from the registry (prefer registry-only)
- Modify: `crates/elephc-magician/src/interpreter/builtins/network_env/extension_loaded.rs`
- Modify: `crates/elephc-magician/src/interpreter/builtins/network_env/get_loaded_extensions.rs`
- Modify: `crates/elephc-magician/Cargo.toml`

**Interfaces:**
- Consumes: crate C ABI / Rust wrappers from Tasks 1–2.
- Produces: eval `bc*` names in the magician registry.

- [ ] **Step 1: Depend on the crate**

```toml
elephc-bcmath = { path = "../elephc-bcmath" }
```

- [ ] **Step 2: Add one shared hook**

In `EvalDirectHook` and `EvalValuesHook`, add a single `Bcmath` variant. Dispatch by looking up the builtin name (or a small match on `"bcadd"` …) and calling a shared helper. Do **not** add 14 hook enum variants.

- [ ] **Step 3: Home files**

Each home:

```rust
eval_builtin! {
    name: "bcadd",
    area: Math,
    params: [num1, num2, scale = EvalBuiltinDefaultValue::Null],
    direct: Bcmath,
    values: Bcmath,
}
```

Implementation:

1. Evaluate args.
2. Coerce operands to strings the way PHP would (`int`/`float` → decimal string). Do not send raw Zend-style floats as `"1e-2"` if PHP’s string cast would; lock with `php -r 'echo (string)(float)$x;'`.
3. `scale` default/null → `scale_is_null = 1`.
4. Call `elephc_bcmath_add`.
5. Status ≠ 0 → raise the matching Magician throwable (`ValueError` / `DivisionByZeroError`) with `BcError::php_message()` (reconstruct from status + function name, or expose a Rust API `bc_add(...) -> Result<String, BcError>` and prefer the Rust API inside Magician to avoid re-encoding).
6. Prefer calling the crate’s **Rust** functions (`bc_add`) from Magician and keep C ABI for AOT only, **if** that is easier **and** scale still goes through `get_scale`/`set_scale` in the same crate instance. Mixed-process identity is Task 9’s gate.

- [ ] **Step 4: `extension_loaded`**

Add `"bcmath"` to **both** Magician `CORE_LOADED_EXTENSIONS` copies (extension_loaded.rs and get_loaded_extensions.rs). Keep the “KEEP IN SYNC” comment honest: eval now reports `bcmath` because it implements the functions; AOT still reports it only when the bridge is linked.

- [ ] **Step 5: Compile magician**

```bash
cargo test -p elephc-magician --lib --no-run
```

Expected: compiles.

- [ ] **Step 6: Commit**

```bash
git add crates/elephc-magician
git commit -m "feat: add magician bcmath builtins"
```

---

### Task 9: Magician tests and mixed scale

**Files:**
- Create: `crates/elephc-magician/src/interpreter/tests/builtins_bcmath.rs`
- Modify: `crates/elephc-magician/src/interpreter/tests/mod.rs`
- Modify: `tests/codegen/math/bcmath.rs` (mixed AOT+eval case)
- Modify: `tests/codegen/eval_builtin_parity.rs` only if that file enumerates names by hand

- [ ] **Step 1: Write Magician tests**

Reuse the Task 2 seeds through `parse_fragment` / `execute_program`, same style as `builtins_math_formatting.rs`:

```rust
#[test]
fn execute_program_bcadd_and_bcscale() {
    let program = parse_fragment(
        br#"bcscale(4); echo bcmul("1", "1"); return function_exists("bcadd");"#,
    )
    .expect("parse");
    // assert output "1.0000" and function_exists true
}
```

Include malformed → throwable, div by zero, `bcdivmod` signs, `extension_loaded('bcmath') === true`.

- [ ] **Step 2: Mixed AOT + eval scale (the gate)**

```rust
#[test]
fn test_bcscale_shared_with_eval() {
    let out = compile_and_run(
        r#"<?php
bcscale(4);
eval('echo bcmul("1", "1");');
"#,
    );
    assert_eq!(out, "1.0000");
}
```

If this fails because two crate copies exist, apply the fallback from the spec **in this task**: Magician uses `extern "C"` only; pipeline force-links `elephc_bcmath` whenever Magician is linked. Re-run the mixed test until it passes. Do not land split scale.

- [ ] **Step 3: Run focused tests**

```bash
cargo test -p elephc-magician builtins_bcmath
cargo test --test codegen_tests test_bcscale_shared_with_eval
cargo test --test builtin_parity_tests
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/elephc-magician tests/codegen/math/bcmath.rs
git commit -m "test: cover magician bcmath and shared scale"
```

---

### Task 10: Example, docs, generated pages

**Files:**
- Create: `examples/bcmath/main.php`
- Create: `examples/bcmath/.gitignore` (`*.s`, `*.o`, `main`)
- Create: `docs/php/bcmath.md`
- Modify: `docs/README.md`, `docs/php/math.md` (one-line pointer)
- Modify: `README.md` (builtin list)
- Generated: `docs/php/builtins.md`, `docs/php/builtins/**`, `docs/internals/builtins/**`, `scripts/docs/builtin_registry.json`

**Interfaces:**
- Consumes: finished AOT+Magician surface.

- [ ] **Step 1: Example**

Small readable program, not a test dump:

```php
<?php
bcscale(2);
$net = '19.99';
$tax = bcmul($net, '0.22', 2);
$gross = bcadd($net, $tax, 2);
echo $net, " + tax ", $tax, " = ", $gross, "\n";
echo "cmp ", bccomp($gross, '24.39', 2), "\n";
```

- [ ] **Step 2: User docs**

`docs/php/bcmath.md`:

```yaml
---
title: "BCMath"
description: "Arbitrary-precision decimal arithmetic: the 14 PHP bcmath functions."
sidebar:
  order: 22
---
```

No top-level `#` heading. Document signatures, scale, truncate-vs-round, errors, `extension_loaded`, `--with-bcmath`, and the explicit non-goals (`BcMath\Number`, `bcmath.scale` ini).

Link it from `docs/README.md` next to Math. Add one sentence on `docs/php/math.md` pointing at the new page.

README: add the 14 names to the math/system builtin list in the same style as neighboring entries.

- [ ] **Step 3: Generated builtin docs**

Follow the `update-builtin-docs` skill:

```bash
cargo build --example gen_builtins
python3 scripts/docs/extract_builtins.py --render --force
python3 scripts/docs/audit_builtins.py
python3 scripts/docs/elephc_builtins/validate_site_compat.py
git diff --check
```

Do not hand-edit generated pages.

Compatibility table (`docs/php/compatibility.md`) is generated; regenerate if the repo’s comparison script is part of that workflow (`python3 scripts/docs/gen_php_comparison.py` when that is how `compatibility.md` is produced).

- [ ] **Step 4: Focused verification**

```bash
cargo build
cargo test --test codegen_tests bcmath
cargo test -p elephc-magician builtins_bcmath
cargo test --test builtin_parity_tests
git diff --check
```

- [ ] **Step 5: Commit**

```bash
git add examples/bcmath docs README.md scripts/docs/builtin_registry.json
git commit -m "docs: document bcmath functions and example"
```

---

## Self-review

**Spec coverage:** 14 functions, shared crate, AOT+Magician, scale, errors, extension_loaded, tests, docs, example, non-goals (`BcMath\Number`, ini). Mixed-process scale has an explicit fallback.

**Placeholders:** none. Messages that must match php-src are identified as “copy from php-src / `php -r`”, not “TBD”.

**Type consistency:** `RuntimeFnId::BcAdd` … `BcRound`, C names `elephc_bcmath_*`, bridge `elephc_bcmath`, flag `bcmath`, Magician hook `Bcmath`.
