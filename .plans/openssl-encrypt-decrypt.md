# Plan: `openssl_encrypt` / `openssl_decrypt` (+ helpers)

## Task

Ship PHP-compatible symmetric crypto builtins on **both** the AOT path and
`elephc-magician`, with a single pure-Rust implementation behind
`elephc-crypto`:

- [ ] `openssl_encrypt`
- [ ] `openssl_decrypt`
- [ ] `openssl_cipher_iv_length`
- [ ] `openssl_get_cipher_methods`
- [ ] Predefined constants: `OPENSSL_RAW_DATA`, `OPENSSL_ZERO_PADDING`,
      `OPENSSL_DONT_ZERO_PAD_KEY`
- [ ] Shared bridge ABI in `crates/elephc-crypto` (RustCrypto, no system OpenSSL)
- [ ] AOT registry + `RuntimeFnId` + runtime helpers + lowering (all supported
      targets)
- [ ] Magician `eval_builtin!` homes calling the same bridge ABI
- [ ] Codegen + magician + crypto unit tests, example, docs, CHANGELOG

This is one PR on branch `feat/openssl-encrypt-decrypt`.

## Goal

Programs can encrypt/decrypt with the common AES modes PHP applications use,
with PHP-shaped signatures (including AEAD `$tag` / `$aad`), options flags, and
`string|false` failure modes — both when compiled natively and when evaluated
through magician. Magician and AOT must not diverge in cipher matrix or flag
semantics: both call `elephc-crypto`.

## Non-goals

- Full parity with every name in stock PHP `openssl_get_cipher_methods()`
  (135+ methods including wrap, SIV, CTS-HMAC-ETM, ARIA, Camellia, …).
- Asymmetric OpenSSL surface (`openssl_sign`, `openssl_verify`,
  `openssl_pkey_*`, `openssl_x509_*`, `openssl_random_pseudo_bytes`, …).
- Linking system `libcrypto` / OpenSSL.
- Streaming/incremental cipher APIs.
- `openssl_error_string` / OpenSSL error queue (may add a minimal warning
  string later; not required for this PR if warnings are fixed messages).

Document the supported cipher matrix in user docs and in
`openssl_get_cipher_methods()` return value so the surface is honest.

## Supported cipher matrix (locked for this PR)

Canonical lowercase names (case-insensitive lookup, like PHP):

| Name | Key | IV | Mode notes |
|---|---|---|---|
| `aes-128-cbc` | 16 | 16 | PKCS#7 (default) / zero-pad if `OPENSSL_ZERO_PADDING` |
| `aes-192-cbc` | 24 | 16 | same |
| `aes-256-cbc` | 32 | 16 | same |
| `aes-128-ecb` | 16 | 0 | same padding rules; IV ignored/empty |
| `aes-192-ecb` | 24 | 0 | same |
| `aes-256-ecb` | 32 | 0 | same |
| `aes-128-ctr` | 16 | 16 | stream; no block padding |
| `aes-192-ctr` | 24 | 16 | same |
| `aes-256-ctr` | 32 | 16 | same |
| `aes-128-gcm` | 16 | 12 (default IV len) | AEAD; `$tag`, `$aad`, `$tag_length` |
| `aes-192-gcm` | 24 | 12 | AEAD |
| `aes-256-gcm` | 32 | 12 | AEAD |

`openssl_get_cipher_methods(false)` returns exactly these names (sorted like
PHP: alphabetical is fine if stable and tested).
`openssl_get_cipher_methods(true)` may return the same list for MVP (no OpenSSL
aliases such as `AES-128-CBC` as separate entries) **or** include uppercase
aliases if cheap; pick one and lock it in tests. Prefer **aliases = same list
for now** unless uppercase aliases are free to emit.

Unknown cipher → PHP-style warning + `false` for encrypt/decrypt; `false` for
`openssl_cipher_iv_length`.

## PHP surface (signatures)

Cross-check with local PHP; values below match PHP 8.x:

```php
openssl_encrypt(
    string $data,
    string $cipher_algo,
    string $passphrase,
    int $options = 0,
    string $iv = "",
    string &$tag = null,   // by-ref output for AEAD encrypt
    string $aad = "",
    int $tag_length = 16
): string|false

openssl_decrypt(
    string $data,
    string $cipher_algo,
    string $passphrase,
    int $options = 0,
    string $iv = "",
    ?string $tag = null,   // by-value input for AEAD decrypt
    string $aad = ""
): string|false

openssl_cipher_iv_length(string $cipher_algo): int|false

openssl_get_cipher_methods(bool $aliases = false): array  // list of string
```

Constants (register as predefined int constants):

| Name | Value |
|---|---|
| `OPENSSL_RAW_DATA` | `1` |
| `OPENSSL_ZERO_PADDING` | `2` |
| `OPENSSL_DONT_ZERO_PAD_KEY` | `4` |

### Options / key / IV semantics (pin with `php -r` fixtures)

Implement to match PHP on the supported matrix; verify each rule with a
fixture before coding the edge case:

1. **`OPENSSL_RAW_DATA`**: output/input is raw binary. Without it, encrypt
   base64-encodes ciphertext; decrypt base64-decodes input first.
2. **`OPENSSL_ZERO_PADDING`**: for CBC/ECB only; no PKCS#7. Encrypt requires
   plaintext length multiple of block size or fails; decrypt does not strip
   PKCS#7.
3. **Key length**:
   - shorter than required → zero-pad **unless** `OPENSSL_DONT_ZERO_PAD_KEY`
     (then return `false` and warn that the key length cannot be set);
   - longer → truncate to required length.
4. **IV length**:
   - CBC/CTR empty or short IV → zero-pad to the required length; long IV →
     truncate. PHP warns on encrypt in all three cases and on decrypt for a
     short/long IV, but not for an empty IV;
   - GCM accepts any non-empty IV length (12 is the reported/default length);
     empty IV returns `false`.
5. **GCM**:
   - encrypt writes authentication tag into by-ref `$tag` (length
     `$tag_length`, default 16, observed valid range 1..16);
   - decrypt takes `$tag` by value; auth failure → `false`;
   - `$aad` optional additional authenticated data.
6. **Failure** returns `false` (bool), not empty string. Emit warnings
   consistent with PHP wording where practical (`Unknown cipher algorithm`,
   IV length messages, etc.).

## Architecture

Mirror the `hash` / `hash_hmac` stack:

```text
PHP call
  ├─ AOT:  builtin! → RuntimeFnId → lower → __rt_openssl_* → elephc_crypto_*
  └─ magician: eval_builtin! → elephc_crypto_* (same ABI)
```

Single source of cipher tables and encrypt/decrypt algorithms lives in
`crates/elephc-crypto`. Runtime and magician only do PHP glue (base64 option,
warnings, by-ref tag store, Mixed/`false` boxing).

### Why not system OpenSSL?

- `elephc-crypto` is already pure-Rust / musl-friendly for Docker Linux tests.
- PHAR signing uses pure-Rust `rsa`, not OpenSSL.
- System OpenSSL would complicate linking and the “no runtime deps” story for
  hash-family-adjacent features.

## Design: `elephc-crypto` ABI

### New modules

```text
crates/elephc-crypto/
  Cargo.toml          # + aes, cipher, ctr, ghash, subtle
  src/
    lib.rs            # re-export / register new externs
    algos.rs          # existing hash
    hmac.rs           # existing
    cipher.rs         # NEW: name table, key/iv normalize, dispatch
    cipher/abi.rs     # NEW: panic-contained C ABI + output sizing
    cipher/block.rs   # NEW: CBC/ECB padding and CTR dispatch
    cipher/gcm.rs     # NEW: runtime-IV/tag AES-GCM composition
  tests/
    openssl_php_fixtures.rs      # fixture schema/inventory
    openssl_php_fixtures/abi.rs  # ABI/NIST/PHP-golden tests
```

Prefer splitting if `cipher.rs` would exceed the soft 500 LoC cohesion
guideline with mixed concerns; a single cohesive cipher engine leaf is OK if
it stays one mental model.

### Suggested C ABI

Keep status codes simple and panic-free across the `extern "C"` boundary.

```rust
// Returns: 0 ok, negative error code
// Error codes (example; lock in code + docs):
//  -1 unknown cipher
//  -2 bad key (DONT_ZERO_PAD_KEY and wrong length)
//  -3 bad iv length
//  -4 bad plaintext length (ZERO_PADDING + non-block-aligned)
//  -5 decrypt/auth/padding failure
//  -6 bad tag_length
//  -7 output buffer too small
//  -8 invalid options or pointer arguments

#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_cipher_iv_length(
    name_ptr: *const u8,
    name_len: usize,
) -> isize; // >=0 length, -1 unknown

#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_cipher_methods(
    aliases: i32,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> isize; // count written; names are trailing-NUL packed

#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_encrypt(
    cipher_ptr: *const u8, cipher_len: usize,
    data_ptr: *const u8, data_len: usize,
    key_ptr: *const u8, key_len: usize,
    iv_ptr: *const u8, iv_len: usize,
    options: u32,
    aad_ptr: *const u8, aad_len: usize,
    tag_len: usize,
    out_ptr: *mut u8, out_cap: usize, out_len: *mut usize,
    tag_out_ptr: *mut u8, tag_out_cap: usize, tag_out_len: *mut usize,
) -> i32;

#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_decrypt(
    cipher_ptr: *const u8, cipher_len: usize,
    data_ptr: *const u8, data_len: usize,
    key_ptr: *const u8, key_len: usize,
    iv_ptr: *const u8, iv_len: usize,
    options: u32,
    aad_ptr: *const u8, aad_len: usize,
    tag_ptr: *const u8, tag_len: usize,
    out_ptr: *mut u8, out_cap: usize, out_len: *mut usize,
) -> i32;
```

**Base64:** implement in the PHP glue layer (`__rt_*` and magician), not in the
crypto ABI. The bridge always deals in raw ciphertext bytes. That keeps the
bridge free of PHP option bit knowledge except for padding/key flags that
affect the cipher itself (`ZERO_PADDING`, `DONT_ZERO_PAD_KEY`). Alternatively
pass full `options` into the bridge and let it ignore `RAW_DATA` — either is
fine if documented; prefer **bridge ignores `OPENSSL_RAW_DATA`**, glue handles
base64.

**Output sizing:** caller provides a generous buffer (e.g. `data_len + 32` for
encrypt and `data_len` for decrypt raw). On `-7`, the ABI still writes the
required ciphertext/plaintext and tag lengths so callers can retry. The packed
method-list ABI follows the same length-query rule.

### Dependencies (`Cargo.toml`)

Locked RustCrypto stack compatible with existing `digest 0.10`:

- `aes`
- `cipher`
- `ctr`
- `ghash`
- `subtle`

CBC/ECB padding is implemented locally so failures map to stable bridge status
codes. GCM is composed from RustCrypto AES + GHASH because PHP accepts arbitrary
non-empty runtime IV lengths and 1..=16-byte tags, while the high-level
`aes-gcm` type encodes both sizes at compile time.

Avoid `openssl` crate. Keep musl/Docker test images working without new system
packages.

### Unit tests (`crates/elephc-crypto/tests`)

- NIST / well-known AES-CBC and AES-GCM vectors.
- Round-trip every supported cipher.
- Key short/long + `DONT_ZERO_PAD_KEY`.
- GCM tag mismatch → failure.
- Unknown cipher → error code.
- Optional: golden bytes produced by `php -r` committed as constants.

## Design: AOT path

### 1. Builtin homes

```text
src/builtins/string/openssl_encrypt.rs
src/builtins/string/openssl_decrypt.rs
src/builtins/string/openssl_cipher_iv_length.rs
src/builtins/string/openssl_get_cipher_methods.rs
```

Register in `src/builtins/string/mod.rs`.

Patterns to copy:

| Builtin | Model after |
|---|---|
| `openssl_encrypt` | `gzuncompress` (`string\|false` via `check`) + `preg_match` (`ref tag`, likely `lazy_check`) |
| `openssl_decrypt` | `gzuncompress` return union; tag is **not** by-ref |
| `openssl_cipher_iv_length` | `int\|false` via `check` → `Union([Int, False])` |
| `openssl_get_cipher_methods` | `hash_algos` → `Array(Str)` via `check` |

Sketch for encrypt:

```rust
builtin! {
    name: "openssl_encrypt",
    area: String,
    params: [
        data: Str,
        cipher_algo: Str,
        passphrase: Str,
        options: Int = DefaultSpec::Int(0),
        iv: Str = DefaultSpec::Str(""),
        ref tag: Mixed = DefaultSpec::Null,
        aad: Str = DefaultSpec::Str(""),
        tag_length: Int = DefaultSpec::Int(16),
    ],
    returns: Mixed,
    check: check,
    lazy_check: true, // if $tag write-only / may be undefined pre-call
    semantics: runtime_fn_semantics(RuntimeFnId::OpensslEncrypt),
    summary: "...",
    php_manual: "https://www.php.net/manual/en/function.openssl-encrypt.php",
}
```

`check` must:

- infer non-ref args;
- if `$tag` present, require `ExprKind::Variable` (like `preg_match`);
- return `PhpType::Union(vec![PhpType::Str, PhpType::False])`.

For non-AEAD ciphers, `$tag` may be ignored (PHP allows passing it); do not
require AEAD when `$tag` is present.

### 2. Constants

```text
src/types/openssl_constants.rs
```

`OPENSSL_INT_CONSTANTS: &[(&str, i64)]` with the three flags. Wire into:

- type checker predefined constants init (`src/types/checker/driver` or
  equivalent site used by `json_constants` / `stream_constants`);
- codegen prescan materialization if other int constant tables are listed
  there.

Magician must expose the same constant names/values (follow how JSON/stream
constants are registered on the eval side).

### 3. `RuntimeFnId` + requirements

In `src/ir/runtime_fn.rs`:

- `OpensslEncrypt`
- `OpensslDecrypt`
- `OpensslCipherIvLength`
- `OpensslGetCipherMethods`

`requirements()`:

```rust
BuiltinRequirement::Bridge("elephc_crypto")
```

for all four (methods list can live in the bridge so AOT and magician share
one table). Effects: at least `MAY_WARN` for encrypt/decrypt/iv_length; methods
can be pure.

Name strings / docs metadata entries as for other runtime fns.

### 4. Function-pointer slots + publish

Extend `src/codegen_support/runtime/data/fixed.rs`:

```text
_elephc_crypto_encrypt_fn
_elephc_crypto_decrypt_fn
_elephc_crypto_cipher_iv_length_fn
_elephc_crypto_cipher_methods_fn   # if needed
```

Extend `src/codegen_support/hash_crypto.rs`
`publish_elephc_crypto_function_pointers` (or rename module later; not required
in this PR) with the new entries. Call publish from each openssl lowerer before
`__rt_*` calls — same link-laziness rule as `hash()`.

### 5. Runtime helpers

New leaf emitters under e.g.:

```text
src/codegen_support/runtime/strings/openssl_encrypt.rs
src/codegen_support/runtime/strings/openssl_decrypt.rs
src/codegen_support/runtime/strings/openssl_cipher_iv_length.rs
src/codegen_support/runtime/strings/openssl_get_cipher_methods.rs
```

Or one `openssl.rs` if still cohesive.

Helpers:

| Label | Behavior |
|---|---|
| `__rt_openssl_encrypt` | args → optional base64 encode of result → string or false; write `$tag` if AEAD |
| `__rt_openssl_decrypt` | optional base64 decode of input → decrypt → string or false |
| `__rt_openssl_cipher_iv_length` | int or false |
| `__rt_openssl_get_cipher_methods` | build indexed array of strings |

Reuse existing runtime string alloc, base64 (`__rt_base64_encode` / decode if
present), warning emission, and false-constant materialization patterns from
`gzuncompress` / `hex2bin` / `inet_pton`.

**By-ref `$tag` on encrypt:** follow `preg_match` / mutating builtin argument
lowering. Ensure EIR lowering passes a ref cell / pointer for the tag slot;
runtime stores a fresh string into that slot on success for GCM. For non-GCM,
leave tag unchanged or set empty — match PHP.

### 6. Codegen lowering

In `src/codegen/lower_inst/runtime_functions/` (appropriate group) and/or
`src/codegen/lower_inst/builtins/strings.rs`:

- dispatch `RuntimeFnId::Openssl*` → lowerers supporting **AArch64 and x86_64**;
- argument materialization via ABI helpers (no hardcoded single-arch paths);
- publish crypto pointers then `emit_call_label` to `__rt_openssl_*`.

Ownership: result is `Fresh` string or non-heap false — set
`semantics` ownership in the builtin descriptor accordingly
(`Fresh` / `NonHeap` may need a shared result type if mixed; follow other
`string|false` builtins).

### 7. Optimizer / effects

Update `src/optimize/effects/` if new builtins are classified specially; default
registry effects from `RuntimeFnId` should mark encrypt/decrypt as impure /
warning-capable so DCE does not drop them.

## Design: Magician path

### Homes

```text
crates/elephc-magician/src/interpreter/builtins/string/openssl_encrypt.rs
crates/elephc-magician/src/interpreter/builtins/string/openssl_decrypt.rs
crates/elephc-magician/src/interpreter/builtins/string/openssl_cipher_iv_length.rs
crates/elephc-magician/src/interpreter/builtins/string/openssl_get_cipher_methods.rs
```

Wire `mod` in `string/mod.rs`.

Use `eval_builtin!` with:

- defaults matching PHP;
- `ref tag` only on encrypt;
- `direct` + `values` hooks (new enum variants if needed, e.g.
  `OpensslEncrypt`, or a shared `OpensslCrypt` family like `HashOneShot`).

Implementation:

```rust
// call elephc_crypto::elephc_crypto_encrypt / decrypt / ...
// apply OPENSSL_RAW_DATA base64 in Rust glue
// return string cell or false cell
// on encrypt GCM: write tag into by-ref target via existing ref writeback helpers
```

Constants: register `OPENSSL_*` where magician predefined constants live
(search sibling pattern for `JSON_*` / stream constants).

### Magician tests

Extend or add:

```text
crates/elephc-magician/src/interpreter/tests/builtins_strings_openssl.rs
```

Cases: round-trip CBC/CTR/ECB/GCM, raw vs base64, named args, `function_exists`,
bad cipher → false, `openssl_cipher_iv_length`, `openssl_get_cipher_methods`
non-empty list containing `aes-256-gcm`.

## Tests (AOT / shared)

### Codegen

```text
tests/codegen/openssl/  # or tests/codegen/strings/openssl_*.rs
```

Minimum cases:

1. AES-256-CBC round-trip (default base64).
2. AES-256-CBC + `OPENSSL_RAW_DATA`.
3. AES-128-ECB round-trip.
4. AES-256-CTR round-trip.
5. AES-256-GCM round-trip with `$tag` and empty AAD; second case with AAD.
6. GCM decrypt wrong tag → false / empty output path.
7. Unknown cipher → false.
8. `openssl_cipher_iv_length('aes-128-cbc') === 16`, GCM `=== 12`, bad → false.
9. `openssl_get_cipher_methods()` contains expected names; count matches matrix.
10. Constants: `OPENSSL_RAW_DATA === 1` etc.
11. Case-insensitive cipher name (`AES-256-CBC`).
12. Named arguments / first-class callable smoke if other string builtins test it.

### Error tests

```text
tests/error_tests/... 
```

- wrong arity;
- `openssl_encrypt(..., $tag)` when `$tag` is not a variable (literal) if
  checker enforces it.

### Example

```text
examples/openssl_crypt/main.php
examples/openssl_crypt/.gitignore   # *.s, *.o, main
```

Small readable demo: encrypt then decrypt a string with AES-256-CBC and
AES-256-GCM (print tag length).

### Docs / changelog / registry

- Run builtin docs pipeline before PR:
  - `cargo build --example gen_builtins`
  - `python3 scripts/docs/extract_builtins.py --render --force`
  - `python3 scripts/docs/audit_builtins.py`
  - `python3 scripts/docs/elephc_builtins/validate_site_compat.py`
- User-facing note in `docs/php/` (strings or a short crypto subsection) listing
  supported ciphers and flags.
- `CHANGELOG.md` → `## [Unreleased]` one or two user-facing bullets.
- Do **not** bump crate versions in `Cargo.toml`.
- ROADMAP: only mark `[x]` if an existing item matches; do not invent roadmap
  entries for completed work.

## Implementation phases (single PR, sequential commits OK)

### Phase 0 — PHP golden fixtures

- [x] Script or checked-in vectors from `php -r` for every matrix cipher:
      ciphertext (raw hex), tag (GCM), iv_length, failure modes.
- [x] Lock empty-IV and short-key behavior to observed PHP on CI PHP version
      notes in the plan/tests comments.

Phase 0 baseline: PHP 8.4.19 CLI with OpenSSL 3.6.1. The repository CI does not
provision or pin PHP, so CI consumes the checked-in Rust fixture module rather
than regenerating it. Regeneration is explicit and stamps both PHP and OpenSSL
versions in the output. The corpus covers all 12 ciphers, GCM with/without AAD
and tag lengths 1/4/12/16, GCM IV lengths 1/12/16/20, IV lengths, base64 mode,
zero padding, short/long keys, and case-insensitive names. CBC and CTR both
carry empty/short/long-IV ciphertext plus the PHP warning observed on successful
encrypt/decrypt calls. Empty CBC/CTR/GCM plaintext is round-tripped and exported.
The corpus also contains 13 false-return failure modes with PHP-level warning
text where PHP emits one. Provider error-queue strings are intentionally
excluded because they vary with the OpenSSL build. The bridge's Elephc-specific
12-name method inventory is locked in Phase 1; its PHP-visible AOT exposure
remains a Phase 2 implementation test, not a PHP golden (stock PHP exposes a
much larger method list).

### Phase 1 — `elephc-crypto` cipher engine

- [x] Dependencies + `cipher` module + C ABI.
- [x] All matrix modes + key/iv/tag rules.
- [x] `cargo test -p elephc-crypto`.

### Phase 2 — AOT constants + non-AEAD encrypt/decrypt + helpers

- [x] `openssl_constants.rs` wired.
- [x] Builtins for all four functions (encrypt/decrypt may still ignore tag
      path initially **within the same PR branch**, but do not merge without AEAD).
- [x] Runtime + lowering for CBC/ECB/CTR + iv_length + get_cipher_methods.
- [x] Focused codegen tests for non-AEAD.

Phase 2 baseline: the three OpenSSL option constants resolve through the
checker, name resolver, and codegen prescan. All four PHP functions have
registry homes backed by typed `RuntimeFnId` entries and require
`elephc_crypto`. Target-aware AArch64 and x86_64 runtime glue publishes the
four cipher ABI entries, keeps base64 outside the bridge, boxes failures as
PHP `false`, and returns owned string/array results. CBC, ECB, and CTR are
covered by raw PHP-golden ciphertexts, base64/raw round trips, zero-padding,
failure, case-insensitive/namespaced-call, IV-length, constant, and exact
12-method-inventory tests. GCM calls deliberately return `false` until phase 3
provides the encrypt tag buffer/writeback and decrypt tag input.

### Phase 3 — AEAD (`$tag`, `$aad`, `$tag_length`)

- [x] Bridge GCM path complete.
- [x] AOT by-ref tag writeback on encrypt; by-value tag on decrypt.
- [x] Codegen GCM tests.

Phase 3 baseline: the existing bridge GCM implementation is now reachable from
the AOT runtime on AArch64 and x86_64. Encrypt allocates a bounded owned tag
buffer, transfers it into direct, pre-existing, named-argument, and ref-parameter
PHP locals only after authenticated GCM success, and releases it on non-AEAD or
failure paths. Decrypt forwards the by-value tag and AAD to the bridge. Focused
tests pin raw and base64 round trips, PHP ciphertext/tag goldens for all three
AES-GCM key sizes, tag lengths 1, 4, 12, and 16, empty plaintext, a 16-byte IV,
wrong/missing tags, wrong AAD, empty IV, invalid tag lengths, missing encrypt tag
targets, and tag-storage overwrite.

### Phase 4 — Magician

- [x] Four `eval_builtin!` homes + constants.
- [x] Magician tests mirroring AOT cases.

Phase 4 baseline: magician declares all four PHP functions in its eval registry
with the same names, parameters, defaults, and encrypt-only by-reference tag as
the AOT registry. The eval glue calls the shared raw `elephc-crypto` ABI, applies
base64 only at the PHP boundary, returns `false` for stable bridge failures, and
writes successful GCM tags back to direct and named caller storage. Predefined
OpenSSL option constants match AOT values. Focused interpreter tests mirror the
AOT corpus across CBC/CTR/ECB, every GCM key size, tag lengths 1/4/12/16, raw
and base64 modes, a 16-byte GCM IV, empty plaintext, exact method inventory,
helpers, named arguments, callable smoke coverage, tag overwrite, and failures.

### Phase 5 — Polish

- [ ] Example `examples/openssl_crypt/`.
- [ ] Docs + generated builtin registry.
- [ ] CHANGELOG.
- [ ] `cargo build` clean; focused tests green; `git diff --check`.

## Verification commands (focused; no full suite by default)

```bash
cargo build
cargo test -p elephc-crypto
cargo test --test codegen_tests openssl
cargo test -p elephc-magician openssl
# optional PHP cross-check:
ELEPHC_PHP_CHECK=1 cargo test --test codegen_tests openssl
git diff --check
```

Linux matrix: rely on CI unless a target-specific ABI bug appears; then use
`./scripts/test-linux-x86_64.sh openssl` / `test-linux-arm64.sh openssl`.

## Risk register

| Risk | Mitigation |
|---|---|
| PHP key/IV padding quirks | Golden fixtures from `php -r` before implementing |
| By-ref `$tag` AOT bugs | Copy `preg_match` lowering; dedicated GCM tests |
| Base64 option double-applied | Base64 only in PHP glue; bridge raw-only |
| Cipher matrix expectations | Docs + `get_cipher_methods` return only what works |
| File size / cohesion | Keep crypto engine in bridge; thin runtime glue |
| Magician/AOT drift | Same ABI; shared test vectors |
| x86_64 stack args for many params | Use existing ABI helpers; test encrypt with full arity |

## Acceptance criteria

1. All four builtins exist in AOT registry and magician eval registry.
2. Round-trip succeeds for every cipher in the locked matrix on AOT and magician.
3. GCM tag is produced on encrypt and required on decrypt.
4. Options `OPENSSL_RAW_DATA` and `OPENSSL_ZERO_PADDING` behave as PHP on CBC.
5. Unknown cipher returns `false` (and warning where applicable).
6. `openssl_cipher_iv_length` and `openssl_get_cipher_methods` match the matrix.
7. Constants resolve at compile time and at runtime.
8. Bridge links only when these (or other crypto) builtins are used
   (`elephc_crypto` requirement path).
9. All supported targets handled in the same change (no ARM64-only helper).
10. Example, docs, CHANGELOG, generated builtin docs updated.
11. Focused tests pass; no new compiler warnings.

## File checklist (expected touch set)

**Bridge**

- `crates/elephc-crypto/Cargo.toml`
- `crates/elephc-crypto/src/lib.rs`
- `crates/elephc-crypto/src/cipher.rs` (+ optional split files)
- `crates/elephc-crypto/tests/vectors.rs`

**AOT builtins / types / IR**

- `src/builtins/string/openssl_*.rs` (4 files)
- `src/builtins/string/mod.rs`
- `src/types/openssl_constants.rs` + registration site(s)
- `src/ir/runtime_fn.rs`
- `src/builtins/requirements.rs` only if table-driven extras needed

**Codegen / runtime**

- `src/codegen_support/runtime/data/fixed.rs`
- `src/codegen_support/hash_crypto.rs` (publish list)
- `src/codegen_support/runtime/strings/openssl_*.rs` or `openssl.rs`
- `src/codegen_support/runtime/emitters.rs` / `strings/mod.rs`
- `src/codegen/lower_inst/builtins/strings.rs` and/or `runtime_functions/group_*.rs`

**Magician**

- `crates/elephc-magician/src/interpreter/builtins/string/openssl_*.rs`
- `crates/elephc-magician/src/interpreter/builtins/string/mod.rs`
- hooks/spec enums if new variants required
- constants registration
- magician tests

**Product**

- `tests/codegen/...`
- `tests/error_tests/...`
- `examples/openssl_crypt/`
- `docs/php/...` + generated builtins
- `CHANGELOG.md`
- `scripts/docs/builtin_registry.json` (generated)

## Commit strategy (suggested)

1. `feat(crypto): add AES cipher ABI to elephc-crypto`
2. `feat: register OPENSSL_* constants and openssl_* AOT builtins`
3. `feat: runtime and lowering for openssl_encrypt/decrypt and helpers`
4. `feat(magician): eval openssl_encrypt/decrypt and helpers`
5. `test: openssl cipher matrix codegen and magician coverage`
6. `docs: openssl crypt builtins, example, changelog`

(Adjust squash style to repo preference; keep messages prefixed
`feat:` / `test:` / `docs:`.)

## Open implementation choices (decide during Phase 0–1, then lock)

1. **`get_cipher_methods(true)`:** same list vs uppercase aliases.
2. **Warning text:** copy PHP strings vs elephc-short messages (prefer PHP-like).
3. **Whether `openssl_encrypt` without GCM still type-checks `ref tag`:** yes;
   runtime no-ops tag for non-AEAD.

Once Phase 0 fixtures exist, do not change matrix or flag meanings without
updating this plan and the tests together.
