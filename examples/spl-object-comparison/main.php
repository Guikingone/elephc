<?php

$invoice = new stdClass();
$reminder = new stdClass();
$pending = new SplObjectStorage();
$snapshot = new SplObjectStorage();

$pending[$invoice] = 'queued';
$pending[$reminder] = 'sent';
$snapshot[$reminder] = 'sent';
$snapshot[$invoice] = 'queued';

var_dump($pending == $snapshot);
$pending->next();
var_dump($pending == $snapshot);
$snapshot[$invoice] = 'sent';
var_dump($pending == $snapshot);
