//! Purpose:
//! Defines demanded `ext/dom`/`ext/xml`/libxml integer constants exposed by elephc.
//! Covers the DOM document-type node plus parser/security flags used by Symfony; the
//! complete XML/libxml constant space is intentionally out of scope.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - DOM node-kind values mirror the W3C DOM `Node.nodeType` numbering, which
//!   PHP exposes verbatim. Verified with
//!   `php -n -r 'var_dump(XML_DOCUMENT_TYPE_NODE);'` (PHP 8.5.6 local).

/// Tuple of `(name, value)` pairs for the demanded DOM/XML/libxml constants.
pub(crate) const XML_INT_CONSTANTS: &[(&str, i64)] = &[
    ("XML_DOCUMENT_TYPE_NODE", 10),
    ("LIBXML_ERR_WARNING", 1),
    ("LIBXML_COMPACT", 65536),
    ("LIBXML_NONET", 2048),
];

#[cfg(test)]
mod tests {
    use super::XML_INT_CONSTANTS;

    /// Verifies `XML_DOCUMENT_TYPE_NODE` matches PHP's DOM node-kind numbering.
    #[test]
    fn test_xml_document_type_node_matches_php() {
        let value = XML_INT_CONSTANTS
            .iter()
            .find(|(n, _)| *n == "XML_DOCUMENT_TYPE_NODE")
            .expect("XML_DOCUMENT_TYPE_NODE defined")
            .1;
        assert_eq!(value, 10);
    }

    /// Verifies the demanded libxml error/security flags match PHP 8.5.6.
    #[test]
    fn test_libxml_flags_match_php() {
        let value_of = |name: &str| {
            XML_INT_CONSTANTS
                .iter()
                .find(|(constant_name, _)| *constant_name == name)
                .unwrap_or_else(|| panic!("{name} defined"))
                .1
        };
        assert_eq!(value_of("LIBXML_ERR_WARNING"), 1);
        assert_eq!(value_of("LIBXML_COMPACT"), 65536);
        assert_eq!(value_of("LIBXML_NONET"), 2048);
    }

    /// Asserts no duplicate names exist in `XML_INT_CONSTANTS`.
    #[test]
    fn test_xml_constants_have_unique_names() {
        let mut names: Vec<&str> = XML_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate xml constant name");
    }
}
