---
title: "Installation"
description: "How to install elephc on supported platforms."
sidebar:
  order: 1
---

## Requirements

- Rust toolchain (`cargo`) if building from source
- A native assembler and linker for your host platform

Projects that install a curated native package also need a POSIX shell, Make, a
target C compiler, `ar`, and `ranlib`. Elephc verifies and uses these tools but
does not install them. The catalogued PCRE2 and zlib sources do **not** need to
be installed from system packages; for example, `elephc native add pcre2`
downloads, verifies, and builds the exact source into Elephc's cache without a
system fallback.

On macOS, install Xcode Command Line Tools if you don't have them already:

```bash
xcode-select --install
```

This provides the assembler (`as`), linker (`ld`), C compiler, archive tools,
and Make used to produce binaries and managed native artifacts.

On Linux, install your distro's standard native toolchain so `as`, `ld`, `cc`,
`ar`, `ranlib`, Make, and the libc development files are available. For example,
Debian/Ubuntu's `build-essential` or Alpine's `build-base` plus `make` provide
the usual host tools. Cross-target artifacts require an explicit matching C
compiler, archiver, and ranlib; see [Native
dependencies](../compiling/native-dependencies.md#build-tools-and-cross-targets).

## Homebrew (macOS)

```bash
brew install illegalstudio/tap/elephc
```

Verify the installation by compiling a small program:

```bash
echo '<?php echo "ok\n";' > check.php
elephc check.php && ./check
```

This prints `ok` and confirms `elephc` can produce and run a native binary.

## From source

If you prefer to build from source, you'll also need the Rust toolchain (`cargo`).

```bash
git clone https://github.com/illegalstudio/elephc.git
cd elephc
cargo build --release
```

The binary is at `./target/release/elephc`. You can copy it to a directory in your `PATH`:

```bash
cp target/release/elephc /usr/local/bin/
```

## From GitHub releases

Each release on the [releases page](https://github.com/illegalstudio/elephc/releases)
ships a tarball per supported platform:

- `elephc-<version>-aarch64-apple-darwin.tar.gz` — macOS ARM64
- `elephc-<version>-x86_64-unknown-linux-gnu.tar.gz` — Linux x86_64
- `elephc-<version>-aarch64-unknown-linux-gnu.tar.gz` — Linux ARM64

Each tarball contains the `elephc` binary plus the bridge staticlibs
(`libelephc_*.a`) that compiled programs link against. Install them in a layout
elephc searches: the staticlibs either in the same directory as the binary, or
in a `lib/` directory next to the `bin/` directory holding the binary — for
example `/usr/local/bin` and `/usr/local/lib`:

```bash
tar xzf elephc-<version>-<target>.tar.gz
sudo install -m 755 elephc /usr/local/bin/
sudo install -m 644 libelephc_*.a /usr/local/lib/
```

The Linux tarballs are built against glibc 2.35 and run on any distribution
with that glibc or newer (Ubuntu 22.04+, Debian 12+, RHEL 9+, ...).
