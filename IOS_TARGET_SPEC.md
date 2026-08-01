# iOS target specification

Tracking issue: [illegalstudio/elephc#662](https://github.com/illegalstudio/elephc/issues/662)
Branch: `feat/ios-target`
Status: draft — no code written yet

---

## 1. Purpose

Let elephc emit an **arm64 static library of AOT-compiled PHP** that an iOS application links into its Xcode project and calls through the C ABI `--emit cdylib` already produces.

The UI stays in Swift. PHP supplies domain logic — validation, pricing, business rules — shared with the server and executed offline on device.

## 2. Non-goals

- **"Write your iOS app in PHP."** elephc ships no interpreter and no source. There is no runtime `eval`, no dynamic `include`, no framework boot on device.
- **Shipping an app shell.** The deliverable is a `.a` the app developer links. Code signing, entitlements, App Store submission and the Xcode project belong to the app, not to elephc.
- **Android, in this spec.** Android is reachable and shares most of the groundwork, but its cost centre is different (see §4) and it is out of scope here.
- **Migrating raw syscalls to libSystem.** Deliberately deferred — see §8.

## 3. What the tree already provides

Facts verified against `origin/main` (`d14d1ee18e`):

| Capability | Where | Note |
|---|---|---|
| C-ABI export trampolines | `src/codegen_support/cdylib.rs` | `#[Export]` PHP functions get unmangled C symbols via tail-branch |
| Lifecycle entry points | same | `elephc_init` / `elephc_shutdown` / `elephc_last_error` / `elephc_free` — currently v1 stubs |
| Shared-library emit | `Emit::Cdylib`, `src/codegen/mod.rs:86` | `-dylib` + `@rpath` install name on Darwin |
| PIC codegen | `Emitter::new_pic`, `src/codegen/mod.rs` | cross-object data through the GOT; required for PIE |
| Both mobile arches | `Arch::{AArch64, X86_64}`, `platform/target.rs:29` | no new ISA backend needed |
| Darwin SDK resolution | `src/linker/sdk.rs` | already drives `xcrun` |
| Host import mechanism | `src/web_prelude.rs` | 112 host symbols declared as PHP signatures, link-resolved against `crates/elephc-web` |

The last row matters beyond `--web`: it is the proven pattern for a future **native bridge** (camera, biometrics, push). Inbound calls — including string returns, e.g. `web_prelude.rs:184` `function elephc_web_header_name(int $i): string;` — already work. Outbound does not (§6).

## 4. Why iOS before Android

| | iOS | Android |
|---|---|---|
| Arch | arm64 — primary backend | arm64 |
| Object format | Mach-O, **identical to macOS** | ELF |
| libc | **the same libSystem as macOS** | Bionic — each of ~292 `bl_c()` sites is a question mark |
| Toolchain | `xcrun`/`ld`, already driven | NDK clang, new plumbing |
| Test loop | **Simulator is arm64 on Apple Silicon** — no device, no dev account | emulator or device |
| Extra constraints | code signing | 16 KB page alignment, JNI glue |

The libc row is the argument. On iOS the long tail is nearly empty because it is the same C library elephc already targets. On Android that tail *is* the project.

## 5. Architecture decision — iOS is a Darwin sub-variant, not a new `Platform`

### Current shape

```rust
pub enum Platform { MacOS, Linux, Windows }   // platform/target.rs:21
pub struct Target { pub platform: Platform, pub arch: Arch }  // :36
```

### Rejected: `Platform::IOS`

Measured cost: **~181 `Platform::MacOS` match arms across 54 files**. Every one would grow an iOS arm returning a value *identical* to macOS — same syscall numbers, same `O_NONBLOCK`, same `SOL_SOCKET`, same Mach-O conventions. Pure churn, and it turns every future OS constant into a four-way match.

### Adopted: a variant field on `Target`

```rust
pub enum AppleVariant { MacOS, IOS, IOSSimulator }
pub struct Target { platform: Platform, arch: Arch, apple_variant: AppleVariant }
```

`Platform::MacOS` keeps answering every ABI and constant question correctly and for free. Only the linker, the native-dependency toolchain and a small set of capability gates read the variant.

**Measured cost — much lower than a field addition usually implies.** There are **zero literal `Target { … }` constructions** in the tree. Every `Target` is built through one of three constructors, and only **7 of their 131 call sites are production code**:

| Constructor | Total | Production | Production sites |
|---|---|---|---|
| `Target::new` | 100 | **1** | `codegen_support/abi/registers.rs:198` |
| `Target::parse` | 7 | **2** | `cli.rs:530`, `native_deps/cli.rs:98` |
| `Target::detect_host` | 24 | **4** | `cli.rs:222`, `native_deps/toolchain.rs:106`, `native_deps/recipes/{pcre2.rs:44,zlib.rs:37}` |

The remaining 124 sites are tests. Adding a field with a sensible default is therefore a contained change, not a sweep.

Further confirmation of the decision: `Target::supports_current_backend()` (`platform/target.rs:602`) matches only on `(Platform, Arch)`, and `(MacOS, AArch64)` is **already `true`**. The central gate at `pipeline.rs:409` needs no change at all. The rejected `Platform::IOS` route would have forced it open.

### Where the variant must actually be read

`Target` derives `Copy, PartialEq, Eq` but **not `Hash`**, and is never a map key. The real serialization surface is `as_str()` / `Display`, and it is wider than first assumed — three distinct persisted keys would collide between a macOS and an iOS build if the variant is not encoded:

- `runtime_cache.rs:152` `runtime_cache_file_name()` — the cached runtime object filename;
- `native_deps/receipt.rs:24-40` `ArtifactReceipt` — a `Serialize`/`Deserialize` JSON persisted to disk with a `target: String` field fed from `as_str()`;
- `native_deps/catalog.rs:124-133` `ensure_target()` — per-package static `supported_targets` lists.

Sites that must branch on the variant:

| Site | Why |
|---|---|
| `linker/sdk.rs:16-29` `macos_sdk_path()` | runs `xcrun --show-sdk-path` with no `--sdk`, resolving the default (macOS) SDK |
| `linker/sdk.rs:53-68` `macos_sdk_version()` | hardcodes `--sdk macosx`, falls back to `"15.0"` |
| `linker/command.rs:108-164` `render_macos_command` | hardcodes `-platform_version macos <v> <v>`, reusing one version for both min-OS and SDK |
| `native_deps/toolchain.rs:183-214` `validate_tuple` | requires the compiler's `-dumpmachine` triple to contain **both** `apple` and `darwin`, and synthesises `"{arch}-apple-darwin"`. An iOS cross-compiler reports `arm64-apple-ios` — **this check rejects it today** |
| `cli.rs:96`, `Target::parse` error arm | user-visible target lists |

Sites that must keep seeing plain "macOS" — no variant reading, because XNU and Mach-O are identical on arm64: every `Platform::MacOS =>` syscall number / struct offset / errno / flag constant in `impl Platform`; `php_os_name()` → `"Darwin"` (correct for iOS too); `extern_symbol()` and `darwin_arch_name()`; `assembler_cmd()`/`linker_cmd()` staying `as`/`ld`; `runtime_cache.rs:96-98`; `linker/mod.rs` `dsymutil` / `archive_dedup` / Homebrew paths.

## 6. The blocking gap — `#[Export]` cannot return a string

```
src/exports.rs:155  is_v1_param_type  → Int | Float | Bool | Str    ← Str accepted IN
src/exports.rs:163  is_v1_return_type → Int | Float | Bool | Void   ← no Str OUT
```

`cdylib.rs` states it outright: `elephc_free` is a stub *"until string-return marshaling lands"*.

Every realistic mobile payload — serialized view tree, JSON result, domain object — **is a string**. An embedded elephc library can currently return only a number. This gap, not any iOS-specific concern, is what blocks the whole embedding story.

It is also **platform-independent**: it serves macOS, Linux, Android and iOS alike, it is testable entirely on macOS today, and it forces the ownership decision (`elephc_free` becoming real) to be made once, deliberately, rather than under demo pressure.

Hence it is Lot 1, ahead of any iOS-specific work.

### 6.1 What a `Str` actually is

There is no Zend-style string object. A `Str` is a raw **pointer + length** byte string (not UTF-8) carried in a register pair, and its provenance decides who owns it:

| Provenance | Storage | Owned? |
|---|---|---|
| Literal | `.rodata` | never — must not be freed |
| Scratch | `_concat_buf` bump arena with `_concat_off` cursor (`runtime/strings/concat.rs:13-82`) | no — **invalidated by the next concat** |
| Persisted | `__rt_str_persist` copies into an `__rt_heap_alloc` block, kind tag `1` (`runtime/strings/str_persist.rs:18-88` arm64, `:95-144` x86_64) | yes |

Heap blocks carry a 16-byte header before the user pointer — `[size:4][refcount:4][kind:8]` (`runtime/arrays/heap_alloc.rs:21`). Kinds: `1` owned string, `2` indexed array, `3` hash, `4` object, `5` boxed Mixed, `6` throwable.

**Strings are move-semantics, not shared-ownership.** `__rt_decref_any` dispatches kind `1` to `__rt_heap_free_safe`, which validates liveness and then frees **unconditionally** — it does not decrement (`runtime/arrays/decref_any.rs:65-66`, `runtime/arrays/heap_free.rs:266-300`). Only boxed Mixed (kind 5) has genuine refcounting. `str_persist.rs:36-37` records why: a zero-length early-return was removed precisely because it let two owners alias one buffer into a double-free.

### 6.2 Two register mismatches that would corrupt silently

The internal string-return pair is `string_result_regs` (`abi/registers.rs:97-104`): **arm64 `(x1, x2)`**, **x86_64 `(rax, rdx)`**.

**Return path — arm64 breaks.** AAPCS64 returns a 16-byte aggregate in `(x0, x1)`. Today's trampoline is a blind tail-branch (`cdylib.rs:108-113`), so the host would read `x0` — an unrelated value — as the pointer, and the real pointer as the length. Fix: a non-tail trampoline, `bl` then `mov x0, x1` / `mov x1, x2` / `ret`. x86_64 needs nothing: `(rax, rdx)` is exactly the SysV convention for a two-INTEGER-member 16-byte struct return, so the tail `jmp` stays valid.

**Free path — x86_64 breaks.** The C first argument arrives in `rdi`, but `__rt_heap_free_safe` expects the pointer in `rax` (`runtime/arrays/heap_free.rs:311`, `:540-554`). Fix: `mov rax, rdi` before the tail `jmp`. arm64 needs nothing: `x0` is both the C argument register and the helper's input.

The pain is symmetric and neither side is covered by the other's tests — which is exactly how this would have shipped as silent corruption on the arm64 path that iOS depends on.

### 6.3 ABI decisions

**Return `(ptr, len)` by value in the existing register pair**, host releases through `elephc_free`. This is the only option requiring no new runtime mechanism — a caller-supplied buffer needs a two-phase size probe, and a NUL-terminated `const char*` cannot represent PHP byte strings containing `\0`.

Three decisions the mapping forces:

1. **Persist every `Str` returned from an export, unconditionally.** `persist_scratch_return_string` (`ir_lower/stmt/mod.rs:2522-2544`) only persists values produced by scratch-categorised ops (`ir_lower/expr/mod.rs:673-686`). So `return $param;` can hand the host back *the very pointer the host passed in*. Today `heap_free_safe` happens to reject out-of-heap pointers silently, but that is an implementation accident, not a contract. An ABI where the host sometimes owns the result and sometimes does not is unusable; one copy per export call buys a rule the host can actually follow.
2. **Translate `NULL_SENTINEL` to a real C `NULL`** at the trampoline boundary. The internal "no string" marker is `0x7fff_ffff_ffff_fffe` (`sentinels.rs:62`), not a null pointer; leaking it to a C caller turns any deref or `elephc_free` into a crash.
3. **No NUL-termination guarantee.** PHP strings are byte strings. The host must use the returned length; `strlen()` breaks silently on embedded `\0`. To be stated in the header docs, not merely implied.

## 7. Work packages

### Lot 0 — iOS relink spike
**No compiler change.** Emit assembly with `--emit-asm`, assemble with `as`, then relink the user object plus the cached runtime object by hand against the iPhoneSimulator SDK:

```
ld -arch arm64 -dylib -o libfoo.dylib foo.o <cache>/runtime-*.o \
   -lSystem -syslibroot $(xcrun --sdk iphonesimulator --show-sdk-path) \
   -platform_version ios-simulator <min> <sdk>
```

Load into a SwiftUI simulator app, call `elephc_init()` then an `int`-returning `#[Export]`.

**Accept:** the call returns the correct value in the simulator.
**Prereq:** full Xcode. Command Line Tools alone carry no iOS SDK.
**Purpose:** kill or validate the premise for half a day, before any investment.

### Lot 1 — `Str` return on `#[Export]`
Allow `PhpType::Str` in `is_v1_return_type`, implement C-ABI marshaling, settle ownership so `elephc_free` stops being a stub.

**Accept:** an exported PHP function returns a string to a C caller; the buffer is released through `elephc_free`; no leak under the existing GC-stats tooling; round-trip covered by a test naming the export.
**Blocked by:** nothing platform-related. Needs a working cargo build.

### Lot 2 — View-protocol spike, on macOS
SwiftUI app on **macOS**, linking an elephc `.dylib`, calling `#[Export] function render(): string` returning a serialized view tree that Swift renders natively.

Proves the UI answer with today's toolchain, with iOS entirely out of the picture. This is the package that de-risks the *product*; Lot 0 de-risks the *platform*.

**Accept:** a native SwiftUI view tree driven end-to-end by compiled PHP.
**Depends on:** Lot 1.

### Lot 3 — `Target` Apple variant, then `Emit::Staticlib`
The enabling refactor (§5), then the delivery form. A staticlib reuses the cdylib PIC path wholesale — iOS mandates PIE — so only the final archiving step differs, and `ar`/`ranlib` already exist in the linker for bridge staticlibs.

Target strings follow the existing dual convention (short `platform-arch` plus an LLVM-style triple): `ios-arm64` / `aarch64-apple-ios`, and `ios-sim-arm64` / `aarch64-apple-ios-simulator`. `test_target_parse` (`platform/mod.rs:27-40`) and the shared integration fixture `tests/codegen/support/platform.rs:14-21` extend with them.

Includes the native-dependency path: `native_deps/toolchain.rs` `validate_tuple` currently **rejects** an iOS cross-compiler outright, and `native_deps/catalog.rs` `supported_targets` must list the iOS strings before pcre2/zlib can be built for the target.

**Accept:** `--target ios-arm64 --emit staticlib` produces a `.a` that links cleanly in an Xcode project.

### Lot 4 — Capability gating
`exec`, `shell_exec`, `system`, `passthru`, `popen`, `pclose`, `proc_open` are unusable in the iOS sandbox — no `fork`. They must fail **at compile time** with a targeted diagnostic, never silently at runtime.

Model to follow: the WASM backend's `capability.rs` (on `feat/wasm-target`) uses **exhaustive matches with no `_` arm**, so a newly added enum variant cannot compile until its support status is decided, and `validate_module` aggregates every violation into one error naming collection / function / block / instruction before planning begins.

What `main` has today is the weaker shape and should not be copied: `Platform::Windows => panic!(…)` repeated ~20 times across `impl Platform`, a runtime trap that only fires when a path actually executes.

**Accept:** compiling a script that calls any of them for an iOS target yields a clear compile error naming the builtin and the reason.

## 8. Deliberately deferred — raw syscalls to libSystem

The runtime issues **225 `.syscall(N)`** calls, all funnelled through one choke point, `Emitter::syscall()` in `src/codegen_support/emit.rs`.

They *work* on iOS and in the simulator. Apple's supported ABI is libSystem, so this is a **long-term supportability risk, not a functional blocker** — Apple has broken the direct syscall ABI before, which is why Go migrated to libSystem on Darwin.

The trap is not the site count, it is the register contract. `svc #0x80` clobbers x0 and x16 only; `bl _write` clobbers x0–x17 and LR per AAPCS. All 225 sites were written under the syscall convention, so any value live in x1–x15 across the call would be silently destroyed. A naive substitution inside `Emitter::syscall()` produces diffuse corruption, not clean crashes.

Clean fix: one save/restore wrapper at the choke point, validated by diffing syscall-mode against libSystem-mode output across the codegen suite — measurable on macOS with no device.

Worth doing on its own merits as Darwin debt. **Not a prerequisite** for proving the rest, and not to be paid before Lots 0–2 have said the rest holds.

## 9. Risks and open questions

1. **Three persisted keys** must incorporate the Apple variant (§5) — the runtime-object cache filename, the `ArtifactReceipt` JSON, and the native-dependency catalog — or macOS and iOS builds collide silently on shared state.
1b. **Native dependencies are a second front.** `native_deps/` builds pcre2 and zlib from source per target; its toolchain validator rejects a non-`darwin` Apple triple today. Any PHP program reaching PCRE or zlib on iOS depends on this path, so it cannot be deferred past Lot 3.
2. **String ownership model** (Lot 1) is an ABI commitment. Options — return `ptr+len` and require `elephc_free`, versus copy into a caller-supplied buffer — must be weighed against the actual internal string representation and refcounting before choosing.
3. **Static vs dynamic delivery.** A `.a` avoids the embedded-framework signing dance entirely. A `.dylib` in an app bundle must live under `Frameworks/` and be signed separately. Lot 3 chooses static for that reason; revisit only if a consumer needs dynamic loading.
4. **Simulator vs device divergence.** The simulator is arm64 on Apple Silicon but is *not* the device: it uses a different platform load command and a host kernel. Lot 0 passing on the simulator does not by itself prove the device path.
5. **Scope honesty.** None of this creates new PHP capability. It relocates an artefact that already exists. It is a different bet from builtin/framework coverage, and should be judged as one.

## 10. Environment prerequisites

- **Full Xcode.** The dev machine currently reports `/Library/Developer/CommandLineTools`; `xcrun --sdk iphonesimulator` cannot locate an SDK. Lot 0 is blocked until this is installed.
- **Disk headroom.** Lots 1–4 need cargo builds; the dev machine has been fluctuating between 3 and 5 GB free, with roughly 31 GB held by accumulated git worktrees.
- **Build serialisation.** Concurrent cargo invocations relink the binary underneath a running suite and produce mass false failures. One cargo command at a time.
