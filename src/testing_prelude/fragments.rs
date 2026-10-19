//! Purpose:
//! The PHP form of the testing prelude, kept as the parse-parity oracle for the built
//! declarations in `crate::testing_prelude::build` and as the readable reference for the
//! runtime's behavior.
//!
//! Called from:
//! - `crate::testing_prelude::oracle_tests` and the transcription driver
//!   (`synthetic_class::transcribe::tests::dump_prelude_on_request`, `ELEPHC_TRANSCRIBE_WHICH=file`).
//!
//! Key details:
//! - NOT compiled into the prelude: `build.rs` carries the same declarations as AST. When
//!   this text changes, re-transcribe and update the builders, or the oracle fails.
//! - The two namespaces are BRACE blocks because the transcriber models `NamespaceBlock`;
//!   the alias extends the runtime class by its fully qualified name on purpose.
//! - `assertInstanceOf` is absent on purpose: `is_a()` with a runtime class name is the
//!   unfinished half of issue #1113.

/// The complete testing prelude as PHP source (with its `<?php` header).
pub(super) const SRC: &str = r##"<?php

namespace Elephc\Testing {

final class AssertionFailedError extends \Exception
{
}

final class SkippedTestError extends \Exception
{
}

abstract class TestCase
{
    private int $__elephc_assertions = 0;

    protected function setUp(): void
    {
    }

    protected function tearDown(): void
    {
    }

    public function __elephc_run_test(callable $test): void
    {
        $this->__elephc_assertions = 0;
        $this->setUp();
        try {
            $test();
        } finally {
            $this->tearDown();
        }
    }

    public function __elephc_assertion_count(): int
    {
        return $this->__elephc_assertions;
    }

    protected function fail(string $message = ''): void
    {
        throw new AssertionFailedError($message);
    }

    private function __elephc_pass(): void
    {
        $this->__elephc_assertions++;
    }

    private function __elephc_fail(string $message, string $description): void
    {
        $this->__elephc_assertions++;
        if ($message !== '') {
            throw new AssertionFailedError($message . "\n" . $description);
        }
        throw new AssertionFailedError($description);
    }

    private function __elephc_export(mixed $value): string
    {
        if ($value === null) {
            return 'null';
        }
        if (is_bool($value)) {
            return $value ? 'true' : 'false';
        }
        if (is_int($value) || is_float($value)) {
            return (string) $value;
        }
        if (is_string($value)) {
            return "'" . $value . "'";
        }
        if (is_array($value)) {
            return 'Array (' . count($value) . ')';
        }
        if (is_object($value)) {
            return get_class($value) . ' Object';
        }
        return gettype($value);
    }

    protected function assertTrue(bool $condition, string $message = ''): void
    {
        if ($condition) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that false is true.');
    }

    protected function assertFalse(bool $condition, string $message = ''): void
    {
        if (!$condition) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that true is false.');
    }

    protected function assertSame(mixed $expected, mixed $actual, string $message = ''): void
    {
        if ($expected === $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' is identical to ' . $this->__elephc_export($expected) . '.');
    }

    protected function assertNotSame(mixed $expected, mixed $actual, string $message = ''): void
    {
        if ($expected !== $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' is not identical to ' . $this->__elephc_export($expected) . '.');
    }

    protected function assertEquals(mixed $expected, mixed $actual, string $message = ''): void
    {
        if ($expected == $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' matches expected ' . $this->__elephc_export($expected) . '.');
    }

    protected function assertNotEquals(mixed $expected, mixed $actual, string $message = ''): void
    {
        if ($expected != $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' is not equal to ' . $this->__elephc_export($expected) . '.');
    }

    protected function assertNull(mixed $actual, string $message = ''): void
    {
        if ($actual === null) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' is null.');
    }

    protected function assertNotNull(mixed $actual, string $message = ''): void
    {
        if ($actual !== null) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that null is not null.');
    }

    protected function assertEmpty(mixed $actual, string $message = ''): void
    {
        if (empty($actual)) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' is empty.');
    }

    protected function assertNotEmpty(mixed $actual, string $message = ''): void
    {
        if (!empty($actual)) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $this->__elephc_export($actual) . ' is not empty.');
    }

    protected function assertCount(int $expected, array $haystack, string $message = ''): void
    {
        $actual = count($haystack);
        if ($expected === $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that actual size ' . $actual . ' matches expected size ' . $expected . '.');
    }

    protected function assertContains(mixed $needle, array $haystack, string $message = ''): void
    {
        if (in_array($needle, $haystack, true)) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that an array contains ' . $this->__elephc_export($needle) . '.');
    }

    protected function assertEqualsWithDelta(float $expected, float $actual, float $delta, string $message = ''): void
    {
        if (abs($expected - $actual) <= $delta) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' matches expected ' . $expected . ' within delta ' . $delta . '.');
    }

    protected function assertGreaterThan(int|float $expected, int|float $actual, string $message = ''): void
    {
        if ($actual > $expected) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' is greater than ' . $expected . '.');
    }

    protected function assertGreaterThanOrEqual(int|float $expected, int|float $actual, string $message = ''): void
    {
        if ($actual >= $expected) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' is equal to or greater than ' . $expected . '.');
    }

    protected function assertLessThan(int|float $expected, int|float $actual, string $message = ''): void
    {
        if ($actual < $expected) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' is less than ' . $expected . '.');
    }

    protected function assertLessThanOrEqual(int|float $expected, int|float $actual, string $message = ''): void
    {
        if ($actual <= $expected) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' is equal to or less than ' . $expected . '.');
    }

    protected function assertStringContainsString(string $needle, string $haystack, string $message = ''): void
    {
        if (str_contains($haystack, $needle)) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, "Failed asserting that '" . $haystack . "' contains \"" . $needle . "\".");
    }

    protected function assertStringStartsWith(string $prefix, string $string, string $message = ''): void
    {
        if (str_starts_with($string, $prefix)) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, "Failed asserting that '" . $string . "' starts with \"" . $prefix . "\".");
    }

    protected function assertStringEndsWith(string $suffix, string $string, string $message = ''): void
    {
        if (str_ends_with($string, $suffix)) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, "Failed asserting that '" . $string . "' ends with \"" . $suffix . "\".");
    }

    protected function assertSameInt(int $expected, int $actual, string $message = ''): void
    {
        if ($expected === $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' is identical to ' . $expected . '.');
    }

    protected function assertSameFloat(float $expected, float $actual, string $message = ''): void
    {
        if ($expected === $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . $actual . ' is identical to ' . $expected . '.');
    }

    protected function assertSameString(string $expected, string $actual, string $message = ''): void
    {
        if ($expected === $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, "Failed asserting that '" . $actual . "' is identical to '" . $expected . "'.");
    }

    protected function assertSameBool(bool $expected, bool $actual, string $message = ''): void
    {
        if ($expected === $actual) {
            $this->__elephc_pass();
            return;
        }
        $this->__elephc_fail($message, 'Failed asserting that ' . ($actual ? 'true' : 'false') . ' is identical to ' . ($expected ? 'true' : 'false') . '.');
    }
}

final class Runner
{
    private int $tests = 0;
    private int $assertions = 0;
    private int $failures = 0;
    private int $errors = 0;

    public function run(TestCase $case, string $name, callable $test): void
    {
        $this->tests++;
        $label = get_class($case) . '::' . $name;
        try {
            $case->__elephc_run_test($test);
            $count = $case->__elephc_assertion_count();
            $this->assertions += $count;
            echo 'ok   ' . $label . ' (' . $count . ($count === 1 ? ' assertion)' : ' assertions)') . "\n";
        } catch (AssertionFailedError $failure) {
            $this->assertions += $case->__elephc_assertion_count();
            $this->failures++;
            echo 'FAIL ' . $label . "\n";
            $this->echoIndented($failure->getMessage());
        } catch (\Throwable $error) {
            $this->assertions += $case->__elephc_assertion_count();
            $this->errors++;
            echo 'ERROR ' . $label . "\n";
            $this->echoIndented(get_class($error) . ': ' . $error->getMessage());
        }
    }

    private function echoIndented(string $text): void
    {
        $lines = explode("\n", $text);
        foreach ($lines as $line) {
            echo '     ' . $line . "\n";
        }
    }

    public function finish(): int
    {
        echo "\n";
        if ($this->tests === 0) {
            echo "No tests executed!\n";
            return 1;
        }
        if ($this->failures === 0 && $this->errors === 0) {
            echo 'OK (' . $this->tests . ($this->tests === 1 ? ' test, ' : ' tests, ') . $this->assertions . ($this->assertions === 1 ? ' assertion)' : ' assertions)') . "\n";
            return 0;
        }
        echo 'Tests: ' . $this->tests . ', Assertions: ' . $this->assertions . ', Failures: ' . $this->failures . ', Errors: ' . $this->errors . '.' . "\n";
        return 1;
    }
}

}

namespace PHPUnit\Framework {

abstract class TestCase extends \Elephc\Testing\TestCase
{
}

}
"##;
