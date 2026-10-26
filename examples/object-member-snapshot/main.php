<?php

class Job
{
    public string $title = 'Index build';
    private int $attempts = 2;
}

$inspect = get_mangled_object_vars(...);
$members = $inspect(new Job());
echo $members['title'], ':', $members["\0Job\0attempts"], "\n";
