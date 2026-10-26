<?php
// A cache consumer may know a property name before it knows the stored value type.
class CachedPayload {
    public $value;
}

$cached = 'O:13:"CachedPayload":1:{s:5:"value";s:5:"ready";}';
$payload = unserialize($cached);
echo json_encode($payload), "\n";
