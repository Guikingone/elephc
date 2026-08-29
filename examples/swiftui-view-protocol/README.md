# PHP-driven SwiftUI — the view-protocol spike

A native app whose entire interface is decided by compiled PHP. Swift draws; it
does not decide. Runs on macOS **and** on iOS, from the same `view.php` and the
same Swift.

```
./run.sh                  # macOS: build and launch
./run.sh --build-only     # macOS: build the .app without launching
./ViewProtocol.app/Contents/MacOS/ViewProtocol --selftest

./run-ios.sh              # iOS: build, install on a booted simulator, launch, screenshot
./run-ios.sh --selftest   # iOS: headless round-trip check inside the simulator
```

macOS needs only the Xcode Command Line Tools — `swiftc` ships with them and
SwiftUI is a system framework. iOS needs full Xcode, an installed runtime and a
booted simulator:

```
xcodebuild -downloadPlatform iOS
xcrun simctl boot "iPhone 17 Pro"
```

Neither path involves an `.xcodeproj`. The library is linked **statically** — the
delivery form an Xcode project would consume — so the exports are ordinary C
symbols rather than `dlsym` lookups, which is also what lets the same Swift run
unchanged on iOS.

## The idea

PHP has no UI toolkit, so "write your app in PHP" is not on the table. What *is*
on the table is the shape React Native and server-driven UI use: **PHP describes
a view tree, a native host renders it.**

```
render_view()  ─────────► {"t":"vstack","children":[ … ]}  ─────────► SwiftUI views
                                                                          │
dispatch("inc") ◄────────────────── button tapped ◄───────────────────────┘
```

`view.php` owns the layout, the labels, the pluralisation and the state.
`ViewProtocolApp.swift` knows four node types and nothing else. Swap `view.php`
and the app changes without recompiling a line of Swift.

This matters for the ahead-of-time story specifically. A template engine has to
*evaluate itself* on the device, which needs a PHP runtime there. A tree
*generator* compiles once and ships as machine code — so this is the one corner
of the UI problem where being AOT costs nothing at all.

## What the spike actually demonstrates

- **A string crosses the boundary in both directions.** `render_view(): string`
  returns ABI-v3 status plus caller-owned `output_ptr`/`output_len`; the host
  copies the bytes and releases the buffer through `elephc_free`.
  `dispatch(string $action)` passes one input pointer/length pair and receives
  the same status/out-parameter result shape.
- **State lives in PHP.** `counter()` uses a function `static`, which persists in
  the loaded library's own memory across host calls. Swift holds no counter.
- **The host stays dumb.** Every string the user sees — including `"2 items"`
  versus `"one item"` — is computed by compiled PHP.

`--selftest` asserts exactly that, headlessly, so the example is verifiable
without a display:

```
initial=nothing yet after++=2 items after-=one item reset=nothing yet
PASS: the view tree, the string ABI and PHP-side state all round-trip
```

## Files

| | |
|---|---|
| `view.php` | the whole application: tree builders, state, action handling |
| `ViewProtocolApp.swift` | JSON → SwiftUI, event dispatch, the self-test |
| `elephc_abi.h` | Swift bridging wrapper around the generated ABI-v3 `libview.h` |
| `run.sh` | macOS: compile both sides, assemble and sign a `.app` |
| `run-ios.sh` | iOS: same, then install, launch and screenshot on a simulator |

## ABI and toolchain details

**Use the generated header.** String exports return `int32_t` status and append
`char **output_ptr, size_t *output_len`; they do not return a C aggregate.
`elephc_abi.h` includes the freshly generated `libview.h` instead of copying its
declarations. Its only adapter renames the source export `dispatch` for Swift;
the inline C call is type-checked against the generated prototype. Successful
buffers are caller-owned; failed calls leave the outputs `NULL`/zero.

**Returned lengths are authoritative.** PHP strings may contain interior zero
bytes. ABI v3 may provide an optional trailing NUL for convenience, but that
byte is outside `output_len`; `strlen` is still wrong.

**Call `elephc_init`.** Besides runtime state, it arms the stack-overflow floor
for host calls. The export boundary has a lazy fallback, but explicit startup
also verifies clean lifecycle handling in the example.

**`-sdk` does not reach the link step.** `swiftc` drives `clang` to link, and
that driver defaults to the *host* sysroot — so an iOS build warns *"using
sysroot for 'MacOSX' but targeting 'iPhone'"* unless `-Xclang-linker -isysroot`
passes the SDK through explicitly.

## Scope

Both platforms, one source. The `--selftest` output is identical on macOS and
inside the iOS Simulator, which is the point: nothing in the design is
platform-specific.

macOS on purpose for the original spike: it proved the UI story with the
toolchain that already worked, leaving the iOS SDK as a separate question. That
question is now answered — see `scripts/ios-relink-spike.sh` and
`IOS_TARGET_SPEC.md` — and `run-ios.sh` runs the very same app on a device-class
arm64 simulator.
