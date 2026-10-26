<?php

// A cast can expose container data or the wrapper's own properties.
class Preferences extends ArrayObject
{
    public string $label = 'screen';
}

$data = new Preferences(['theme' => 'dark', 'pageSize' => 20]);
foreach ((array) $data as $key => $value) {
    echo $key, '=', $value, "\n";
}

$properties = new Preferences([], Preferences::STD_PROP_LIST);
foreach ((array) $properties as $key => $value) {
    echo $key, '=', $value, "\n";
}

echo 'Visible wrapper properties: ', count(get_object_vars($data)), "\n";
