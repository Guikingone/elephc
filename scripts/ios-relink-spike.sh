#!/usr/bin/env bash
# Lot 0 of IOS_TARGET_SPEC.md: prove that elephc's existing arm64 Mach-O output
# links and runs against an iOS SDK, without changing one line of the compiler.
#
# Usage:
#   ./scripts/ios-relink-spike.sh                 # iOS Simulator (default)
#   ./scripts/ios-relink-spike.sh device          # iOS device (link + inspect only)
#   ./scripts/ios-relink-spike.sh --keep          # keep the work directory
#
# What it does: compiles a PHP file with two `#[Export]`s to assembly for the
# native macos-aarch64 target, assembles it, then relinks that object plus the
# cached runtime object against the iOS SDK instead of the macOS one. The only
# difference from an ordinary build is `-syslibroot` and `-platform_version`.
#
# On the simulator it goes further and actually runs the result: a C host is
# built for the simulator triple and spawned inside a booted device through
# `simctl spawn`, so the spike ends in a real call into compiled PHP rather than
# in a well-formed file. On device the run stops after inspection — executing
# there needs provisioning and a signed app bundle, which is out of scope.
#
# Requires full Xcode. The Command Line Tools alone carry no iOS SDK; if that is
# all that is installed the script says so and exits without doing damage.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

MODE="simulator"
KEEP=0
for arg in "$@"; do
  case "$arg" in
    simulator|sim) MODE="simulator" ;;
    device)        MODE="device" ;;
    --keep)        KEEP=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [ "$MODE" = "simulator" ]; then
  SDK="iphonesimulator"
  PLATFORM="ios-simulator"
  HOST_TRIPLE="arm64-apple-ios17.0-simulator"
else
  SDK="iphoneos"
  PLATFORM="ios"
  HOST_TRIPLE="arm64-apple-ios17.0"
fi

# -- preconditions -----------------------------------------------------------
# xcrun resolves against the *selected* developer directory, so Command Line
# Tools alone fail here rather than silently falling back to the macOS SDK.
if ! SDK_PATH="$(xcrun --sdk "$SDK" --show-sdk-path 2>/dev/null)" || [ -z "$SDK_PATH" ]; then
  cat >&2 <<EOF
No '$SDK' SDK found.

xcode-select currently points at:
  $(xcode-select -p 2>/dev/null || echo '<unset>')

The Command Line Tools do not ship iOS SDKs. Install full Xcode, point at it,
and fetch the platform:

  xcodes install --latest            # or install Xcode from the App Store
  sudo xcode-select -s /Applications/Xcode.app
  xcodebuild -downloadPlatform iOS
EOF
  exit 1
fi
SDK_VERSION="$(xcrun --sdk "$SDK" --show-sdk-version)"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/elephc_ios_spike.XXXXXX")"
cleanup() { [ "$KEEP" = "1" ] || rm -rf "$WORK"; }
trap cleanup EXIT
[ "$KEEP" = "1" ] && echo "work directory: $WORK"

ELEPHC="${ELEPHC_BIN:-$PROJECT_DIR/target/debug/elephc}"
if [ ! -x "$ELEPHC" ]; then
  echo "building elephc ..." >&2
  (cd "$PROJECT_DIR" && cargo build)
fi

# -- 1. a PHP library exercising both export return shapes --------------------
cat > "$WORK/spike.php" <<'PHP'
<?php
#[Export]
function spike_add(int $a, int $b): int {
    return $a + $b;
}

#[Export]
function spike_greet(string $name): string {
    return "hi " . $name;
}
PHP

# The runtime object is cached under a key that encodes the program's runtime
# feature set, so an isolated cache guarantees exactly one candidate below
# instead of a fragile "newest match" glob over the shared cache.
export XDG_CACHE_HOME="$WORK/cache"

echo "==> compiling for macos-aarch64 (populates the runtime cache)"
(cd "$WORK" && "$ELEPHC" --emit cdylib spike.php >/dev/null)

echo "==> emitting user assembly"
(cd "$WORK" && "$ELEPHC" --emit cdylib --emit-asm spike.php >/dev/null)

RUNTIME_OBJ="$(find "$XDG_CACHE_HOME" -name 'runtime-*macos-aarch64*.o' -print -quit)"
if [ -z "$RUNTIME_OBJ" ]; then
  echo "no cached runtime object was produced" >&2
  exit 1
fi

echo "==> assembling"
as -arch arm64 -o "$WORK/spike.o" "$WORK/spike.s"

# -- 2. the actual experiment: same objects, iOS SDK --------------------------
echo "==> relinking against the $SDK SDK ($SDK_VERSION)"
LIB="$WORK/libspike.dylib"
ld -arch arm64 -dylib -o "$LIB" \
   "$WORK/spike.o" "$RUNTIME_OBJ" \
   -lSystem -syslibroot "$SDK_PATH" \
   -platform_version "$PLATFORM" 17.0 "$SDK_VERSION" \
   -install_name @rpath/libspike.dylib

echo "==> Mach-O platform"
vtool -show-build "$LIB" | sed 's/^/    /'

if [ "$MODE" = "device" ]; then
  cat <<EOF

Linked for a device. Running it needs provisioning and a signed app bundle, so
the spike stops here: the link succeeding is the answer it was asked for.
EOF
  exit 0
fi

# -- 3. run it inside a booted simulator -------------------------------------
cat > "$WORK/host.c" <<'C'
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stddef.h>

typedef struct { const char *ptr; size_t len; } elephc_str;

int main(int argc, char **argv) {
    if (argc != 2) return 1;
    void *lib = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
    if (!lib) { fprintf(stderr, "dlopen: %s\n", dlerror()); return 2; }
    int32_t (*init)(void) = (int32_t (*)(void))dlsym(lib, "elephc_init");
    int64_t (*add)(int64_t, int64_t) = (int64_t (*)(int64_t, int64_t))dlsym(lib, "spike_add");
    elephc_str (*greet)(const char *, size_t) =
        (elephc_str (*)(const char *, size_t))dlsym(lib, "spike_greet");
    void (*efree)(void *) = (void (*)(void *))dlsym(lib, "elephc_free");
    if (!init || !add || !greet || !efree) { fprintf(stderr, "dlsym failed\n"); return 3; }
    if (init() != 0) return 4;
    elephc_str g = greet("iOS", 3);
    printf("%lld %.*s %zu\n", (long long)add(40, 2), (int)g.len, g.ptr, g.len);
    efree((void *)g.ptr);
    return 0;
}
C

echo "==> building the C host for $HOST_TRIPLE"
xcrun --sdk "$SDK" clang -target "$HOST_TRIPLE" -isysroot "$SDK_PATH" \
      -o "$WORK/host" "$WORK/host.c"

DEVICE="$(xcrun simctl list devices booted -j 2>/dev/null \
          | grep -o '"udid" : "[^"]*"' | head -1 | cut -d'"' -f4 || true)"
if [ -z "$DEVICE" ]; then
  cat <<EOF

Linked and built, but no simulator is booted, so nothing was executed.
Boot one and re-run to get the end-to-end answer:

  xcrun simctl boot "iPhone 15"     # or any device from: xcrun simctl list devices
EOF
  exit 0
fi

echo "==> running inside booted simulator $DEVICE"
OUTPUT="$(xcrun simctl spawn "$DEVICE" "$WORK/host" "$LIB")"
echo "    $OUTPUT"

EXPECTED="42 hi iOS 6"
if [ "$OUTPUT" != "$EXPECTED" ]; then
  echo "unexpected output: got '$OUTPUT', want '$EXPECTED'" >&2
  exit 1
fi

cat <<EOF

Lot 0 answered: compiled PHP links against the iOS SDK and runs on the
simulator, through the same C ABI the cdylib path already exposes -- with no
compiler change. Both export return shapes work, and the string result was
released through elephc_free.
EOF
