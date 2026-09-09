//! Purpose:
//! Oracle-pinned reflection coverage for the complete public PHP 8.5 DOM,
//! libxml, and SimpleXML surface.
//!
//! Called from:
//! - `cargo test --test codegen_tests codegen::dom_reflection_surface`.
//!
//! Key details:
//! - Expectations were captured with PHP 8.5.8 and libxml2 2.15.3.
//! - Reflection checks make hierarchy, signatures, virtual-property metadata, and
//!   extension registration independently observable from native DOM behaviour.

use crate::support::compile_and_run;

/// Verifies legacy and modern DOM hierarchy, interfaces, finality, construction, and cloning metadata.
#[test]
fn dom_reflection_class_hierarchy_matches_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function flag(bool $value): string {
    return $value ? "yes" : "no";
}

function class_row(string $name): void {
    $reflection = new ReflectionClass($name);
    $parent = $reflection->getParentClass();
    echo $name, "|", flag($reflection->isInterface()), "|";
    echo flag($reflection->isAbstract()), "|", flag($reflection->isFinal()), "|";
    echo flag($reflection->isInstantiable()), "|", flag($reflection->isCloneable()), "|";
    echo ($parent ? $parent->getName() : "-"), "|";
    echo implode(",", $reflection->getInterfaceNames()), "\n";
}

class_row("DOMDocument");
class_row("DOMNode");
class_row("Dom\\Document");
class_row("Dom\\XMLDocument");
class_row("Dom\\Element");
class_row("DOMNodeList");
class_row("DOMNamedNodeMap");
class_row("SimpleXMLElement");
class_row("SimpleXMLIterator");
class_row("DOMException");
"#,
    );

    assert_eq!(
        output,
        concat!(
            "DOMDocument|no|no|no|yes|yes|DOMNode|DOMParentNode\n",
            "DOMNode|no|no|no|yes|yes|-|\n",
            "Dom\\Document|no|yes|no|no|no|Dom\\Node|Dom\\ParentNode\n",
            "Dom\\XMLDocument|no|no|yes|no|yes|Dom\\Document|Dom\\ParentNode\n",
            "Dom\\Element|no|no|no|no|yes|Dom\\Node|Dom\\ParentNode,Dom\\ChildNode\n",
            "DOMNodeList|no|no|no|yes|no|-|IteratorAggregate,Traversable,Countable\n",
            "DOMNamedNodeMap|no|no|no|yes|no|-|IteratorAggregate,Traversable,Countable\n",
            "SimpleXMLElement|no|no|no|yes|yes|-|Stringable,Countable,RecursiveIterator,Traversable,Iterator\n",
            "SimpleXMLIterator|no|no|no|yes|yes|SimpleXMLElement|Iterator,Traversable,RecursiveIterator,Countable,Stringable\n",
            "DOMException|no|no|yes|yes|no|Exception|Throwable,Stringable\n",
        ),
    );
}

/// Verifies reflection exposes PHP's parameter names, nullable/union returns, and readonly slots.
#[test]
fn dom_reflection_method_and_property_signatures_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function method_row(string $class, string $method): void {
    $reflection = new ReflectionMethod($class, $method);
    echo $class, "::", $method, "|";
    echo $reflection->isStatic() ? "static" : "instance";
    echo "|", ($reflection->hasReturnType() ? $reflection->getReturnType() : "-");
    echo "|", $reflection->getNumberOfRequiredParameters(), "/";
    echo $reflection->getNumberOfParameters(), "|";
    foreach ($reflection->getParameters() as $parameter) {
        echo $parameter->getName(), ":";
        echo $parameter->hasType() ? $parameter->getType() : "-";
        echo ":", ($parameter->isOptional() ? "optional" : "required"), ":";
        echo $parameter->isPassedByReference() ? "ref" : "value", ";";
    }
    echo "\n";
}

function property_row(string $class, string $property): void {
    $reflection = new ReflectionProperty($class, $property);
    echo $class, "::$", $property, "|", $reflection->getType(), "|";
    echo $reflection->isReadOnly() ? "readonly" : "mutable";
    echo "|", ($reflection->isPublic() ? "public" : "nonpublic"), "\n";
}

method_row("DOMDocument", "loadXML");
method_row("Dom\\XMLDocument", "createFromString");
method_row("DOMXPath", "query");
method_row("SimpleXMLElement", "__toString");
method_row("SimpleXMLElement", "current");
property_row("DOMDocument", "documentElement");
property_row("Dom\\NamespaceInfo", "prefix");
property_row("Dom\\NamespaceInfo", "element");
property_row("DOMNodeList", "length");
"#,
    );

    assert_eq!(
        output,
        concat!(
            "DOMDocument::loadXML|instance|-|1/2|source:string:required:value;options:int:optional:value;\n",
            "Dom\\XMLDocument::createFromString|static|Dom\\XMLDocument|1/3|source:string:required:value;options:int:optional:value;overrideEncoding:?string:optional:value;\n",
            "DOMXPath::query|instance|-|1/3|expression:string:required:value;contextNode:?DOMNode:optional:value;registerNodeNS:bool:optional:value;\n",
            "SimpleXMLElement::__toString|instance|string|0/0|\n",
            "SimpleXMLElement::current|instance|-|0/0|\n",
            "DOMDocument::$documentElement|?DOMElement|mutable|public\n",
            "Dom\\NamespaceInfo::$prefix|?string|readonly|public\n",
            "Dom\\NamespaceInfo::$element|Dom\\Element|readonly|public\n",
            "DOMNodeList::$length|int|mutable|public\n",
        ),
    );
}

/// Verifies DOM's backed enum and the extension's classes, interfaces, traits, and ancestry probes.
#[test]
fn dom_reflection_enum_and_extension_existence_probes_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
$enum = new ReflectionEnum("Dom\\AdjacentPosition");
echo $enum->getName(), "|", ($enum->isBacked() ? "backed" : "unit"), "|";
echo $enum->getBackingType(), "|";
echo implode(",", array_map(
    fn($case) => $case->getName() . "=" . $case->getBackingValue(),
    $enum->getCases(),
)), "\n";

echo "exists|", extension_loaded("dom") ? "yes" : "no";
echo "|", extension_loaded("libxml") ? "yes" : "no";
echo "|", extension_loaded("SimpleXML") ? "yes" : "no";
echo "|", class_exists("DOMDocument") ? "yes" : "no";
echo "|", class_exists("Dom\\XMLDocument") ? "yes" : "no";
echo "|", class_exists("SimpleXMLElement") ? "yes" : "no";
echo "|", interface_exists("DOMParentNode") ? "yes" : "no";
echo "|", interface_exists("Dom\\ParentNode") ? "yes" : "no";
echo "|", trait_exists("DOMNode") ? "yes" : "no";
echo "|", is_subclass_of("DOMDocument", "DOMNode") ? "yes" : "no";
echo "|", is_subclass_of("Dom\\XMLDocument", "Dom\\Document") ? "yes" : "no", "\n";
"#,
    );

    assert_eq!(
        output,
        concat!(
            "Dom\\AdjacentPosition|backed|string|BeforeBegin=beforebegin,AfterBegin=afterbegin,BeforeEnd=beforeend,AfterEnd=afterend\n",
            "exists|yes|yes|yes|yes|yes|yes|yes|yes|no|yes|yes\n",
        ),
    );
}

/// Verifies ReflectionEnum exposes DOM enum metadata, case reflectors, and catchable failures.
#[test]
fn dom_reflection_enum_metadata_and_errors_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function reflection_error_type(Closure $probe): string {
    try {
        $probe();
        return "none";
    } catch (Throwable $error) {
        return get_class($error);
    }
}

$enum = new ReflectionEnum("Dom\\AdjacentPosition");
echo $enum->getName(), "|", ($enum->isEnum() ? "enum" : "not-enum"), "|";
echo $enum->isBacked() ? "backed" : "unit";
echo "|", $enum->getBackingType()->getName(), "|";
echo $enum->isInternal() ? "internal" : "user";
echo "|", $enum->isFinal() ? "final" : "not-final", "|";
echo $enum->getModifiers(), "|", $enum->getExtensionName(), "|";
echo $enum->getExtension()->getName(), "|", implode(",", $enum->getInterfaceNames()), "\n";

foreach ($enum->getCases() as $case) {
    echo get_class($case), ":", $case->getName(), ":", $case->getBackingValue(), ":";
    echo get_class($case->getEnum()), ":";
    echo $case->isEnumCase() ? "case" : "not-case";
    echo ":", $case->isPublic() ? "public" : "not-public";
    echo ":", $case->isFinal() ? "final" : "not-final";
    echo ":", $case->getModifiers(), "\n";
}

echo reflection_error_type(fn() => new ReflectionEnum("Dom\\Document")), "\n";
"#,
    );

    assert_eq!(
        output,
        concat!(
            "Dom\\AdjacentPosition|enum|backed|string|internal|not-final|0|dom|dom|BackedEnum,UnitEnum\n",
            "ReflectionEnumBackedCase:BeforeBegin:beforebegin:ReflectionEnum:case:public:not-final:1\n",
            "ReflectionEnumBackedCase:AfterBegin:afterbegin:ReflectionEnum:case:public:not-final:1\n",
            "ReflectionEnumBackedCase:BeforeEnd:beforeend:ReflectionEnum:case:public:not-final:1\n",
            "ReflectionEnumBackedCase:AfterEnd:afterend:ReflectionEnum:case:public:not-final:1\n",
            "ReflectionException\n",
        ),
    );
}

/// Verifies every public DOM/libxml/SimpleXML function family retains names, types, and named arguments.
#[test]
fn dom_reflection_function_signatures_and_registration_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function function_row(string $name): void {
    $reflection = new ReflectionFunction($name);
    echo $name, "|";
    echo $reflection->hasReturnType() ? $reflection->getReturnType() : "-";
    echo "|", $reflection->getNumberOfRequiredParameters(), "/";
    echo $reflection->getNumberOfParameters(), "|";
    foreach ($reflection->getParameters() as $parameter) {
        echo $parameter->getName(), ":";
        echo $parameter->hasType() ? $parameter->getType() : "-";
        echo ":", ($parameter->isOptional() ? "optional" : "required"), ";";
    }
    echo "\n";
}

function_row("dom_import_simplexml");
function_row("Dom\\import_simplexml");
function_row("libxml_use_internal_errors");
function_row("libxml_set_external_entity_loader");
function_row("simplexml_load_string");
function_row("simplexml_import_dom");
echo "functions|", function_exists("dom_import_simplexml") ? "yes" : "no";
echo "|", function_exists("Dom\\import_simplexml") ? "yes" : "no";
echo "|", function_exists("libxml_get_errors") ? "yes" : "no";
echo "|", function_exists("simplexml_load_file") ? "yes" : "no", "\n";
"#,
    );

    assert_eq!(
        output,
        concat!(
            "dom_import_simplexml|DOMAttr|DOMElement|1/1|node:object:required;\n",
            "Dom\\import_simplexml|Dom\\Attr|Dom\\Element|1/1|node:object:required;\n",
            "libxml_use_internal_errors|bool|0/1|use_errors:?bool:optional;\n",
            "libxml_set_external_entity_loader|true|1/1|resolver_function:?callable:required;\n",
            "simplexml_load_string|SimpleXMLElement|false|1/5|data:string:required;class_name:?string:optional;options:int:optional;namespace_or_prefix:string:optional;is_prefix:bool:optional;\n",
            "simplexml_import_dom|?SimpleXMLElement|1/2|node:object:required;class_name:?string:optional;\n",
            "functions|yes|yes|yes|yes\n",
        ),
    );
}

/// Verifies every public extension registry reports the same DOM/libxml/SimpleXML surface.
///
/// The extension table is intentionally bounded to the DOM bridge's PHP-visible families.
#[test]
fn dom_extension_registry_visibility_matches_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function sorted_functions(string $extension): string {
    $functions = get_extension_funcs($extension);
    sort($functions);
    return implode(",", $functions);
}

$loaded = get_loaded_extensions();
echo "loaded|", extension_loaded("dom") ? "yes" : "no";
echo "|", extension_loaded("libxml") ? "yes" : "no";
echo "|", extension_loaded("simplexml") ? "yes" : "no";
echo "|", in_array("dom", $loaded, true) ? "dom" : "-";
echo "|", in_array("libxml", $loaded, true) ? "libxml" : "-";
echo "|", in_array("SimpleXML", $loaded, true) ? "SimpleXML" : "-", "\n";

foreach (["dom", "libxml", "SimpleXML"] as $extension) {
    echo "functions|", $extension, "|", sorted_functions($extension), "\n";
    $reflection = new ReflectionExtension($extension);
    $classes = $reflection->getClassNames();
    echo "extension|", $reflection->getName(), "|", count($classes), "|";
    echo in_array("DOMDocument", $classes, true) ? "DOMDocument" : "-", "|";
    echo in_array("Dom\\XMLDocument", $classes, true) ? "Dom\\XMLDocument" : "-", "|";
    echo in_array("LibXMLError", $classes, true) ? "LibXMLError" : "-", "|";
    echo in_array("SimpleXMLElement", $classes, true) ? "SimpleXMLElement" : "-", "\n";
}

echo "missing|", get_extension_funcs("not-an-extension") === false ? "false" : "bad", "\n";

foreach (["DOMDocument", "Dom\\XMLDocument", "LibXMLError", "SimpleXMLElement"] as $class) {
    $reflection = new ReflectionClass($class);
    echo "class|", $reflection->getName(), "|", $reflection->getExtensionName(), "|";
    echo $reflection->getExtension()->getName(), "\n";
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "loaded|yes|yes|yes|dom|libxml|SimpleXML\n",
            "functions|dom|Dom\\import_simplexml,dom_import_simplexml\n",
            "extension|dom|51|DOMDocument|Dom\\XMLDocument|-|-\n",
            "functions|libxml|libxml_clear_errors,libxml_disable_entity_loader,libxml_get_errors,libxml_get_external_entity_loader,libxml_get_last_error,libxml_set_external_entity_loader,libxml_set_streams_context,libxml_use_internal_errors\n",
            "extension|libxml|1|-|-|LibXMLError|-\n",
            "functions|SimpleXML|simplexml_import_dom,simplexml_load_file,simplexml_load_string\n",
            "extension|SimpleXML|2|-|-|-|SimpleXMLElement\n",
            "missing|false\n",
            "class|DOMDocument|dom|dom\n",
            "class|Dom\\XMLDocument|dom|dom\n",
            "class|LibXMLError|libxml|libxml\n",
            "class|SimpleXMLElement|SimpleXML|SimpleXML\n",
        ),
    );
}

/// Verifies `ReflectionExtension::getFunctions()` preserves PHP 8.5.8's ordered
/// associative registry of internal `ReflectionFunction` instances.
#[test]
fn reflection_extension_functions_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
foreach (["dom", "libxml", "SimpleXML"] as $extension) {
    $reflection = new ReflectionExtension($extension);
    $functions = $reflection->getFunctions();
    echo "extension|", $reflection->getName(), "|", count($functions), "\n";
    foreach ($functions as $key => $function) {
        echo "function|", $key, "|", get_class($function), "|";
        echo $function->getName(), "|", $function->getExtensionName(), "|";
        echo $function->isInternal() ? "internal" : "user", "|";
        echo $function->isUserDefined() ? "user" : "internal", "\n";
    }
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "extension|dom|2\n",
            "function|dom_import_simplexml|ReflectionFunction|dom_import_simplexml|dom|internal|internal\n",
            "function|Dom\\import_simplexml|ReflectionFunction|Dom\\import_simplexml|dom|internal|internal\n",
            "extension|libxml|8\n",
            "function|libxml_set_streams_context|ReflectionFunction|libxml_set_streams_context|libxml|internal|internal\n",
            "function|libxml_use_internal_errors|ReflectionFunction|libxml_use_internal_errors|libxml|internal|internal\n",
            "function|libxml_get_last_error|ReflectionFunction|libxml_get_last_error|libxml|internal|internal\n",
            "function|libxml_get_errors|ReflectionFunction|libxml_get_errors|libxml|internal|internal\n",
            "function|libxml_clear_errors|ReflectionFunction|libxml_clear_errors|libxml|internal|internal\n",
            "function|libxml_disable_entity_loader|ReflectionFunction|libxml_disable_entity_loader|libxml|internal|internal\n",
            "function|libxml_set_external_entity_loader|ReflectionFunction|libxml_set_external_entity_loader|libxml|internal|internal\n",
            "function|libxml_get_external_entity_loader|ReflectionFunction|libxml_get_external_entity_loader|libxml|internal|internal\n",
            "extension|SimpleXML|3\n",
            "function|simplexml_load_file|ReflectionFunction|simplexml_load_file|SimpleXML|internal|internal\n",
            "function|simplexml_load_string|ReflectionFunction|simplexml_load_string|SimpleXML|internal|internal\n",
            "function|simplexml_import_dom|ReflectionFunction|simplexml_import_dom|SimpleXML|internal|internal\n",
        ),
    );
}

/// Verifies `ReflectionExtension::getConstants()` preserves PHP 8.5.8's ordered,
/// typed DOM-family constant registries without inventing `getConstant()`.
#[test]
fn reflection_extension_constants_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function extension_constants(ReflectionExtension $reflection): string {
    $entries = [];
    foreach ($reflection->getConstants() as $name => $value) {
        $entries[] = $name . ":" . gettype($value) . ":" . $value;
    }
    return implode(",", $entries);
}

foreach (["dom", "libxml", "SimpleXML"] as $extension) {
    $reflection = new ReflectionExtension($extension);
    echo $reflection->getName(), "|", extension_constants($reflection), "|";
    echo method_exists($reflection, "getConstant") ? "getConstant" : "no-getConstant", "\n";
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "dom|XML_ELEMENT_NODE:integer:1,XML_ATTRIBUTE_NODE:integer:2,XML_TEXT_NODE:integer:3,XML_CDATA_SECTION_NODE:integer:4,XML_ENTITY_REF_NODE:integer:5,XML_ENTITY_NODE:integer:6,XML_PI_NODE:integer:7,XML_COMMENT_NODE:integer:8,XML_DOCUMENT_NODE:integer:9,XML_DOCUMENT_TYPE_NODE:integer:10,XML_DOCUMENT_FRAG_NODE:integer:11,XML_NOTATION_NODE:integer:12,XML_HTML_DOCUMENT_NODE:integer:13,XML_DTD_NODE:integer:14,XML_ELEMENT_DECL_NODE:integer:15,XML_ATTRIBUTE_DECL_NODE:integer:16,XML_ENTITY_DECL_NODE:integer:17,XML_NAMESPACE_DECL_NODE:integer:18,XML_LOCAL_NAMESPACE:integer:18,XML_ATTRIBUTE_CDATA:integer:1,XML_ATTRIBUTE_ID:integer:2,XML_ATTRIBUTE_IDREF:integer:3,XML_ATTRIBUTE_IDREFS:integer:4,XML_ATTRIBUTE_ENTITY:integer:6,XML_ATTRIBUTE_NMTOKEN:integer:7,XML_ATTRIBUTE_NMTOKENS:integer:8,XML_ATTRIBUTE_ENUMERATION:integer:9,XML_ATTRIBUTE_NOTATION:integer:10,DOM_PHP_ERR:integer:0,DOM_INDEX_SIZE_ERR:integer:1,DOMSTRING_SIZE_ERR:integer:2,DOM_HIERARCHY_REQUEST_ERR:integer:3,DOM_WRONG_DOCUMENT_ERR:integer:4,DOM_INVALID_CHARACTER_ERR:integer:5,DOM_NO_DATA_ALLOWED_ERR:integer:6,DOM_NO_MODIFICATION_ALLOWED_ERR:integer:7,DOM_NOT_FOUND_ERR:integer:8,DOM_NOT_SUPPORTED_ERR:integer:9,DOM_INUSE_ATTRIBUTE_ERR:integer:10,DOM_INVALID_STATE_ERR:integer:11,DOM_SYNTAX_ERR:integer:12,DOM_INVALID_MODIFICATION_ERR:integer:13,DOM_NAMESPACE_ERR:integer:14,DOM_INVALID_ACCESS_ERR:integer:15,DOM_VALIDATION_ERR:integer:16,Dom\\INDEX_SIZE_ERR:integer:1,Dom\\STRING_SIZE_ERR:integer:2,Dom\\HIERARCHY_REQUEST_ERR:integer:3,Dom\\WRONG_DOCUMENT_ERR:integer:4,Dom\\INVALID_CHARACTER_ERR:integer:5,Dom\\NO_DATA_ALLOWED_ERR:integer:6,Dom\\NO_MODIFICATION_ALLOWED_ERR:integer:7,Dom\\NOT_FOUND_ERR:integer:8,Dom\\NOT_SUPPORTED_ERR:integer:9,Dom\\INUSE_ATTRIBUTE_ERR:integer:10,Dom\\INVALID_STATE_ERR:integer:11,Dom\\SYNTAX_ERR:integer:12,Dom\\INVALID_MODIFICATION_ERR:integer:13,Dom\\NAMESPACE_ERR:integer:14,Dom\\VALIDATION_ERR:integer:16,Dom\\HTML_NO_DEFAULT_NS:integer:2147483648|no-getConstant\n",
            "libxml|LIBXML_VERSION:integer:21503,LIBXML_DOTTED_VERSION:string:2.15.3,LIBXML_LOADED_VERSION:string:21503,LIBXML_RECOVER:integer:1,LIBXML_NOENT:integer:2,LIBXML_NO_XXE:integer:8388608,LIBXML_DTDLOAD:integer:4,LIBXML_DTDATTR:integer:8,LIBXML_DTDVALID:integer:16,LIBXML_NOERROR:integer:32,LIBXML_NOWARNING:integer:64,LIBXML_NOBLANKS:integer:256,LIBXML_XINCLUDE:integer:1024,LIBXML_NSCLEAN:integer:8192,LIBXML_NOCDATA:integer:16384,LIBXML_NONET:integer:2048,LIBXML_PEDANTIC:integer:128,LIBXML_COMPACT:integer:65536,LIBXML_NOXMLDECL:integer:2,LIBXML_PARSEHUGE:integer:524288,LIBXML_BIGLINES:integer:4194304,LIBXML_NOEMPTYTAG:integer:4,LIBXML_SCHEMA_CREATE:integer:1,LIBXML_HTML_NOIMPLIED:integer:8192,LIBXML_HTML_NODEFDTD:integer:4,LIBXML_ERR_NONE:integer:0,LIBXML_ERR_WARNING:integer:1,LIBXML_ERR_ERROR:integer:2,LIBXML_ERR_FATAL:integer:3|no-getConstant\n",
            "SimpleXML||no-getConstant\n",
        ),
    );
}

/// Verifies `ReflectionExtension::getINIEntries()` exposes PHP 8.5.8's empty,
/// ordered INI map for the bounded DOM-family extensions and retains constructor errors.
#[test]
fn reflection_extension_ini_entries_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
foreach (["dom", "libxml", "SimpleXML"] as $extension) {
    $reflection = new ReflectionExtension($extension);
    $entries = $reflection->getINIEntries();
    echo "extension|", $reflection->getName(), "|";
    echo method_exists($reflection, "getINIEntries") ? "method" : "no-method", "|";
    echo gettype($entries), "|", count($entries), "|";
    foreach ($entries as $name => $value) {
        echo gettype($name), ":", $name, "=", gettype($value), ":", $value, ";";
    }
    echo "\n";
}

try {
    (new ReflectionExtension("not-an-extension"))->getINIEntries();
    echo "missing|none\n";
} catch (ReflectionException $error) {
    echo "missing|", get_class($error), "|", $error->getMessage(), "\n";
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "extension|dom|method|array|0|\n",
            "extension|libxml|method|array|0|\n",
            "extension|SimpleXML|method|array|0|\n",
            "missing|ReflectionException|Extension \"not-an-extension\" does not exist\n",
        ),
    );
}

/// Verifies `ReflectionExtension::getDependencies()` preserves PHP 8.5.8's ordered,
/// string-keyed DOM-family dependency maps and unknown-extension constructor error.
#[test]
fn reflection_extension_dependencies_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
foreach (["dom", "libxml", "SimpleXML"] as $extension) {
    $reflection = new ReflectionExtension($extension);
    $dependencies = $reflection->getDependencies();
    echo "extension|", $reflection->getName(), "|";
    echo method_exists($reflection, "getDependencies") ? "method" : "no-method", "|";
    echo gettype($dependencies), "|", count($dependencies), "|";
    foreach ($dependencies as $name => $kind) {
        echo gettype($name), ":", $name, "=", gettype($kind), ":", $kind, ";";
    }
    echo "\n";
}

try {
    (new ReflectionExtension("not-an-extension"))->getDependencies();
    echo "missing|none\n";
} catch (ReflectionException $error) {
    echo "missing|", get_class($error), "|", $error->getCode(), "|", $error->getMessage(), "\n";
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "extension|dom|method|array|3|string:libxml=string:Required;string:lexbor=string:Required;string:domxml=string:Conflicts;\n",
            "extension|libxml|method|array|1|string:standard=string:Required;\n",
            "extension|SimpleXML|method|array|2|string:libxml=string:Required;string:spl=string:Required;\n",
            "missing|ReflectionException|0|Extension \"not-an-extension\" does not exist\n",
        ),
    );
}

/// Verifies `ReflectionExtension`'s DOM-family metadata predicates and versions retain the
/// PHP 8.5.8 value and scalar-type contracts without inventing `isInternal()`.
#[test]
fn reflection_extension_metadata_methods_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
foreach (["dom", "libxml", "SimpleXML"] as $extension) {
    $reflection = new ReflectionExtension($extension);
    echo $reflection->getName(), "|";
    echo gettype($reflection->isPersistent()), ":", $reflection->isPersistent() ? "true" : "false", "|";
    echo gettype($reflection->isTemporary()), ":", $reflection->isTemporary() ? "true" : "false", "|";
    echo gettype($reflection->getVersion()), ":", $reflection->getVersion(), "|";
    echo method_exists($reflection, "isInternal") ? "isInternal" : "no-isInternal", "\n";
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "dom|boolean:true|boolean:false|string:20031129|no-isInternal\n",
            "libxml|boolean:true|boolean:false|string:8.5.8|no-isInternal\n",
            "SimpleXML|boolean:true|boolean:false|string:8.5.8|no-isInternal\n",
        ),
    );
}

/// Verifies `ReflectionExtension` canonicalizes known names and reports literal and runtime
/// unknown extensions through PHP's catchable, name-bearing `ReflectionException`.
#[test]
fn reflection_extension_unknown_name_matches_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
$dom = new ReflectionExtension("DOM");
echo "known|DOM|", get_class($dom), "|", $dom->getName(), "\n";

$libxml = new ReflectionExtension("LiBxMl");
echo "known|LiBxMl|", get_class($libxml), "|", $libxml->getName(), "\n";

$simplexml = new ReflectionExtension("simplexml");
echo "known|simplexml|", get_class($simplexml), "|", $simplexml->getName(), "\n";

try {
    new ReflectionExtension("not-an-extension");
    echo "missing|none\n";
} catch (ReflectionException $error) {
    echo "missing|", get_class($error), "|", $error->getCode(), "|";
    echo $error->getMessage(), "|";
    echo $error instanceof Exception ? "Exception" : "no", "|";
    echo $error instanceof Throwable ? "Throwable" : "no", "\n";
}

$runtimeUnknown = $argc === 1 ? "dynamic-unknown" : "not-reached";
try {
    new ReflectionExtension($runtimeUnknown);
    echo "runtime-missing|none\n";
} catch (ReflectionException $error) {
    echo "runtime-missing|", get_class($error), "|", $error->getCode(), "|";
    echo $error->getMessage(), "|";
    echo $error instanceof Exception ? "Exception" : "no", "|";
    echo $error instanceof Throwable ? "Throwable" : "no", "\n";
}
"#,
    );

    assert_eq!(
        output,
        concat!(
            "known|DOM|ReflectionExtension|dom\n",
            "known|LiBxMl|ReflectionExtension|libxml\n",
            "known|simplexml|ReflectionExtension|SimpleXML\n",
            "missing|ReflectionException|0|Extension \"not-an-extension\" does not exist|Exception|Throwable\n",
            "runtime-missing|ReflectionException|0|Extension \"dynamic-unknown\" does not exist|Exception|Throwable\n",
        ),
    );
}

/// Verifies `get_extension_funcs()` resolves runtime strings and non-strict scalar coercion
/// through PHP's case-insensitive DOM bridge registry while preserving unknown `false`.
#[test]
fn dynamic_dom_extension_function_registry_matches_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function extension_functions(string $extension): string {
    $functions = get_extension_funcs($extension);
    if ($functions === false) {
        return "false";
    }
    sort($functions);
    return implode(",", $functions);
}

foreach (["DOM", "LiBxMl", "simpleXML", "not-an-extension"] as $extension) {
    echo $extension, "|", extension_functions($extension), "\n";
}
echo "int|", get_extension_funcs(42) === false ? "false" : "bad", "\n";
"#,
    );

    assert_eq!(
        output,
        concat!(
            "DOM|Dom\\import_simplexml,dom_import_simplexml\n",
            "LiBxMl|libxml_clear_errors,libxml_disable_entity_loader,libxml_get_errors,libxml_get_external_entity_loader,libxml_get_last_error,libxml_set_external_entity_loader,libxml_set_streams_context,libxml_use_internal_errors\n",
            "simpleXML|simplexml_import_dom,simplexml_load_file,simplexml_load_string\n",
            "not-an-extension|false\n",
            "int|false\n",
        ),
    );
}

/// Verifies arity, unknown named parameters, and type errors remain catchable PHP runtime exceptions.
#[test]
fn dom_public_surface_argument_errors_match_php_8_5_8() {
    let output = compile_and_run(
        r#"<?php
function probe(string $id, Closure $call): void {
    try {
        $call();
        echo $id, "|none\n";
    } catch (Throwable $error) {
        echo $id, "|", get_class($error), "|", $error->getMessage(), "\n";
    }
}

$legacy = new DOMDocument();
probe("legacy-arity", fn() => $legacy->loadXML());
probe("legacy-named", fn() => $legacy->loadXML(source: "<r/>", unexpected: 1));
probe("legacy-type", fn() => $legacy->loadXML([]));
probe("xpath-arity", fn() => new DOMXPath());
probe("xpath-named", fn() => new DOMXPath(document: $legacy, unexpected: true));
probe("xpath-type", fn() => new DOMXPath(new stdClass()));
probe("modern-factory-arity", fn() => Dom\\XMLDocument::createFromString());
probe("modern-factory-type", fn() => Dom\\XMLDocument::createFromString([]));
probe("simplexml-arity", fn() => simplexml_load_string());
probe("simplexml-type", fn() => simplexml_load_string([]));
probe("libxml-arity", fn() => libxml_set_streams_context());
probe("libxml-type", fn() => libxml_use_internal_errors([]));
"#,
    );

    assert_eq!(
        output,
        concat!(
            "legacy-arity|ArgumentCountError|DOMDocument::loadXML() expects at least 1 argument, 0 given\n",
            "legacy-named|Error|Unknown named parameter $unexpected\n",
            "legacy-type|TypeError|DOMDocument::loadXML(): Argument #1 ($source) must be of type string, array given\n",
            "xpath-arity|ArgumentCountError|DOMXPath::__construct() expects at least 1 argument, 0 given\n",
            "xpath-named|Error|Unknown named parameter $unexpected\n",
            "xpath-type|TypeError|DOMXPath::__construct(): Argument #1 ($document) must be of type DOMDocument, stdClass given\n",
            "modern-factory-arity|ArgumentCountError|Dom\\XMLDocument::createFromString() expects at least 1 argument, 0 given\n",
            "modern-factory-type|TypeError|Dom\\XMLDocument::createFromString(): Argument #1 ($source) must be of type string, array given\n",
            "simplexml-arity|ArgumentCountError|simplexml_load_string() expects at least 1 argument, 0 given\n",
            "simplexml-type|TypeError|simplexml_load_string(): Argument #1 ($data) must be of type string, array given\n",
            "libxml-arity|ArgumentCountError|libxml_set_streams_context() expects exactly 1 argument, 0 given\n",
            "libxml-type|TypeError|libxml_use_internal_errors(): Argument #1 ($use_errors) must be of type ?bool, array given\n",
        ),
    );
}
