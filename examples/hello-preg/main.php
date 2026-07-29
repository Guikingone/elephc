<?php

$message = "hello preg";
if (preg_match("/^hello preg$/", $message)) {
    echo "Hello, preg!\n";
}
