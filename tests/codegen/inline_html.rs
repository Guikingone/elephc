//! Purpose:
//! End-to-end tests for PHP's tag handling: inline HTML around `<?php ... ?>` blocks, `<?=`
//! short echo tags, `?>` as a statement terminator, and the alternative control syntax
//! (`if (): ... endif;`, `endwhile`, `endfor`, `endforeach`, `endswitch`) used in templates.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was produced by the PHP interpreter.

use crate::support::*;

/// Text around `<?php ... ?>` blocks is echoed verbatim, `?>` ends a statement and swallows one newline, line comments stop at `?>`, `<?=` echoes, and alternative-syntax loops/ifs and function bodies interleave with inline HTML.
#[test]
fn test_closing_tags_inline_html_and_short_echo() {
    let out = compile_and_run(r#"head line
<?php
$x = 3; // comment ends at the tag ?> mid <?= $x ?>
<?php if ($x > 1) { ?>yes <?php echo $x + 1; } ?>
tail
<?PHP
# hash comment ?>after-hash
<?php for ($i = 0; $i < 2; $i++): ?>[<?= $i ?>]
<?php endfor; ?>
<?php
function f() { ?>in-f<?php }
f();
$g = function () { return "cl"; } ?>
<?= $g(), "\n" ?>
<?php
echo "end\n";
?>
"#);
    assert_eq!(out, "head line\n mid 3yes 4tail\nafter-hash\n[0]\n[1]\nin-fcl\nend\n");
}

/// `if/elseif/else/endif`, `while/endwhile`, `for/endfor`, `foreach/endforeach` and `switch/endswitch` (including `case X;`) build the same statements as the brace forms.
#[test]
fn test_alternative_control_syntax() {
    let out = compile_and_run(r#"<?php
for ($i=0;$i<2;$i++): echo $i; endfor;
foreach ([1,2] as $k => $v): echo $k, $v; endforeach;
foreach ([3,4] as $v): echo $v; endforeach;
$n = 2;
if ($n == 1): echo "a"; elseif ($n == 2): echo "b"; else: echo "c"; endif;
if ($n == 3): echo "x"; else: echo "y"; ENDIF;
$i=0; while ($i<2): echo $i++; endwhile;
switch ($n): case 1: echo "s1"; break; case 2; echo "s2"; default: echo "d"; endswitch;
switch ($n):
  case 5: echo "no";
endswitch;
if ($n): if ($n > 1): echo "nested"; endif; endif;
echo "\n";
"#);
    assert_eq!(out, "01011234by01s2dnested\n");
}

/// A `?>` swallows a following CRLF; inline HTML at the end of the file without a final open tag is still printed.
#[test]
fn test_closing_tag_crlf_and_trailing_html() {
    let out = compile_and_run(r#"<?php
echo "a";
?>
b<?php echo "c";?>"#);
    assert_eq!(out, "abc");
}

/// `<?php` and `?>` inside a string literal are plain text.
#[test]
fn test_tag_like_text_inside_string_is_not_a_tag() {
    let out = compile_and_run(r#"<?php echo "<?php not a tag ?>", "\n";"#);
    assert_eq!(out, "<?php not a tag ?>\n");
}
