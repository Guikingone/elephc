<?php

declare(strict_types=1);

// A compiled test suite: the classes below extend the runtime's TestCase, and the
// explicit driver at the bottom runs each test through the Runner. The same suite
// runs under php-src with PHPUnit installed, because the alias namespace is what
// PHPUnit itself declares.

namespace App;

final class Counter
{
    private int $value = 0;

    public function increment(int $by = 1): void
    {
        $this->value += $by;
    }

    public function value(): int
    {
        return $this->value;
    }
}

namespace App\Tests;

use App\Counter;
use PHPUnit\Framework\TestCase;

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
        $this->counter->increment();
        $this->counter->increment(2);
        $this->assertSame(3, $this->counter->value());
        $this->assertGreaterThan(2, $this->counter->value());
    }

    public function testSetUpRunsBeforeEveryTest(): void
    {
        // A fresh Counter: the increments of the previous test are gone.
        $this->assertSame(0, $this->counter->value());
    }
}

final class StringsTest extends TestCase
{
    public function testHaystacks(): void
    {
        $this->assertStringContainsString('eph', 'elephc');
        $this->assertStringStartsWith('ele', 'elephc');
        $this->assertCount(2, explode(' ', 'compiled php'));
        $this->assertEqualsWithDelta(0.3, 0.1 + 0.2, 1e-9);
    }
}

namespace App;

use App\Tests\CounterTest;
use App\Tests\StringsTest;
use Elephc\Testing\Runner;

$runner = new Runner();

$counter = new CounterTest();
$runner->run($counter, 'testStartsAtZero', fn() => $counter->testStartsAtZero());
$runner->run($counter, 'testIncrements', fn() => $counter->testIncrements());
$runner->run($counter, 'testSetUpRunsBeforeEveryTest', fn() => $counter->testSetUpRunsBeforeEveryTest());

$strings = new StringsTest();
$runner->run($strings, 'testHaystacks', fn() => $strings->testHaystacks());

exit($runner->finish());
