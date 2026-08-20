<?php
// Only declarations reachable from executable code are lowered to native assembly.

interface Report {
    public function render(): string;
}

class TextReport implements Report {
    public function render(): string {
        return "ready";
    }

    public function debugPreview(): string {
        return "debug-only";
    }
}

class LegacyReport implements Report {
    public function render(): string {
        return "legacy";
    }
}

function buildReport(): Report {
    return new TextReport();
}

function unusedLegacyFactory(): Report {
    return new LegacyReport();
}

echo buildReport()->render() . "\n";
