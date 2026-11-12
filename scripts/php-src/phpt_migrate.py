#!/usr/bin/env python3
"""Turn passing php-src .phpt tests into Rust codegen tests.

usage: phpt_migrate.py ELEPHC TRIAGE_JSON OUT_DIR
Keeps tests whose stdout under elephc equals the PHP interpreter's stdout byte for byte
and contains no diagnostics. Writes OUT_DIR/<group>.rs (array, strings, math, json, zend...).
"""
import concurrent.futures as cf, json, os, re, subprocess, sys, tempfile
sys.path.insert(0, os.path.dirname(__file__))
from phpt_triage import sections  # noqa: E402

elephc, tri, out_dir = sys.argv[1], sys.argv[2], sys.argv[3]
res = json.load(open(tri))
DIAG = re.compile(rb"^(PHP )?(Warning|Notice|Deprecated|Fatal error|Parse error)\b", re.M)

def group(path):
    if "/Zend/tests/" in path: return "zend"
    if "/ext/json/" in path: return "json"
    return path.split("/ext/standard/tests/")[1].split("/")[0]

def check(path):
    secs = sections(path)
    code = secs["FILE"]
    try: code.encode("utf-8")
    except UnicodeEncodeError: return None
    with tempfile.TemporaryDirectory(dir=os.environ.get("TRIAGE_TMP")) as d:
        src = os.path.join(d, "t.php"); open(src, "w").write(code)
        ref = subprocess.run(["php", "-n", "-d", "display_errors=1", "-d", "error_reporting=-1", src], capture_output=True, cwd=d, timeout=20).stdout
        if DIAG.search(ref): return None
        c = subprocess.run([elephc, "--heap-size=8388608", src], capture_output=True, cwd=d, timeout=120)
        if c.returncode: return None
        r = subprocess.run([os.path.join(d, "t")], capture_output=True, cwd=d, timeout=20)
        if r.returncode or r.stdout != ref: return None
        try: exp = ref.decode("utf-8")
        except UnicodeDecodeError: return None
    title = secs.get("TEST", "").strip().splitlines()[0] if secs.get("TEST") else ""
    return path, title, code, exp

def rust_str(s):
    out = []
    for ch in s:
        o = ord(ch)
        if ch == "\\": out.append("\\\\")
        elif ch == '"': out.append('\\"')
        elif ch == "\n": out.append("\\n")
        elif ch == "\t": out.append("\\t")
        elif ch == "\r": out.append("\\r")
        elif o < 0x20 or o == 0x7f: out.append("\\u{%x}" % o)
        else: out.append(ch)
    return '"' + "".join(out) + '"'

def raw(s):
    n = 1
    while ('"' + "#" * n) in s: n += 1
    return "r" + "#" * n + '"' + s + '"' + "#" * n

paths = [p for p, (st, _) in res.items() if st == "pass"]
groups = {}
print("candidates", len(paths), file=sys.stderr)
with cf.ThreadPoolExecutor(os.cpu_count()) as ex:
    for r in ex.map(check, paths):
        if r: groups.setdefault(group(r[0]), []).append(r)
os.makedirs(out_dir, exist_ok=True)
for g, items in sorted(groups.items()):
    items.sort()
    body = f"""//! Purpose:
//! php-src `.phpt` conformance tests for {g}, migrated to Rust: each test compiles the
//! phpt `--FILE--` section and compares stdout with PHP's output.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`codegen::php_src`).
//!
//! Key details:
//! - Generated from php-src (https://github.com/php/php-src); only tests whose output elephc
//!   reproduces byte for byte (with no warnings) are kept. The expected strings are the PHP
//!   interpreter's own output, which matches the phpt `--EXPECT--` section.

use crate::support::*;
"""
    seen = set()
    for path, title, code, exp in items:
        rel = path.split("php-src/")[1]
        name = "phpt_" + re.sub(r"[^a-z0-9]+", "_", os.path.basename(path)[:-5].lower()).strip("_")
        while name in seen: name += "_2"
        seen.add(name)
        doc = title.replace("*/", "* /") or rel
        body += f"""
/// php-src `{rel}`: {doc}
#[test]
fn {name}() {{
    let out = compile_and_run({raw(code)});
    assert_eq!(out, {rust_str(exp)});
}}
"""
    open(os.path.join(out_dir, g + ".rs"), "w").write(body)
    print(g, len(items))
