# PHP-driven SwiftUI — the view-protocol spike

A native macOS app whose entire interface is decided by compiled PHP. Swift draws;
it does not decide.

```
./run.sh                  # build and launch
./run.sh --build-only     # build the .app without launching
./ViewProtocol.app/Contents/MacOS/ViewProtocol --selftest    # headless check
```

Needs only the Xcode Command Line Tools — `swiftc` ships with them and SwiftUI is
a system framework, so there is no Xcode install and no `.xcodeproj` here.

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
  returns a `(ptr, len)` pair the host owns and releases through `elephc_free`;
  `dispatch(string $action)` passes one in.
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
| `ViewProtocolApp.swift` | library loading, JSON → SwiftUI, event dispatch |
| `elephc_abi.h` | the C declaration of the `(ptr, len)` return type |
| `run.sh` | compile both sides, assemble and sign a `.app` bundle |

## Two things that will bite you

**`ElephcStr` has to be a C type.** Swift rejects a Swift-declared struct in a
`@convention(c)` signature — only a C type carries the guarantee that the value
rides the platform's aggregate-return registers. Hence `elephc_abi.h` and
`-import-objc-header`.

**Returned strings are not NUL-terminated.** They are PHP byte strings and may
contain interior zero bytes, so the returned length is authoritative and
`strlen` is wrong.

## Scope

macOS on purpose. This proves the UI story with the toolchain that already
works, leaving the iOS SDK as a separate question — see `scripts/ios-relink-spike.sh`
and `IOS_TARGET_SPEC.md`. Nothing in the design is macOS-specific: the same
library and the same protocol drive a UIKit or SwiftUI host on iOS once that
target links.
