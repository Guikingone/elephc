#!/bin/sh
# Verify that the PDO_DBLIB bridge uses FreeTDS's portable open symbol.
#
# The optional dbopen compatibility export is absent from FreeTDS builds that
# do not enable Sybase ABI compatibility. Inspecting the built bridge catches a
# regression without making CI depend on one distribution's FreeTDS build flags.

set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <libelephc_pdo.a>" >&2
    exit 2
fi

archive=$1
nm_bin=${NM:-nm}

if [ ! -f "$archive" ]; then
    echo "error: archive not found: $archive" >&2
    exit 1
fi

nm_stderr=$(mktemp "${TMPDIR:-/tmp}/elephc-pdo-nm.XXXXXX")
trap 'rm -f "$nm_stderr"' EXIT HUP INT TERM

nm_status=0
nm_output=$("$nm_bin" -u "$archive" 2>"$nm_stderr") || nm_status=$?

# GNU nm prints `tdsdbopen`; Darwin nm prefixes C symbols with `_`. Versioned
# ELF symbols may also carry an `@VERSION` suffix. Normalize all three forms.
symbols=$(printf '%s\n' "$nm_output" | awk '
    NF {
        symbol = $NF
        if (symbol !~ /:$/) {
            sub(/@.*/, "", symbol)
            sub(/^_/, "", symbol)
            print symbol
        }
    }
')

if printf '%s\n' "$symbols" | grep -Fx dbopen >/dev/null; then
    echo "error: PDO_DBLIB references optional FreeTDS symbol \`dbopen\`; use \`tdsdbopen(..., 0)\` for portable Sybase semantics" >&2
    exit 1
fi

if ! printf '%s\n' "$symbols" | grep -Fx tdsdbopen >/dev/null; then
    echo "error: PDO_DBLIB archive does not reference expected symbol \`tdsdbopen\`" >&2
    if [ "$nm_status" -ne 0 ]; then
        echo "error: $nm_bin exited with status $nm_status while inspecting the archive" >&2
    fi
    if [ -s "$nm_stderr" ]; then
        cat "$nm_stderr" >&2
    fi
    exit 1
fi

echo "PDO_DBLIB portable-symbol audit passed: tdsdbopen present, dbopen absent."
