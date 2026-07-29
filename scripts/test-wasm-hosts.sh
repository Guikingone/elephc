#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly REPO_ROOT
readonly ELEPHC="${1:-$REPO_ROOT/target/debug/elephc}"
readonly FIXTURE="$REPO_ROOT/tests/fixtures/wasm/host_portability.php"
readonly PARTIAL_WRITE_HARNESS="$REPO_ROOT/tests/fixtures/wasm/partial_fd_write.mjs"
readonly TYPESCRIPT_CONSUMER="$REPO_ROOT/tests/fixtures/wasm/npm_consumer.ts"
readonly MAX_OUTPUT_BYTES=1048576

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/elephc-wasm-hosts.XXXXXX")"
readonly WORK_DIR
export NPM_CONFIG_CACHE="$WORK_DIR/npm-cache"
trap 'rm -rf "$WORK_DIR"' EXIT

fail() {
  printf 'wasm host portability: %s\n' "$*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command is missing: $1"
}

assert_bounded_output() {
  local path="$1"
  local label="$2"
  local size
  size="$(wc -c <"$path")"
  if ((size > MAX_OUTPUT_BYTES)); then
    fail "$label exceeded $MAX_OUTPUT_BYTES bytes (got $size)"
  fi
}

assert_file_equals() {
  local actual="$1"
  local expected="$2"
  local label="$3"
  if ! cmp -s "$expected" "$actual"; then
    printf '%s mismatch\nexpected bytes:\n' "$label" >&2
    od -An -tx1 -v "$expected" >&2
    printf 'actual bytes:\n' >&2
    od -An -tx1 -v "$actual" >&2
    exit 1
  fi
}

run_and_assert() {
  local label="$1"
  local expected_status="$2"
  local expected_stdout="$3"
  local expected_stderr="$4"
  shift 4

  local stdout_path="$WORK_DIR/$label.stdout"
  local stderr_path="$WORK_DIR/$label.stderr"
  local status

  set +e
  timeout --signal=KILL 10s "$@" >"$stdout_path" 2>"$stderr_path"
  status=$?
  set -e

  if [[ "$status" -eq 124 || "$status" -eq 137 ]]; then
    fail "$label exceeded the 10 second process timeout"
  fi
  if [[ "$status" -ne "$expected_status" ]]; then
    printf '%s exited with %s, expected %s\nstderr:\n' \
      "$label" "$status" "$expected_status" >&2
    sed -n '1,120p' "$stderr_path" >&2
    exit 1
  fi

  assert_bounded_output "$stdout_path" "$label stdout"
  assert_bounded_output "$stderr_path" "$label stderr"
  assert_file_equals "$stdout_path" "$expected_stdout" "$label stdout"
  assert_file_equals "$stderr_path" "$expected_stderr" "$label stderr"
}

for command in cmp diff node npm od timeout tsc wasmer wasmtime wasm-tools; do
  require_command "$command"
done
[[ -x "$ELEPHC" ]] || fail "compiler binary is not executable: $ELEPHC"

mkdir -p "$WORK_DIR/first" "$WORK_DIR/second"
cp "$FIXTURE" "$WORK_DIR/first/host_portability.php"
cp "$FIXTURE" "$WORK_DIR/second/host_portability.php"

"$ELEPHC" --target wasm32-wasi --emit npm \
  "$WORK_DIR/first/host_portability.php" \
  >"$WORK_DIR/first.compile.stdout" 2>"$WORK_DIR/first.compile.stderr"
"$ELEPHC" --target wasm32-wasi --emit npm \
  "$WORK_DIR/second/host_portability.php" \
  >"$WORK_DIR/second.compile.stdout" 2>"$WORK_DIR/second.compile.stderr"
"$ELEPHC" --target wasm32-wasi --emit-asm \
  "$WORK_DIR/first/host_portability.php" \
  >"$WORK_DIR/first.wat.stdout" 2>"$WORK_DIR/first.wat.stderr"
"$ELEPHC" --target wasm32-wasi --emit-asm \
  "$WORK_DIR/second/host_portability.php" \
  >"$WORK_DIR/second.wat.stdout" 2>"$WORK_DIR/second.wat.stderr"

readonly FIRST_PACKAGE="$WORK_DIR/first/host_portability-npm"
readonly SECOND_PACKAGE="$WORK_DIR/second/host_portability-npm"
readonly WASM_MODULE="$FIRST_PACKAGE/module.wasm"
readonly SECOND_WASM_MODULE="$SECOND_PACKAGE/module.wasm"
readonly FIRST_WAT="$WORK_DIR/first/host_portability.wat"
readonly SECOND_WAT="$WORK_DIR/second/host_portability.wat"

[[ -f "$WASM_MODULE" ]] || fail "compiler did not publish $WASM_MODULE"
[[ -f "$SECOND_WASM_MODULE" ]] || fail "compiler did not publish $SECOND_WASM_MODULE"
[[ -f "$FIRST_WAT" ]] || fail "compiler did not publish $FIRST_WAT"
[[ -f "$SECOND_WAT" ]] || fail "compiler did not publish $SECOND_WAT"
cmp "$WASM_MODULE" "$SECOND_WASM_MODULE" ||
  fail "independent compiler processes emitted different WASM binaries"
cmp "$FIRST_WAT" "$SECOND_WAT" ||
  fail "independent compiler processes emitted different WAT files"
if ! diff -ru --no-dereference "$FIRST_PACKAGE" "$SECOND_PACKAGE" \
  >"$WORK_DIR/package.diff"; then
  sed -n '1,200p' "$WORK_DIR/package.diff" >&2
  fail "independent compiler processes emitted different NPM packages"
fi

wasm-tools validate "$WASM_MODULE"
wasmer validate "$WASM_MODULE"
wasmtime compile "$WASM_MODULE" -o "$WORK_DIR/module.cwasm"
node --input-type=module --eval '
  import { readFile } from "node:fs/promises";
  new WebAssembly.Module(await readFile(process.argv[1]));
' "$WASM_MODULE"

printf '2|first\n' >"$WORK_DIR/expected.stdout"
: >"$WORK_DIR/expected.stderr"

run_and_assert wasmer 7 "$WORK_DIR/expected.stdout" "$WORK_DIR/expected.stderr" \
  wasmer run --quiet "$WASM_MODULE" -- first
run_and_assert wasmtime 7 "$WORK_DIR/expected.stdout" "$WORK_DIR/expected.stderr" \
  wasmtime run "$WASM_MODULE" first
run_and_assert node-cli 7 "$WORK_DIR/expected.stdout" "$WORK_DIR/expected.stderr" \
  env NODE_NO_WARNINGS=1 node "$FIRST_PACKAGE/index.mjs" first

printf '2|first\n2|first\n' >"$WORK_DIR/expected-import.stdout"
run_and_assert node-import 0 \
  "$WORK_DIR/expected-import.stdout" "$WORK_DIR/expected.stderr" \
  env NODE_NO_WARNINGS=1 node --input-type=module --eval '
    import { pathToFileURL } from "node:url";
    const { run } = await import(pathToFileURL(process.argv[2]));
    for (let index = 0; index < 2; index += 1) {
      const status = await run({ args: ["host-portability", "first"] });
      if (status !== 7) {
        throw new Error("run() returned " + status + ", expected 7");
      }
    }
  ' "$FIRST_PACKAGE/package.json" "$FIRST_PACKAGE/index.mjs"

run_and_assert partial-fd-write 0 \
  "$WORK_DIR/expected.stderr" "$WORK_DIR/expected.stderr" \
  node "$PARTIAL_WRITE_HARNESS" "$WASM_MODULE"

(
  cd "$FIRST_PACKAGE"
  npm pack --dry-run --ignore-scripts --json >"$WORK_DIR/npm-dry-run.json"
)
node --input-type=module --eval '
  import assert from "node:assert/strict";
  import { readFile } from "node:fs/promises";
  const report = JSON.parse(await readFile(process.argv[1], "utf8"));
  assert.equal(report.length, 1);
  const files = report[0].files.map(({ path }) => path).sort();
  assert.deepEqual(files, [
    "README.md",
    "index.d.ts",
    "index.mjs",
    "module.wasm",
    "package.json",
  ]);
' "$WORK_DIR/npm-dry-run.json"

mkdir -p "$WORK_DIR/pack-first" "$WORK_DIR/pack-second"
(
  cd "$FIRST_PACKAGE"
  npm pack --ignore-scripts --json --pack-destination "$WORK_DIR/pack-first" \
    >"$WORK_DIR/npm-pack-first.json"
)
(
  cd "$SECOND_PACKAGE"
  npm pack --ignore-scripts --json --pack-destination "$WORK_DIR/pack-second" \
    >"$WORK_DIR/npm-pack-second.json"
)
FIRST_ARCHIVE="$(
  node --eval 'console.log(JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"))[0].filename)' \
    "$WORK_DIR/npm-pack-first.json"
)"
readonly FIRST_ARCHIVE
SECOND_ARCHIVE="$(
  node --eval 'console.log(JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"))[0].filename)' \
    "$WORK_DIR/npm-pack-second.json"
)"
readonly SECOND_ARCHIVE
cmp "$WORK_DIR/pack-first/$FIRST_ARCHIVE" "$WORK_DIR/pack-second/$SECOND_ARCHIVE"

mkdir -p "$WORK_DIR/typescript-consumer"
cp "$TYPESCRIPT_CONSUMER" "$WORK_DIR/typescript-consumer/npm_consumer.ts"
(
  cd "$WORK_DIR/typescript-consumer"
  npm install --ignore-scripts --no-audit --no-fund --no-save "$FIRST_PACKAGE"
  tsc --noEmit --strict --target ES2022 --module NodeNext \
    --moduleResolution NodeNext npm_consumer.ts
)

printf 'WASM host portability checks passed.\n'
