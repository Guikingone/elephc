<?php

$source = $argc > 1
    ? $argv[1]
    : '$ok = preg_match("/([a-z]+)([0-9]+)/", "id42", $matches);'
        . ' echo $ok . ":" . $matches[1] . ":" . $matches[2] . "\n";';

eval($source);
