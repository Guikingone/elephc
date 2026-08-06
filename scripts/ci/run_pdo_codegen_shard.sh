#!/usr/bin/env bash
# Run one deterministic PDO shard inside a single libtest process.

set -euo pipefail

# Several PDO fixtures lower the full generated prelude recursively. Rust's default 2 MiB
# libtest worker stack is too small when many tests share one process, especially on AArch64.
export RUST_MIN_STACK=${RUST_MIN_STACK:-33554432}

if [[ $# -ne 3 ]]; then
    echo "usage: run_pdo_codegen_shard.sh <test-binary> <shard> <shard-count>" >&2
    exit 2
fi

test_binary=$1
shard=$2
shard_count=$3

if [[ ! -f $test_binary || ! $shard =~ ^[0-9]+$ || ! $shard_count =~ ^[0-9]+$ \
    || $shard -lt 1 || $shard -gt $shard_count ]]; then
    echo "invalid test binary or shard selection" >&2
    exit 2
fi

selected=()
skipped=()
while IFS= read -r line; do
    [[ $line == codegen::pdo*:" test" ]] || continue
    test_name=${line%: test}
    checksum_line=$(printf '%s' "$test_name" | cksum)
    checksum=${checksum_line%% *}
    test_shard=$((checksum % shard_count + 1))
    if [[ $test_shard -eq $shard ]]; then
        selected+=("$test_name")
    else
        skipped+=(--skip "$test_name")
    fi
done < <("$test_binary" --list)

if [[ ${#selected[@]} -eq 0 ]]; then
    echo "PDO shard $shard/$shard_count selected no tests" >&2
    exit 2
fi

echo "Running ${#selected[@]} PDO tests in shard $shard/$shard_count"
exec "$test_binary" codegen::pdo "${skipped[@]}" --test-threads 4
