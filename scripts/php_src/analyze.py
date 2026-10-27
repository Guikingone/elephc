"""Inspect PHPT replay evidence produced by replay.py.

usage:
  analyze.py diff OLD_REPLAY NEW_REPLAY
      per-test status moves between two replays, most frequent first
  analyze.py clusters REPLAY [LIMIT]
      compile failures grouped by first error message (names and digits folded)
  analyze.py lines REPLAY PATTERN [--php-src DIR]
      for each case whose first compile error matches PATTERN: the error and its source line
  analyze.py recheck ELEPHC LIST WORK_DIR [--full] [--php-src DIR]
      extract each listed PHPT's FILE section (paths relative to the php-src root), compile
      it with ELEPHC (`--check` unless --full) and print the first error or OK

The php-src checkout defaults to `.artifacts/php-src-oracle` in the current directory.
"""
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path


def option(args, name, default):
    """Removes `name VALUE` from args and returns VALUE, or `default`."""
    if name in args:
        index = args.index(name)
        value = args[index + 1]
        del args[index:index + 2]
        return value
    return default


def statuses(replay):
    """Maps each test of a replay to its final status."""
    rows = (json.loads(line) for line in (Path(replay) / 'results.jsonl').read_text().splitlines())
    return {row['test']: row['status'] for row in rows}


def first_error(text):
    """Returns the first `error...` line of a compiler's output, or None."""
    return next((line for line in text.splitlines() if line.startswith('error')), None)


def file_section(phpt):
    """Returns the bytes of a PHPT's FILE section, or None."""
    match = re.search(rb'^--FILE--\r?\n(.*?)(?=^--[A-Z_]+--\r?$)', phpt.read_bytes(), re.S | re.M)
    return match.group(1) if match else None


def diff(old, new):
    before, after = statuses(old), statuses(new)
    moves, examples = Counter(), {}
    for test, state in after.items():
        if before.get(test) != state:
            key = (before.get(test), state)
            moves[key] += 1
            examples.setdefault(key, []).append(test)
    for (was, now), count in moves.most_common():
        print(f'{count:4d} {was} -> {now}: {", ".join(examples[(was, now)][:4])}')


def clusters(replay, limit=40):
    counts = Counter()
    for err in Path(replay).glob('cases/**/compile.stderr'):
        line = first_error(err.read_text(errors='replace'))
        if not line:
            continue
        message = line.split(']: ', 1)[-1]
        message = re.sub(r"'[^']*'", "'_'", message)
        message = re.sub(r'\$\w+', '$_', message)
        message = re.sub(r'\d+', 'N', message)
        counts[message[:110]] += 1
    for message, count in counts.most_common(limit):
        print(f'{count:4d} {message}')
    print('total', sum(counts.values()))


def lines(replay, pattern, php_src):
    regex = re.compile(pattern)
    cases = Path(replay) / 'cases'
    for err in sorted(cases.glob('**/compile.stderr')):
        line = first_error(err.read_text(errors='replace'))
        if not line or not regex.search(line):
            continue
        name = str(err.parent.relative_to(cases))
        match = re.search(r':(\d+):\d+\]: (.*)$', line)
        source = '?'
        section = file_section(php_src / (name + '.phpt')) if (php_src / (name + '.phpt')).exists() else None
        if match and section:
            numbered = section.decode(errors='replace').splitlines()
            index = int(match.group(1))
            if 0 < index <= len(numbered):
                source = numbered[index - 1].strip()[:120]
        print(f"{name.split('/')[-1]} | {(match.group(2) if match else line)[:70]} | {source}")


def recheck(elephc, listing, work, full, php_src):
    work.mkdir(parents=True, exist_ok=True)
    names = [line.strip() for line in Path(listing).read_text().splitlines() if line.strip()]
    ok = 0
    for name in names:
        section = file_section(php_src / (name if name.endswith('.phpt') else name + '.phpt'))
        if section is None:
            print(f'{name}: no FILE section')
            continue
        source = work / (name.replace('/', '__').removesuffix('.phpt') + '.php')
        source.write_bytes(section)
        args = [elephc] + ([] if full else ['--check']) + [source.name]
        try:
            result = subprocess.run(args, cwd=work, capture_output=True, timeout=120)
        except subprocess.TimeoutExpired:
            print(f'{name}: TIMEOUT')
            continue
        text = (result.stderr + result.stdout).decode(errors='replace')
        error = first_error(text) or next((l for l in text.splitlines() if 'backend error' in l), None)
        if result.returncode == 0 and error is None:
            ok += 1
            print(f'{name}: OK')
        else:
            error = error or (text.strip().splitlines() or ['?'])[-1]
            print(f'{name}: {re.sub(r"error.[^]]*]: ", "", error)}')
    print(f'OK {ok}/{len(names)}')


def main(argv):
    args = list(argv)
    php_src = Path(option(args, '--php-src', '.artifacts/php-src-oracle'))
    full = '--full' in args
    if full:
        args.remove('--full')
    if not args:
        sys.exit(__doc__)
    command, rest = args[0], args[1:]
    if command == 'diff' and len(rest) == 2:
        diff(*rest)
    elif command == 'clusters' and len(rest) in (1, 2):
        clusters(rest[0], int(rest[1]) if len(rest) == 2 else 40)
    elif command == 'lines' and len(rest) == 2:
        lines(rest[0], rest[1], php_src)
    elif command == 'recheck' and len(rest) == 3:
        recheck(rest[0], rest[1], Path(rest[2]), full, php_src)
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main(sys.argv[1:])
