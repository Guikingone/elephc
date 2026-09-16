//! Purpose:
//! Regression tests for issue #692: `inet_pton()` and `inet_ntop()` over IPv6, the family PHP
//! supports and elephc answered `false` for.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - The parsing and rendering are the platform's own `inet_pton(3)`/`inet_ntop(3)`, so these
//!   fixtures are about the shapes PHP itself defines: `::` compression, the embedded-IPv4
//!   form, zone identifiers, and the length that decides the family on the way back.
//! - Every expectation is verbatim host PHP 8.5.10, including the canonical spelling
//!   `inet_ntop()` chooses when several are possible (`2001:db8:85a3:0:0:8a2e:370:7334` renders
//!   as `2001:db8:85a3::8a2e:370:7334`).
//! - The packed bytes land in the shared concat buffer, so several live at once and a call in
//!   the middle of a concatenation are both exercised: a mistracked cursor shows up as one
//!   address overwriting another.

use crate::support::*;

/// Verifies `inet_pton()` packs IPv6 addresses, in every spelling PHP accepts.
///
/// The whole family answered `false` before: the helper was a dotted-quad IPv4 parser, so a
/// program validating a proxy-provided client address rejected every IPv6 one.
#[test]
fn test_inet_pton_packs_ipv6() {
    let out = compile_and_run(
        r#"<?php
foreach ([
    '2001:4860:4860::8888',
    '::1',
    '::',
    '2001:db8:85a3:0:0:8a2e:370:7334',
    '2001:0db8:85a3:0000:0000:8a2e:0370:7334',
    '::ffff:192.0.2.128',
    'fe80::1%eth0',
] as $address) {
    $packed = inet_pton($address);
    echo $packed === false ? "false" : strlen($packed) . ":" . bin2hex($packed), "\n";
}
"#,
    );
    assert_eq!(
        out,
        concat!(
            "16:20014860486000000000000000008888\n",
            "16:00000000000000000000000000000001\n",
            "16:00000000000000000000000000000000\n",
            "16:20010db885a3000000008a2e03707334\n",
            "16:20010db885a3000000008a2e03707334\n",
            "16:00000000000000000000ffffc0000280\n",
            "16:fe800000000000000000000000000001\n",
        )
    );
}

/// Verifies the IPv4 family still packs the way it did, and that invalid input of either
/// family is still `false`.
///
/// The IPv4 path is what the helper used to be, so it is the one a rewrite can lose.
#[test]
fn test_inet_pton_keeps_ipv4_and_still_rejects_invalid_input() {
    let out = compile_and_run(
        r#"<?php
foreach ([
    '127.0.0.1',
    '0.0.0.0',
    '255.255.255.255',
    '256.0.0.1',
    '1.2.3',
    '',
    '1:2:3:4:5:6:7:8:9',
    'gggg::1',
    'not an address',
] as $address) {
    $packed = inet_pton($address);
    echo $packed === false ? "false" : strlen($packed) . ":" . bin2hex($packed), "\n";
}
"#,
    );
    assert_eq!(
        out,
        concat!(
            "4:7f000001\n",
            "4:00000000\n",
            "4:ffffffff\n",
            "false\n",
            "false\n",
            "false\n",
            "false\n",
            "false\n",
            "false\n",
        )
    );
}

/// Verifies `inet_ntop()` renders a 16-byte address, choosing PHP's canonical spelling.
///
/// Its own IPv6 half was missing too, which left a packed IPv6 address unusable: the pair is
/// how a program stores and reads one back.
#[test]
fn test_inet_ntop_renders_ipv6_round_trips() {
    let out = compile_and_run(
        r#"<?php
foreach ([
    '8.8.8.8',
    '2001:db8::1',
    '::ffff:10.0.0.1',
    'fe80::200:5aee:feaa:20a2',
    '2001:db8:85a3:0:0:8a2e:370:7334',
    '::1',
    '::',
] as $address) {
    echo inet_ntop(inet_pton($address)), "\n";
}
var_dump(inet_ntop("xx"));
var_dump(inet_ntop("123456789012345"));
"#,
    );
    assert_eq!(
        out,
        concat!(
            "8.8.8.8\n",
            "2001:db8::1\n",
            "::ffff:10.0.0.1\n",
            "fe80::200:5aee:feaa:20a2\n",
            "2001:db8:85a3::8a2e:370:7334\n",
            "::1\n",
            "::\n",
            "bool(false)\n",
            "bool(false)\n",
        )
    );
}

/// Verifies several packed addresses stay intact while others are produced.
///
/// Both helpers write into the shared concat buffer and publish its cursor, so an address that
/// forgets to reserve its own bytes is overwritten by the next one — visible only when more
/// than one is alive, or when a call sits inside a concatenation that is also writing there.
#[test]
fn test_inet_helpers_share_the_concat_buffer_without_clobbering() {
    let out = compile_and_run(
        r#"<?php
$a = inet_pton('2001:4860:4860::8888');
$b = inet_pton('127.0.0.1');
$c = inet_pton('::1');
echo bin2hex($a), "|", bin2hex($b), "|", bin2hex($c), "\n";
echo "x" . bin2hex(inet_pton('::')) . "y" . bin2hex(inet_pton('1.2.3.4')) . "z\n";
echo "[" . inet_ntop(inet_pton('2001:db8::dead:beef')) . "]\n";
$n = 0;
for ($i = 0; $i < 50; $i++) {
    $n += strlen(inet_pton('2001:db8::1'));
    $n += strlen(inet_ntop(inet_pton('2001:db8::1')));
}
echo $n;
"#,
    );
    assert_eq!(
        out,
        concat!(
            "20014860486000000000000000008888|7f000001|00000000000000000000000000000001\n",
            "x00000000000000000000000000000000y01020304z\n",
            "[2001:db8::dead:beef]\n",
            "1350",
        )
    );
}

/// Verifies a zone identifier is accepted up to the helper's documented input bound.
///
/// `inet_pton(3)` ignores everything past the `%`, so both platforms take a long one; the copy
/// into the NUL-terminated buffer the C parser needs is what bounds it here. 255 bytes is an
/// order of magnitude past `IF_NAMESIZE`, so the bound is reachable only by input that is not
/// an address — but the boundary is pinned, because a shorter buffer would silently start
/// refusing real addresses.
#[test]
fn test_inet_pton_accepts_a_zone_identifier_up_to_the_input_bound() {
    let out = compile_and_run(
        r#"<?php
$at_bound = '2001:db8::1%' . str_repeat('a', 243);
$past_bound = '2001:db8::1%' . str_repeat('a', 244);
echo strlen($at_bound), ":", inet_pton($at_bound) === false ? "false" : "ok", "\n";
echo strlen($past_bound), ":", inet_pton($past_bound) === false ? "false" : "ok", "\n";
echo inet_pton('fe80::1%eth0') === false ? "false" : bin2hex(inet_pton('fe80::1%eth0')), "\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "255:ok\n",
            "256:false\n",
            "fe800000000000000000000000000001\n",
        )
    );
}
