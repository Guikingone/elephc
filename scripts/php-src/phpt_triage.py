#!/usr/bin/env python3
"""Triage php-src .phpt tests against elephc.

usage: phpt_triage.py ELEPHC OUT_JSON DIR...
For each phpt with a plain FILE + EXPECT/EXPECTF section (no SKIPIF/INI/ARGS/STDIN/...),
runs it with the local `php` (reference) and requires that php output matches the phpt
expectation; then compiles+runs it with elephc and records pass/compile_fail/mismatch/crash.
"""
import concurrent.futures as cf, json, os, re, subprocess, sys, tempfile

BAD = {"SKIPIF", "INI", "ARGS", "STDIN", "POST", "POST_RAW", "GET", "COOKIE", "ENV", "EXTENSIONS",
       "CGI", "FILEEOF", "FILE_EXTERNAL", "REDIRECTTEST", "CAPTURE_STDIO", "PHPDBG", "CONFLICTS",
       "XFAIL", "CLEAN", "GZIP_POST", "DEFLATE_POST", "EXPECTREGEX", "EXPECTHEADERS", "FLAKY", "WHITESPACE_SENSITIVE"}

def sections(path):
    secs, cur = {}, None
    with open(path, encoding="utf-8", errors="surrogateescape") as f:
        for line in f:
            m = re.match(r"^--([A-Z_]+)--\s*$", line)
            if m:
                cur = m.group(1); secs[cur] = ""; continue
            if cur: secs[cur] += line
    return secs

def expectf_regex(s):
    out = ""
    i = 0
    rep = {"%s": r"[^\r\n]+", "%S": r"[^\r\n]*", "%a": r".+", "%A": r".*", "%w": r"\s*", "%i": r"[+-]?\d+",
           "%d": r"\d+", "%x": r"[0-9a-fA-F]+", "%f": r"[+-]?\.?\d+\.?\d*(?:[Ee][+-]?\d+)?", "%c": r".", "%e": r"[\\/]", "%0": r"\x00"}
    while i < len(s):
        two = s[i:i+2]
        if two in rep: out += rep[two]; i += 2
        elif s.startswith("%r", i):
            j = s.index("%r", i+2); out += "(?:" + s[i+2:j] + ")"; i = j + 2
        else: out += re.escape(s[i]); i += 1
    return re.compile(out, re.S)

def norm(s): return re.sub(r"[ \t]+\n", "\n", s.replace("\r\n", "\n")).strip()

def matches(secs, out):
    out = norm(out)
    if "EXPECT" in secs: return out == norm(secs["EXPECT"])
    if "EXPECTF" in secs:
        try: return expectf_regex(norm(secs["EXPECTF"])).fullmatch(out) is not None
        except Exception: return False
    return False

DIAG = re.compile(r"^(PHP )?(Warning|Notice|Deprecated|Fatal error|Parse error)\b.*$", re.M)
def strip_diag(s):
    return "\n".join(l for l in DIAG.sub("", s.replace("\r\n", "\n")).split("\n") if l.strip())

def run(path):
    secs = sections(path)
    if "FILE" not in secs or not ({"EXPECT", "EXPECTF"} & secs.keys()) or (BAD & secs.keys()):
        return path, "skip", ""
    code = secs["FILE"]
    with tempfile.TemporaryDirectory(dir=os.environ.get("TRIAGE_TMP")) as d:
        src = os.path.join(d, "t.php")
        open(src, "w", encoding="utf-8", errors="surrogateescape").write(code)
        try:
            ref = subprocess.run(["php", "-n", "-d", "display_errors=1", "-d", "error_reporting=-1", "-d", "html_errors=0", src],
                                 capture_output=True, timeout=20, cwd=d).stdout.decode("utf-8", "replace")
        except subprocess.TimeoutExpired:
            return path, "skip_php", ""
        if not matches(secs, ref):
            return path, "skip_php_differs", ""
        try:
            c = subprocess.run([elephc, src], capture_output=True, timeout=120, cwd=d)
        except subprocess.TimeoutExpired:
            return path, "compile_timeout", ""
        if c.returncode != 0:
            err = (c.stderr + c.stdout).decode("utf-8", "replace")
            first = next((l for l in err.splitlines() if "error" in l.lower()), err.strip()[:200])
            return path, "compile_fail", first[:300]
        try:
            r = subprocess.run([os.path.join(d, "t")], capture_output=True, timeout=20, cwd=d)
        except subprocess.TimeoutExpired:
            return path, "run_timeout", ""
        out = r.stdout.decode("utf-8", "replace") + r.stderr.decode("utf-8", "replace")
        if r.returncode < 0: return path, "crash", f"signal {-r.returncode}"
        if matches(secs, out): return path, "pass", ""
        if strip_diag(out) == strip_diag(ref): return path, "diag_only", ""
        return path, "mismatch", ""

if __name__ == "__main__":
    elephc, out_json, dirs = sys.argv[1], sys.argv[2], sys.argv[3:]
    files = sorted(p for d in dirs for root, _, fs in os.walk(d) for p in [os.path.join(root, f) for f in fs] if p.endswith(".phpt"))
    res = {}
    with cf.ThreadPoolExecutor(os.cpu_count()) as ex:
        for path, status, info in ex.map(run, files):
            res[path] = [status, info]
    json.dump(res, open(out_json, "w"), indent=0)
    from collections import Counter
    print(Counter(s for s, _ in res.values()))
    