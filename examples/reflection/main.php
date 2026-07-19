<?php

// Reflecting over a function's signature at compile time.
//
// elephc resolves reflection in a closed world: ReflectionFunction and
// ReflectionParameter read metadata baked from the function declaration, so
// parameter names, positions, optionality and declared types are all known.

interface Notifier {}
trait Loggable {}

class Mailer implements Notifier {
    use Loggable;
}

function send(string $to, Mailer $mailer, int $retries = 3, ?string $subject = null): void
{
}

$fn = new ReflectionFunction('send');

echo $fn->getName(), " takes ", $fn->getNumberOfParameters(), " parameters",
    " (", $fn->getNumberOfRequiredParameters(), " required)\n";

foreach ($fn->getParameters() as $param) {
    echo "  #", $param->getPosition(), " \$", $param->getName();

    if ($param->hasType()) {
        $type = $param->getType();
        echo ": ";
        if ($type->allowsNull()) {
            echo "?";
        }
        echo $type->getName();
        echo $type->isBuiltin() ? " (builtin)" : " (class)";
    } else {
        echo ": mixed (no type hint)";
    }

    if ($param->isOptional()) {
        echo " [optional]";
    }

    echo "\n";
}

// Reflecting a class chosen at RUNTIME (not a compile-time literal).
//
// `new ReflectionClass($className)` also accepts a name that is only known
// while the program is running — here, the first CLI argument, defaulting to
// "Mailer" when none is given. elephc resolves it through a shared,
// case-insensitive dispatcher built from the program's closed world of
// declared classes; an unknown name throws a real, catchable
// \ReflectionException instead of crashing.
$className = $argv[1] ?? 'Mailer';

try {
    $rc = new ReflectionClass($className);
    echo "Reflecting class \"", $rc->getName(), "\" (short name: ", $rc->getShortName(), ")\n";
    echo "  instantiable: ", $rc->isInstantiable() ? "yes" : "no", "\n";
} catch (\ReflectionException $e) {
    echo "No such class: ", $e->getMessage(), "\n";
}

// class_implements()/class_parents()/class_uses() on that same RUNTIME-chosen
// name. A non-literal name (or an object argument) resolves through a
// closed-world per-class relation registry instead of compile-time-folded
// metadata; an unknown name returns `false` rather than throwing.
$implements = class_implements($className);
if ($implements === false) {
    echo "  implements: (unknown class)\n";
} else {
    echo "  implements:";
    foreach ($implements as $interfaceName => $_) {
        echo " ", $interfaceName;
    }
    echo "\n";
}

$uses = class_uses($className);
echo "  uses traits:";
if ($uses !== false) {
    foreach ($uses as $traitName => $_) {
        echo " ", $traitName;
    }
}
echo "\n";
