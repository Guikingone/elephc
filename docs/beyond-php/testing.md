---
title: "Testing"
description: "The native test runtime: Elephc\\Testing\\TestCase, the supported PHPUnit-compatible assertion subset, the Runner and its report, and the PHPUnit\\Framework\\TestCase alias."
sidebar:
  order: 13
---

elephc ships a test runtime inside the compiler, so a suite tests the compiled binary
instead of php-src. It matters twice: a project that uses elephc extensions (`.lfc`,
`packed class`, `buffer<T>`, `extern`) cannot run PHPUnit at all, and a plain-PHP project
compiled by elephc wants to verify what the binary does, which can differ from Zend.

The runtime is a prelude: it is declared only when a program references it, costs nothing
otherwise, and needs no package to install. The design and the roadmap of the runner live
in `.plans/elephc-test.md`.

## Writing a test

A test class extends `Elephc\Testing\TestCase`. Public methods are the tests; `setUp()` and
`tearDown()` run around each one:

```php
<?php

namespace App\Tests;

use App\Counter;
use Elephc\Testing\TestCase;

final class CounterTest extends TestCase
{
    private Counter $counter;

    protected function setUp(): void
    {
        $this->counter = new Counter();
    }

    public function testStartsAtZero(): void
    {
        $this->assertSame(0, $this->counter->value());
    }

    public function testIncrements(): void
    {
        $this->counter->increment(2);
        $this->assertSameInt(2, $this->counter->value());
    }
}
```

A suite written against PHPUnit compiles unchanged: `PHPUnit\Framework\TestCase` is
declared as an alias of the elephc class, so the same file runs under php-src with the real
PHPUnit installed and under elephc with nothing installed. Only the subset documented here
exists; a PHPUnit method outside it is a compile error (`Undefined method`), never a silent
no-op.

## Running a suite

Until the `elephc test` subcommand lands, a suite is an ordinary program with an explicit
driver: one `Runner`, one call per test, and the runner's verdict as the exit code.

```php
<?php

use App\Tests\CounterTest;
use Elephc\Testing\Runner;

$runner = new Runner();

$counter = new CounterTest();
$runner->run($counter, 'testStartsAtZero', fn() => $counter->testStartsAtZero());
$runner->run($counter, 'testIncrements', fn() => $counter->testIncrements());

exit($runner->finish());
```

```text
$ elephc tests/main.php && ./tests/main
ok   App\Tests\CounterTest::testStartsAtZero (1 assertion)
ok   App\Tests\CounterTest::testIncrements (1 assertion)

OK (2 tests, 2 assertions)
```

The test is passed as a closure rather than by name on purpose: declaration reachability
compiles only what the program reaches, and `$case->testIncrements()` inside the closure is
what keeps the method in the binary. `elephc test` will synthesize exactly this driver from
the classes it discovers.

`Runner::run()` records one of three outcomes per test and prints one line for it:

| Line | Meaning |
|---|---|
| `ok   Class::method (n assertions)` | Every assertion passed. |
| `FAIL Class::method` | An assertion failed, or `fail()` was called. The message follows, indented. |
| `ERROR Class::method` | The test threw anything else. The exception class and message follow. |

`Runner::finish()` prints the totals and returns the exit code: `0` only when every test
passed and at least one ran.

```text
Tests: 4, Assertions: 12, Failures: 1, Errors: 1.
```

`tearDown()` always runs after a `setUp()` that succeeded, whatever the test did. A user
message given to an assertion is printed on its own line above the assertion text, as
PHPUnit prints it.

## Assertions

Every assertion is `protected`, returns `void`, takes an optional trailing
`string $message = ''`, counts itself, and throws `Elephc\Testing\AssertionFailedError`
when it fails. Failure text follows PHPUnit's wording where elephc can reproduce it.

| Assertion | Passes when |
|---|---|
| `assertTrue(bool $condition)` | `$condition` is `true` |
| `assertFalse(bool $condition)` | `$condition` is `false` |
| `assertSame(mixed $expected, mixed $actual)` | `$expected === $actual` |
| `assertNotSame(mixed $expected, mixed $actual)` | `$expected !== $actual` |
| `assertEquals(mixed $expected, mixed $actual)` | `$expected == $actual` |
| `assertNotEquals(mixed $expected, mixed $actual)` | `$expected != $actual` |
| `assertNull(mixed $actual)` | `$actual === null` |
| `assertNotNull(mixed $actual)` | `$actual !== null` |
| `assertEmpty(mixed $actual)` | `empty($actual)` |
| `assertNotEmpty(mixed $actual)` | `!empty($actual)` |
| `assertCount(int $expected, array $haystack)` | `count($haystack) === $expected` |
| `assertContains(mixed $needle, array $haystack)` | `in_array($needle, $haystack, true)` |
| `assertEqualsWithDelta(float $expected, float $actual, float $delta)` | `abs($expected - $actual) <= $delta` |
| `assertGreaterThan(int\|float $expected, int\|float $actual)` | `$actual > $expected` |
| `assertGreaterThanOrEqual(int\|float $expected, int\|float $actual)` | `$actual >= $expected` |
| `assertLessThan(int\|float $expected, int\|float $actual)` | `$actual < $expected` |
| `assertLessThanOrEqual(int\|float $expected, int\|float $actual)` | `$actual <= $expected` |
| `assertStringContainsString(string $needle, string $haystack)` | `str_contains($haystack, $needle)` |
| `assertStringStartsWith(string $prefix, string $string)` | `str_starts_with($string, $prefix)` |
| `assertStringEndsWith(string $suffix, string $string)` | `str_ends_with($string, $suffix)` |
| `fail(string $message = '')` | never: throws immediately |

The `mixed` parameters box a scalar into a runtime cell. Four elephc-only variants take the
typed value directly and print the same failure text, for suites that assert in hot loops:

| Assertion | Passes when |
|---|---|
| `assertSameInt(int $expected, int $actual)` | `$expected === $actual` |
| `assertSameFloat(float $expected, float $actual)` | `$expected === $actual` |
| `assertSameString(string $expected, string $actual)` | `$expected === $actual` |
| `assertSameBool(bool $expected, bool $actual)` | `$expected === $actual` |

These four do not exist in PHPUnit: a suite that must also run under php-src uses the
`mixed` forms.

`assertInstanceOf` is deliberately absent. `is_a()` with a class name that is only known at
run time is the unfinished half of [issue #1113](https://github.com/illegalstudio/elephc/issues/1113)
and currently answers `false`, so the assertion would fail every time; write
`$this->assertTrue($value instanceof Foo)` instead, which the compiler resolves statically.

## Classes

| Class | Role |
|---|---|
| `Elephc\Testing\TestCase` | Abstract base of every test class: lifecycle hooks and the assertions above. |
| `Elephc\Testing\Runner` | `run(TestCase $case, string $name, callable $test)` and `finish(): int`. |
| `Elephc\Testing\AssertionFailedError` | Thrown by a failing assertion and by `fail()`; extends `Exception`. |
| `Elephc\Testing\SkippedTestError` | Reserved for `markTestSkipped()`. |
| `PHPUnit\Framework\TestCase` | Empty abstract subclass of `Elephc\Testing\TestCase`: the compatibility alias. |

## Limits

- One instance of a test class serves every test method in a hand-written driver; `setUp()`
  reinitializes state before each test, as the example above relies on.
- A native fatal (buffer out of bounds, null pointer dereference, FFI segfault) ends the
  process: the runner cannot report it as a failure yet. Per-test process isolation is the
  next step of the plan.
- Data providers, `expectException()`, skipped and incomplete tests, JUnit and TAP output
  are not available yet.
