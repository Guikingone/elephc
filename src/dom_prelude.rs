//! Purpose:
//! Declares the standard DOM class surface as ordinary PHP so references type-check and lower
//! through the same class and method pipeline as user declarations.
//!
//! Called from:
//! - `crate::pipeline::compile()` and frontend test harnesses before name resolution.
//!
//! Key details:
//! - The declarations model the shared node hierarchy, iterable node collections, properties,
//!   named parameters, and return types used by general DOM consumers.
//! - Parsing and schema validation report failure until an XML parser backs this surface; they
//!   never claim that an empty placeholder document successfully parsed user input.
//! - Closed-world pruning removes these declarations from programs that never reference them.

use crate::parser::ast::Program;

/// PHP source for the standard DOM class surface currently supported by the native backend.
pub const DOM_PRELUDE_SRC: &str = r#"<?php
const XML_DOCUMENT_TYPE_NODE = 10;

class DOMNodeList implements Iterator {
    public function item(int $index): ?DOMNode { $_unused = $index; return null; }
    public function current(): mixed { return null; }
    public function key(): mixed { return 0; }
    public function next(): void {}
    public function rewind(): void {}
    public function valid(): bool { return false; }
}

class DOMNamedNodeMap implements Iterator {
    public function getNamedItem(string $qualifiedName): ?DOMNode { $_unused = $qualifiedName; return null; }
    public function item(int $index): ?DOMNode { $_unused = $index; return null; }
    public function current(): mixed { return null; }
    public function key(): mixed { return ""; }
    public function next(): void {}
    public function rewind(): void {}
    public function valid(): bool { return false; }
}

class DOMNode {
    public ?DOMDocument $ownerDocument = null;
    public DOMNodeList $childNodes;
    public ?string $nodeValue = null;
    public string $prefix = "";
    public string $localName = "";
    public ?DOMNamedNodeMap $attributes = null;

    public function __construct() { $this->childNodes = new DOMNodeList(); }

    public function appendChild(DOMNode $node): DOMNode { return $node; }
}

class DOMDocument extends DOMNode {
    public bool $formatOutput = false;
    public bool $validateOnParse = false;

    public function __construct(string $version = "1.0", string $encoding = "") {
        $_unused = [$version, $encoding];
        $this->childNodes = new DOMNodeList();
    }

    public function createElement(string $name, mixed $value = ""): DOMElement {
        $_unused = [$name, $value];
        $element = new DOMElement();
        $element->ownerDocument = $this;
        return $element;
    }

    public function createTextNode(mixed $data): DOMText {
        $text = new DOMText();
        $text->ownerDocument = $this;
        $text->nodeValue = $data;
        return $text;
    }

    public function saveXML(?DOMNode $node = null): string { $_unused = $node; return ""; }

    public function importNode(DOMNode $node, bool $deep = false): DOMNode { $_unused = $deep; return $node; }

    public function getElementsByTagName(string $qualifiedName): DOMNodeList {
        $_unused = $qualifiedName;
        return new DOMNodeList();
    }

    public function loadXML(string $source, int $options = 0): bool { $_unused = [$source, $options]; return false; }

    public function normalizeDocument(): void {}

    public function schemaValidateSource(string $source, int $flags = 0): bool { $_unused = [$source, $flags]; return false; }
}

class DOMElement extends DOMNode {
    public function setAttribute(mixed $name, mixed $value): void { $_unused = [$name, $value]; }
}

class DOMText extends DOMNode {
}
"#;

/// Prepends the DOM declarations before namespace resolution and class collection.
///
/// The surface is always available, matching a PHP runtime with the DOM extension enabled.
/// Its declarations are position-independent, and closed-world pruning drops unused classes.
pub fn inject(program: Program) -> Program {
    let tokens = crate::lexer::tokenize(DOM_PRELUDE_SRC).expect("DOM prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("DOM prelude must parse");
    combined.extend(program);
    combined
}
