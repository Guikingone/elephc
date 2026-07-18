//! Purpose:
//! Defines `ext/dom`/`ext/xml` node-kind integer constants (`XML_*`) exposed by
//! elephc. Only the subset demanded by real-world PHP source (currently
//! `XML_DOCUMENT_TYPE_NODE`) is registered; the full DOM node-kind space is
//! intentionally out of scope.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - DOM node-kind values mirror the W3C DOM `Node.nodeType` numbering, which
//!   PHP exposes verbatim. Verified with
//!   `php -n -r 'var_dump(XML_DOCUMENT_TYPE_NODE);'` (PHP 8.5.6 local).

/// Tuple of `(name, value)` pairs for the demanded `ext/dom` `XML_*` node-kind constants.
pub(crate) const XML_INT_CONSTANTS: &[(&str, i64)] = &[("XML_DOCUMENT_TYPE_NODE", 10)];

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
