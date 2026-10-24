<?php
$finfo = new finfo(FILEINFO_MIME_TYPE);
echo $finfo->buffer("hello"), "\n";
#[Export]
function ios_fileinfo_link_smoke(): int { return 0; }
