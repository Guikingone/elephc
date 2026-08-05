<?php

declare(strict_types=1);

// elephc has one parameter-binding model, so the directive changes nothing.
echo "strict_types is parsed and ignored\n";

declare(ticks=1) {
    echo "braced declare body\n";
}

declare(ticks=1):
    echo "alternative declare body\n";
enddeclare;
