//! Purpose:
//! The `ext-dom` class surface (`DOMNode`, `DOMDocument`, `DOMElement`, `DOMText`,
//! `DOMNodeList`, `DOMNamedNodeMap`), implemented in elephc-PHP so it compiles through
//! the normal pipeline and therefore has EIR method bodies.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`,
//!   after include resolution and before name resolution.
//!
//! Key details:
//! - This REPLACES the former checker-only shells (`FlattenedClass` values injected
//!   straight into the checker's class map). Those type-checked but never reached
//!   lowering, so `new DOMDocument()` died at the backend with "constructor call to
//!   DOMDocument::__construct without an emitted EIR method body" — every DOM-using
//!   program was un-compilable no matter how clean its types were. Declaring the same
//!   surface as ordinary PHP gets it collected, checked, lowered and emitted like any
//!   user class, for free.
//! - The MEMBER SET and the STUB BODIES are transposed unchanged from those shells, so
//!   this swap is a delivery-mechanism change, not a semantic one. What the surface
//!   promises is exactly what it promised before.
//! - Still incomplete on purpose: there is no XML PARSER behind these classes. `loadXML()`
//!   and `schemaValidateSource()` report FAILURE rather than success — a shell may answer
//!   "I could not", never "I did". Claiming success would hand `XmlUtils::parse()` an empty
//!   document as though it were the parsed configuration and let a container be built from
//!   silently-missing input; returning false routes it into its own `XmlParsingException`
//!   path instead.
//! - Injected UNCONDITIONALLY, matching the reach the checker shells had (they were
//!   registered for every program). Unused classes are dropped by the closed-world prune,
//!   so a non-DOM binary carries nothing extra.
//! - Unused PARAMETERS are consumed by a `$_unused = …;` line rather than renamed. The
//!   `$_` prefix is what exempts a name from the unused-variable warning, and without it the
//!   prelude emitted five warnings on EVERY compile — `<?php echo "hi";` included, since the
//!   injection is unconditional. Renaming the parameters themselves would have silenced it too,
//!   but PHP named arguments make a parameter's name part of the public API, and a shell has no
//!   licence to rename `setAttribute(name:, value:)`.
//! - Method-local variables are `$_`-prefixed for the same reason as `pdo_prelude`: the
//!   checker resolves a method-body variable's type against top-level variables of the
//!   same name, so a user global named `$node` would otherwise clash with a plain
//!   method-local `$node`.

use crate::parser::ast::Program;

/// The elephc-PHP source declaring the DOM class surface.
///
/// `DOMNodeList` and `DOMNamedNodeMap` implement `Iterator` so `foreach ($node->childNodes
/// as $child)` and `foreach ($element->attributes as $name => $attr)` resolve; their cursor
/// methods carry no real state, mirroring the shells they replace. `DOMNamedNodeMap::key()`
/// returns a string rather than an index, matching PHP's name-keyed map.
pub const DOM_PRELUDE_SRC: &str = r#"<?php
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

    public function appendChild(DOMNode $node): DOMNode { return $node; }
}

class DOMDocument extends DOMNode {
    public bool $formatOutput = false;
    public bool $validateOnParse = false;

    public function __construct(string $version = "1.0", string $encoding = "") { $_unused = [$version, $encoding]; }

    public function createElement(string $name, mixed $value = ""): DOMElement {
        $_unused = [$name, $value];
        return new DOMElement();
    }

    public function createTextNode(mixed $data): DOMText { $_unused = $data; return new DOMText(); }

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

/// Prepends the DOM class surface to the program.
///
/// Injected unconditionally: the checker shells this replaces were registered for every
/// program, so gating injection on a usage scan would NARROW the surface rather than
/// preserve it. The prelude carries only declarations, which are discovered
/// position-independently, so prepending it does not change top-level execution order.
/// The prelude is static and tested, so a tokenize/parse failure is a compiler bug and
/// panics rather than silently degrading.
pub fn inject(program: Program) -> Program {
    let tokens = crate::lexer::tokenize(DOM_PRELUDE_SRC).expect("DOM prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("DOM prelude must parse");
    combined.extend(program);
    combined
}
