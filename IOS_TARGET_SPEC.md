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

Measured cost: **67 `Platform::MacOS =>` match arms across 54 files**. Every one would grow an iOS arm returning a value *identical* to macOS — same syscall numbers, same `O_NONBLOCK`, same `SOL_SOCKET`, same Mach-O conventions. Pure churn, and it turns every future OS constant into a four-way match.

### Adopted: a variant field on `Target`

```rust
pub enum AppleVariant { MacOS, IOS, IOSSimulator }
pub struct Target { platform: Platform, arch: Arch, apple_variant: AppleVariant }
```

`Platform::MacOS` keeps answering every ABI and constant question correctly and for free. Only the linker and a small set of capability gates read the variant.

Measured cost: **71 literal `Target { … }` constructions** to migrate. Mechanical; `Target::new` is the single constructor, so most sites can route through it.

**Open point** — `Target` is `Copy + PartialEq` and the runtime object cache encodes the target in its filename. A third field must be reflected in that cache key, or a macOS build and an iOS build will collide on the same cached runtime object. To be confirmed against `src/runtime_cache.rs` before Lot 3.

## 6. The blocking gap — `#[Export]` cannot return a string

```
src/exports.rs:155  is_v1_param_type  → Int | Float | Bool | Str    ← Str accepted IN
src/exports.rs:163  is_v1_return_type → Int | Float | Bool | Void   ← no Str OUT
```

`cdylib.rs` states it outright: `elephc_free` is a stub *"until string-return marshaling lands"*.

Every realistic mobile payload — serialized view tree, JSON result, domain object — **is a string**. An embedded elephc library can currently return only a number. This gap, not any iOS-specific concern, is what blocks the whole embedding story.

It is also **platform-independent**: it serves macOS, Linux, Android and iOS alike, it is testable entirely on macOS today, and it forces the ownership decision (`elephc_free` becoming real) to be made once, deliberately, rather than under demo pressure.

Hence it is Lot 1, ahead of any iOS-specific work.

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

**Accept:** `--target ios-arm64 --emit staticlib` produces a `.a` that links cleanly in an Xcode project.

### Lot 4 — Capability gating
`exec`, `shell_exec`, `system`, `passthru`, `popen`, `pclose`, `proc_open` are unusable in the iOS sandbox — no `fork`. They must fail **at compile time** with a targeted diagnostic, never silently at runtime.

**Accept:** compiling a script that calls any of them for an iOS target yields a clear compile error naming the builtin and the reason.

## 8. Deliberately deferred — raw syscalls to libSystem

The runtime issues **225 `.syscall(N)`** calls, all funnelled through one choke point, `Emitter::syscall()` in `src/codegen_support/emit.rs`.

They *work* on iOS and in the simulator. Apple's supported ABI is libSystem, so this is a **long-term supportability risk, not a functional blocker** — Apple has broken the direct syscall ABI before, which is why Go migrated to libSystem on Darwin.

The trap is not the site count, it is the register contract. `svc #0x80` clobbers x0 and x16 only; `bl _write` clobbers x0–x17 and LR per AAPCS. All 225 sites were written under the syscall convention, so any value live in x1–x15 across the call would be silently destroyed. A naive substitution inside `Emitter::syscall()` produces diffuse corruption, not clean crashes.

Clean fix: one save/restore wrapper at the choke point, validated by diffing syscall-mode against libSystem-mode output across the codegen suite — measurable on macOS with no device.

Worth doing on its own merits as Darwin debt. **Not a prerequisite** for proving the rest, and not to be paid before Lots 0–2 have said the rest holds.

## 9. Risks and open questions

1. **Runtime-object cache key** must incorporate the Apple variant (§5) or macOS and iOS builds collide.
2. **String ownership model** (Lot 1) is an ABI commitment. Options — return `ptr+len` and require `elephc_free`, versus copy into a caller-supplied buffer — must be weighed against the actual internal string representation and refcounting before choosing.
3. **Static vs dynamic delivery.** A `.a` avoids the embedded-framework signing dance entirely. A `.dylib` in an app bundle must live under `Frameworks/` and be signed separately. Lot 3 chooses static for that reason; revisit only if a consumer needs dynamic loading.
4. **Simulator vs device divergence.** The simulator is arm64 on Apple Silicon but is *not* the device: it uses a different platform load command and a host kernel. Lot 0 passing on the simulator does not by itself prove the device path.
5. **Scope honesty.** None of this creates new PHP capability. It relocates an artefact that already exists. It is a different bet from builtin/framework coverage, and should be judged as one.

## 10. Environment prerequisites

- **Full Xcode.** The dev machine currently reports `/Library/Developer/CommandLineTools`; `xcrun --sdk iphonesimulator` cannot locate an SDK. Lot 0 is blocked until this is installed.
- **Disk headroom.** Lots 1–4 need cargo builds; the dev machine has been fluctuating between 3 and 5 GB free, with roughly 31 GB held by accumulated git worktrees.
- **Build serialisation.** Concurrent cargo invocations relink the binary underneath a running suite and produce mass false failures. One cargo command at a time.
