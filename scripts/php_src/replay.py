#!/usr/bin/env python3
"""Replay unchanged SPL/OPcache PHPTs against a pinned php-src oracle and Elephc.

Every source test stays in the inventory. Unsupported adapters, oracle skips,
reference failures and runtime failures remain distinct from passing tests.
EXPECTF matching uses the actual pinned run-tests.php function through PHP/PCRE.
"""
import argparse
import base64
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time

PIN = "34308a6666b2d489c509541ea9befea9e2b42348"
SECTIONS = {"TEST", "DESCRIPTION", "CREDITS", "FILE", "FILE_EXTERNAL", "EXPECT",
            "EXPECTF", "INI", "ENV", "SKIPIF", "CLEAN", "EXTENSIONS", "XFAIL", "CONFLICTS"}
# Startup overrides copied from the pinned run-tests.php registration order.
DEFAULT_INI = ['output_handler=', 'open_basedir=', 'disable_functions=', 'output_buffering=Off', 'fatal_error_backtraces=Off', 'display_errors=1', 'display_startup_errors=1', 'log_errors=0', 'html_errors=0', 'track_errors=0', 'report_zend_debug=0', 'docref_root=', 'docref_ext=.html', 'error_prepend_string=', 'error_append_string=', 'auto_prepend_file=', 'auto_append_file=', 'ignore_repeated_errors=0', 'precision=14', 'serialize_precision=-1', 'memory_limit=128M', 'opcache.fast_shutdown=0', 'opcache.file_update_protection=0', 'opcache.revalidate_freq=0', 'opcache.jit_hot_loop=1', 'opcache.jit_hot_func=1', 'opcache.jit_hot_return=1', 'opcache.jit_hot_side_exit=1', 'opcache.jit_max_root_traces=100000', 'opcache.jit_max_side_traces=100000', 'opcache.jit_max_exit_counters=100000', 'opcache.protect_memory=1', 'zend.assertions=1', 'zend.exception_ignore_args=0', 'zend.exception_string_param_max_len=15', 'short_open_tag=0', 'date.timezone=UTC', 'error_reporting=30719']


def digest(data):
    """Return a stable SHA-256 for source, binaries, or evidence."""
    return hashlib.sha256(data).hexdigest()


def parse_sections(data):
    """Parse PHPT boundaries without changing the bytes inside any section."""
    marks = list(re.finditer(rb"^--([A-Z_]+)--[ \t]*\r?$", data, re.M))
    result = {}
    for i, mark in enumerate(marks):
        name = mark[1].decode("ascii")
        if name in result:
            raise ValueError("duplicate PHPT section: " + name)
        begin = mark.end()
        if data[begin:begin + 1] == b"\n":
            begin += 1
        result[name] = data[begin:marks[i + 1].start() if i + 1 < len(marks) else len(data)]
    if not marks or marks[0][1] != b"TEST":
        raise ValueError("missing initial TEST section")
    return result


def command(args, cwd, env, timeout, merge_output=False):
    """Run one isolated phase; keep stdout/stderr and make timeouts explicit."""
    started = time.monotonic()
    try:
        p = subprocess.run(args, cwd=cwd, env=env, stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT if merge_output else subprocess.PIPE,
                           timeout=timeout)
        return {"exit": p.returncode, "stdout": p.stdout, "stderr": p.stderr or b"",
                "seconds": round(time.monotonic() - started, 3)}
    except subprocess.TimeoutExpired as e:
        return {"exit": None, "stdout": e.stdout or b"", "stderr": e.stderr or b"",
                "timeout": True, "seconds": round(time.monotonic() - started, 3)}


def matcher_source(oracle):
    """Extract the authoritative EXPECTF function from the pinned upstream runner."""
    source = (oracle / "run-tests.php").read_text()
    start = source.index("\nfunction expectf_to_regex(")
    end = source.index("\n/**", start)
    function = source[start:end]
    return ("<?php\n" + function + r'''
$x = json_decode(stream_get_contents(STDIN), true, flags: JSON_THROW_ON_ERROR);
$expected = trim(str_replace("\r\n", "\n", base64_decode($x['expected'])));
$actual = trim(str_replace("\r\n", "\n", base64_decode($x['actual'])));
if ($x['kind'] === 'EXPECT') { echo $expected === $actual ? '1' : '0'; }
else { $re = expectf_to_regex($expected); $match = preg_match('/^' . $re . '$/s', $actual);
       if ($match === false) { fwrite(STDERR, preg_last_error_msg()); exit(2); }
       echo $match ? '1' : '0'; }
''').encode()


def matches(php, matcher, kind, expected, actual):
    """Compare raw bytes with upstream EXPECT/EXPECTF semantics, never strip diagnostics."""
    payload = json.dumps({"kind": kind, "expected": base64.b64encode(expected).decode(),
                          "actual": base64.b64encode(actual).decode()}).encode()
    p = subprocess.run([php, str(matcher)], input=payload, capture_output=True, timeout=10)
    if p.returncode:
        raise ValueError("upstream matcher failed: " + p.stderr.decode(errors="replace"))
    return p.stdout == b"1"


def settings(sections, directory):
    """Expand PHPT INI/ENV working-directory placeholders and preserve directive ordering."""
    def expand(raw):
        return raw.decode().replace("{PWD}", str(directory)).replace("{TMP}", str(directory))
    ini = DEFAULT_INI.copy()
    for line in expand(sections.get("INI", b"")).splitlines():
        if line.strip() and not line.lstrip().startswith(";"):
            if "=" not in line:
                raise ValueError("unsupported INI line: " + line)
            key, value = line.split("=", 1)
            value = value.strip()
            if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
                value = value[1:-1]
            ini.append(key.strip() + "=" + value)
    env = os.environ.copy()
    for line in expand(sections.get("ENV", b"")).splitlines():
        if line.strip():
            key, value = line.split("=", 1)
            env[key] = value
    return ini, env


def php_command(php, ini, source):
    """Materialize the reference startup INI settings in the PHPT's declared order."""
    return [php] + [arg for setting in ini for arg in ("-d", setting)] + [str(source)]


def copy_fixture_directory(source, destination):
    """Copy sibling fixtures, including subdirectories, without stale generated artifacts."""
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns("*.phpt", "*.s", "*.o"))


def record_phase(folder, phase, data):
    """Retain exact phase streams and return a compact JSON record."""
    result = {k: v for k, v in data.items() if k not in ("stdout", "stderr")}
    for stream in ("stdout", "stderr"):
        path = folder / (phase + "." + stream)
        path.write_bytes(data[stream])
        result[stream + "_sha256"] = digest(data[stream])
    return result


def replay_case(path, options, matcher, extensions):
    """Replay one PHPT with separate reference and native fixture directories."""
    relative = path.relative_to(options.php_src)
    evidence = options.output / "cases" / relative.with_suffix("")
    evidence.mkdir(parents=True, exist_ok=True)
    result = {"test": str(relative), "sha256": digest(path.read_bytes())}
    try:
        sections = parse_sections(path.read_bytes())
        unknown = sorted(sections.keys() - SECTIONS)
        if unknown:
            raise ValueError("unsupported sections: " + ", ".join(unknown))
        expectations = [x for x in ("EXPECT", "EXPECTF") if x in sections]
        if len(expectations) != 1 or sum(x in sections for x in ("FILE", "FILE_EXTERNAL")) != 1:
            raise ValueError("ambiguous or missing FILE/EXPECT section")
        result["xfail"] = sections.get("XFAIL", b"").decode(errors="replace").strip()
        required = sections.get("EXTENSIONS", b"").decode().split()
        missing = sorted(set(x.lower() for x in required) - extensions)
        if missing:
            return dict(result, status="oracle_skip", reason="missing reference extensions: " + ", ".join(missing))
        with tempfile.TemporaryDirectory(prefix="case-", dir=options.output / "tmp") as temporary:
            root = Path(temporary)
            for engine in ("reference", "native"):
                sandbox = root / engine
                suite = Path(*relative.parts[:2])
                copy_fixture_directory(options.php_src / suite, sandbox / suite)
                directory = sandbox / relative.parent
                ini, env = settings(sections, directory)
                source = directory / path.with_suffix(".php").name
                body = sections.get("FILE")
                if body is None:
                    external = (path.parent / sections["FILE_EXTERNAL"].decode().strip()).resolve()
                    if not external.is_relative_to(options.php_src):
                        raise ValueError("FILE_EXTERNAL escapes oracle")
                    body = external.read_bytes()
                source.write_bytes(body)
                executable = options.php if engine == "reference" else str(options.output / "unsupported_php_subprocess.py")
                env["TEST_PHP_EXECUTABLE"] = executable
                env["TEST_PHP_EXECUTABLE_ESCAPED"] = executable
                subprocess_flag = directory / ".unsupported-subprocess"
                env["ELEPHC_PHPT_SUBPROCESS_FLAG"] = str(subprocess_flag)
                if engine == "native" and b"TEST_PHP_EXECUTABLE" in body:
                    return dict(result, status="adapter_error", reason="PHP subprocess executable adapter is required")
                if engine == "reference" and "SKIPIF" in sections:
                    skip = directory / (path.stem + ".skip.php")
                    skip.write_bytes(sections["SKIPIF"])
                    data = command(php_command(options.php, ini, skip), directory, env, options.timeout)
                    result["skipif"] = record_phase(evidence, "skipif", data)
                    if data.get("timeout") or data["exit"] != 0:
                        return dict(result, status="oracle_error", reason="SKIPIF failed")
                    output = data["stdout"].strip()
                    if output.lower().startswith(b"skip"):
                        return dict(result, status="oracle_skip", reason=output.decode(errors="replace"))
                    if output and not output.lower().startswith((b"info", b"warn")):
                        return dict(result, status="oracle_error", reason="unexpected SKIPIF output")
                if engine == "native":
                    args = [str(options.elephc), "--quiet", "--php-version", "8.5"]
                    args += [arg for setting in ini for arg in ("--ini", setting)]
                    data = command(args + [str(source)], options.project, env, options.timeout)
                    result["compile"] = record_phase(evidence, "compile", data)
                    if data.get("timeout"):
                        return dict(result, status="compile_timeout")
                    if data["exit"] != 0:
                        return dict(result, status="compile_error")
                    args = [str(source.with_suffix(""))]
                else:
                    args = php_command(options.php, ini, source)
                data = command(args, directory, env, options.timeout,
                               merge_output=options.output_channel == "merged")
                result[engine] = record_phase(evidence, engine, data)
                if engine == "native" and subprocess_flag.exists():
                    return dict(result, status="adapter_error", reason="fixture requires a PHP subprocess executable adapter")
                kind = expectations[0]
                # Upstream PHPTs score descriptor 2 redirected into descriptor 1. The merged
                # mode preserves kernel write ordering and exposes duplicate diagnostics.
                # The optional stdout mode is explicitly an exploratory measurement only.
                ok = matches(options.php, matcher, kind, sections[kind], data["stdout"])
                if "CLEAN" in sections:
                    clean = directory / (path.stem + ".clean.php")
                    clean.write_bytes(sections["CLEAN"])
                    cleaned = command(php_command(options.php, ini, clean), directory, env, options.timeout)
                    result[engine + "_clean"] = record_phase(evidence, engine + "_clean", cleaned)
                    if cleaned.get("timeout") or cleaned["exit"] != 0:
                        return dict(result, status="adapter_error", reason=engine + " CLEAN failed")
                if engine == "reference":
                    if data.get("timeout") or not ok:
                        return dict(result, status="oracle_error", reason="reference did not match upstream expectation")
                else:
                    if data.get("timeout"):
                        return dict(result, status="runtime_timeout")
                    if data["exit"] < 0:
                        return dict(result, status="runtime_crash")
                    if not ok:
                        return dict(result, status="mismatch")
                    # Even if stdout matches, a different exit status is an observable gap.
                    if data["exit"] != result["reference"]["exit"]:
                        return dict(result, status="exit_mismatch")
        return dict(result, status="xfail_match" if result["xfail"] else
                    ("pass" if options.output_channel == "merged" else "stdout_pass"))
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        return dict(result, status="adapter_error", reason=str(error))


def main():
    """Verify provenance, inventory the full corpus, and publish resumable JSON evidence."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--php-src", type=Path, required=True)
    parser.add_argument("--elephc", type=Path, required=True)
    parser.add_argument("--project", type=Path, default=Path.cwd())
    parser.add_argument("--php", default="php")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--filter", default="")
    parser.add_argument("--output-channel", choices=("merged", "stdout"), default="merged",
                        help="merged stdout/stderr matches upstream PHPT scoring; stdout is exploratory")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=30)
    options = parser.parse_args()
    for field in ("php_src", "elephc", "project", "output"):
        setattr(options, field, getattr(options, field).resolve())
    head = subprocess.check_output(["git", "-C", str(options.php_src), "rev-parse", "HEAD"], text=True).strip()
    dirty = subprocess.check_output(["git", "-C", str(options.php_src), "status", "--porcelain"], text=True)
    if dirty:
        parser.error("oracle working tree must be clean")
    if head != PIN:
        parser.error("oracle must be php-8.5.10 at " + PIN + ", got " + head)
    version = subprocess.check_output([options.php, "-r", "echo PHP_VERSION;"], text=True).strip()
    if version != "8.5.10":
        parser.error("reference PHP must be 8.5.10, got " + version)
    corpus = sorted(p for area in ("spl", "opcache") for p in (options.php_src / "ext" / area).rglob("*.phpt"))
    pattern = re.compile(options.filter)
    selected = [p for p in corpus if pattern.search(str(p.relative_to(options.php_src)))]
    options.output.mkdir(parents=True, exist_ok=True)
    (options.output / "tmp").mkdir(exist_ok=True)
    if (options.output / "results.jsonl").exists():
        parser.error("output already contains results; choose a new evidence directory")
    inventory = [{"test": str(p.relative_to(options.php_src)), "sha256": digest(p.read_bytes()),
                  "selected": p in selected} for p in corpus]
    guard = options.output / "unsupported_php_subprocess.py"
    guard.write_text("#!/usr/bin/env python3\nimport os, pathlib, sys\n"
                     "pathlib.Path(os.environ['ELEPHC_PHPT_SUBPROCESS_FLAG']).touch()\n"
                     "sys.stderr.write('PHPT PHP subprocess adapter unavailable\\n')\n"
                     "sys.exit(125)\n")
    guard.chmod(0o755)
    matcher = options.output / "upstream_matcher.php"
    matcher.write_bytes(matcher_source(options.php_src))
    extension_json = subprocess.check_output([options.php, "-r", "echo json_encode(get_loaded_extensions());"], text=True)
    extensions = {x.lower().replace("zend opcache", "opcache") for x in json.loads(extension_json)}
    archives = [{"name": p.name, "sha256": digest(p.read_bytes())}
                for p in sorted(options.elephc.parent.glob("libelephc_*.a"))]
    metadata = {"oracle_tag": "php-8.5.10", "oracle_sha": head, "bridge_archives": archives, "php_version": version, "php_executable": shutil.which(options.php),
                "php_sha256": digest(Path(shutil.which(options.php)).read_bytes()), "compiler_sha256": digest(options.elephc.read_bytes()),
                "compiler_head": subprocess.check_output(["git", "-C", str(options.project), "rev-parse", "HEAD"], text=True).strip(),
                "compiler_diff_sha256": digest(subprocess.check_output(["git", "-C", str(options.project), "diff", "HEAD"])),
                "matcher_sha256": digest(matcher.read_bytes()), "runner_sha256": digest(Path(__file__).read_bytes()),
                "startup_ini": DEFAULT_INI, "inventory": inventory,
                "stdout_only": options.output_channel == "stdout", "target": "host", "jobs": options.jobs, "timeout": options.timeout,
                "unselected": len(corpus) - len(selected)}
    (options.output / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")
    counts = Counter()
    with (options.output / "results.jsonl").open("w") as output, ThreadPoolExecutor(max_workers=options.jobs) as pool:
        for result in pool.map(lambda p: replay_case(p, options, matcher, extensions), selected):
            counts[result["status"]] += 1
            output.write(json.dumps(result) + "\n")
            output.flush()
            if sum(counts.values()) % 25 == 0:
                print(json.dumps({"completed": sum(counts.values()), "selected": len(selected), "counts": counts}), flush=True)
    summary = {"inventory": len(corpus), "selected": len(selected), "unselected": len(corpus) - len(selected),
               "counts": counts, "complete": len(selected) == len(corpus), "full_compliance": False,
               "limitations": (["stdout-only comparison; stderr retained but unscored"]
                               if options.output_channel == "stdout" else []) + ["host AOT only", "PHP build configuration differs from a php-src in-tree build",
                               "reference PHP executes CLEAN for both sandboxes", "reference skips are not implementation passes"]}
    (options.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary), flush=True)
    # This adapter intentionally cannot certify stderr, eval, or the target matrix.
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
