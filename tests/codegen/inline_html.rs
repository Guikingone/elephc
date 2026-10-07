//! Purpose:
//! End-to-end tests for PHP's close tag `?>` and inline HTML in AOT `.php` source.
//!
//! Called from:
//! - `cargo test --test codegen_tests inline_html` through the integration suite.
//!
//! Key details:
//! - Every expectation is byte-checked against reference PHP 8.5.10: a `?>` closes PHP mode, the
//!   single newline right after it is swallowed, the HTML is echoed, and `<?php`/`<?=` reopen code.
//! - The AOT lexer used to require `<?php` at the start and had no close-tag token, so any file
//!   with a `?>` was a parse error; inline HTML was handled only inside `eval()`.

use crate::support::*;

/// Verifies a trailing `?>` is accepted and echoes nothing.
#[test]
fn trailing_close_tag_is_accepted() {
    assert_eq!(compile_and_run("<?php echo 1; ?>"), "1");
}

/// Verifies inline HTML after `?>` is echoed verbatim.
#[test]
fn inline_html_after_close_tag_is_echoed() {
    assert_eq!(compile_and_run("<?php echo 1; ?>text"), "1text");
}

/// Verifies exactly ONE newline immediately after `?>` is swallowed, matching Zend.
#[test]
fn one_newline_after_close_tag_is_swallowed() {
    assert_eq!(compile_and_run("<?php echo 1; ?>\n\ntext"), "1\ntext");
}

/// Verifies a CRLF immediately after `?>` is swallowed as one newline.
#[test]
fn crlf_after_close_tag_is_swallowed() {
    assert_eq!(compile_and_run("<?php echo 1; ?>\r\ntext"), "1text");
}

/// Verifies leading inline HTML before the first open tag is echoed.
#[test]
fn leading_inline_html_is_echoed() {
    assert_eq!(compile_and_run("lead<?php echo 1; ?>"), "lead1");
}

/// Verifies `<?=` is the short echo tag.
#[test]
fn short_echo_tag_echoes() {
    assert_eq!(compile_and_run("<?= 1 ?>"), "1");
}

/// Verifies a control-flow block can span the tags.
#[test]
fn a_block_can_span_the_tags() {
    assert_eq!(compile_and_run("<?php if (true) { ?>IN<?php } ?>"), "IN");
}

/// Verifies an empty statement (`;`) is valid on its own — it is what a close tag leaves behind.
#[test]
fn an_empty_statement_is_valid() {
    assert_eq!(compile_and_run("<?php ; echo 1; ?>"), "1");
}

/// Verifies several PHP/HTML blocks interleave in source order.
#[test]
fn several_blocks_interleave_in_order() {
    assert_eq!(compile_and_run("A<?php echo 1; ?>B<?= 2 ?>C"), "A1B2C");
}

/// Verifies a pure-HTML file (no `<?php` at all) echoes its whole contents.
#[test]
fn a_pure_html_file_is_echoed() {
    assert_eq!(compile_and_run("just text\nno php here"), "just text\nno php here");
}
