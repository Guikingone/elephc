<?php
$values = preg_grep('/^[a-z]+$/', ["123", "charlie", "alpha", "bravo"]);
echo implode(",", array_values($values));
