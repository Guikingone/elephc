<?php

// Keep Unicode paragraph separators literal in an exported text payload.
$text = "First paragraph\u{2029}Second paragraph";
$flags = JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_LINE_TERMINATORS;
echo json_encode(["text" => $text], $flags) . "\n";

// Unit enum cases need an explicit representation before JSON export.
enum ExportState {
    case Pending;
}

$result = json_encode(ExportState::Pending);
if ($result === false && json_last_error() === JSON_ERROR_NON_BACKED_ENUM) {
    echo "Export failed: " . json_last_error_msg() . "\n";
}

// Choose a stable wire value for the public API.
echo json_encode(["state" => "pending"]) . "\n";
