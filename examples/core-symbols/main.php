<?php

class Document {
    public function __construct(public string $title) {}
}

// Cache derived data without keeping documents alive.
$titles = new WeakMap();
$document = new Document('Compiler notes');
$reference = WeakReference::create($document);
$titles[$document] = strtoupper($document->title);

echo $titles[$document], PHP_EOL;
unset($document);
echo 'Cached documents: ', count($titles), PHP_EOL;
echo $reference->get() === null ? 'Document released' : 'Document retained', PHP_EOL;

// Keep a source location and severity when converting an upload failure into an exception.
try {
    throw new ErrorException('No upload supplied', UPLOAD_ERR_NO_FILE, E_WARNING, 'upload.php', 12);
} catch (ErrorException $error) {
    echo $error->getFile(), ':', $error->getLine(), ': ', $error->getMessage(), PHP_EOL;
}
