//! Purpose:
//! Integration tests for nested packed class fields: a packed field whose type is another packed
//! class is an embedded record reached through its compile-time offset.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Covers reads, writes and compound assignments through buffer elements and `ptr<T>` typed
//!   pointers, three levels of nesting, layout offsets and stride, whole-record copies, and the
//!   null-pointer guard on typed pointer receivers.

use crate::support::*;

/// Verifies nested fields are written and read through buffer elements without touching the
/// sibling record or the other element.
#[test]
fn test_nested_packed_fields_read_and_write_through_buffer_elements() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public float $x; public float $y; }
packed class Body { public Vec2 $pos; public Vec2 $vel; }
$b = buffer_new<Body>(2);
$b[0]->pos->x = 1.25;
$b[0]->pos->y = 2.5;
$b[1]->vel->y = 7.5;
echo $b[0]->pos->x, " ", $b[0]->pos->y, " ", $b[1]->vel->y, "\n";
echo $b[0]->vel->x, " ", $b[0]->vel->y, " ", $b[1]->pos->x, " ", $b[1]->pos->y, "\n";
"#,
    );
    assert_eq!(out, "1.25 2.5 7.5\n0 0 0 0\n");
}

/// Verifies compound assignments, increments and decrements on nested fields, including an int
/// field fed by a counter the checker types as `int|float`.
#[test]
fn test_nested_packed_fields_support_compound_assignment() {
    let out = compile_and_run(
        r#"<?php
packed class Counter { public int $hits; public float $total; }
packed class Stats { public Counter $a; public Counter $b; }
$s = buffer_new<Stats>(1);
$s[0]->a->hits = 4;
$s[0]->a->hits += 3;
$s[0]->a->hits++;
$s[0]->b->hits--;
$s[0]->a->total = 1.5;
$s[0]->a->total *= 4.0;
$s[0]->b->total -= 0.25;
$n = 0;
$n++;
$s[0]->b->hits = $n;
echo $s[0]->a->hits, " ", $s[0]->b->hits, " ", $s[0]->a->total, " ", $s[0]->b->total, "\n";
"#,
    );
    assert_eq!(out, "8 1 6 -0.25\n");
}

/// Verifies three levels of nesting resolve the summed offsets for reads and writes, and that a
/// local holding a nested record aliases the buffer storage like a local holding an element.
#[test]
fn test_nested_packed_fields_resolve_three_levels() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public float $x; public float $y; }
packed class Body { public Vec2 $pos; public Vec2 $vel; public int $id; }
packed class World { public int $tick; public Body $player; }
$w = buffer_new<World>(2);
$w[1]->tick = 11;
$w[1]->player->id = 4;
$w[1]->player->vel->x = 3.5;
$w[1]->player->pos->y = -2.0;
$player = $w[1]->player;
$player->vel->y = 9.0;
echo $w[1]->tick, " ", $w[1]->player->id, " ", $w[1]->player->vel->x, " ", $w[1]->player->pos->y, " ", $w[1]->player->vel->y, "\n";
echo $w[0]->tick, " ", $w[0]->player->id, " ", $w[0]->player->vel->x, "\n";
"#,
    );
    assert_eq!(out, "11 4 3.5 -2 9\n0 0 0\n");
}

/// Verifies the embedded layout: an inner record occupies its own size inside the outer one, the
/// outer size is the sum of its fields, and a buffer of outer records strides by that size.
#[test]
fn test_nested_packed_layout_offsets_and_stride() {
    let out = compile_and_run(
        r#"<?php
extern function calloc(u64 $count, u64 $size): ptr;
extern function free(ptr $p): void;
packed class Vec2 { public float $x; public float $y; }
packed class Body { public Vec2 $pos; public Vec2 $vel; }
echo ptr_sizeof("Vec2"), " ", ptr_sizeof("Body"), "\n";
$raw = calloc(2, ptr_sizeof("Body"));
$second = ptr_cast<Body>(ptr_offset($raw, ptr_sizeof("Body")));
$second->pos->x = 1.0;
$second->pos->y = 2.0;
$second->vel->x = 3.0;
$second->vel->y = 4.0;
echo ptr_read_f64(ptr_offset($raw, 32)), " ", ptr_read_f64(ptr_offset($raw, 40)), " ";
echo ptr_read_f64(ptr_offset($raw, 48)), " ", ptr_read_f64(ptr_offset($raw, 56)), " ";
echo ptr_read_f64($raw), "\n";
free($raw);
$b = buffer_new<Body>(3);
$b[1]->vel->y = 5.0;
echo $b[0]->vel->y, " ", $b[1]->vel->y, " ", $b[2]->pos->x, "\n";
"#,
    );
    assert_eq!(out, "16 32\n1 2 3 4 0\n0 5 0\n");
}

/// Verifies nested and scalar fields through `ptr<T>` typed pointers, including function
/// parameters typed as `ptr<T>` and `buffer<T>`, and an int field fed by an `int|float` counter.
#[test]
fn test_nested_packed_fields_through_typed_pointers() {
    let out = compile_and_run(
        r#"<?php
extern function calloc(u64 $count, u64 $size): ptr;
extern function free(ptr $p): void;
packed class Vec2 { public float $x; public float $y; }
packed class Body { public Vec2 $pos; public Vec2 $vel; public int $id; }
function tag(ptr<Body> $p, int $id): void {
    $p->id = $id;
    $p->vel->x = 0.5;
}
function step(buffer<Body> $b, int $n, float $dt): void {
    for ($i = 0; $i < $n; $i++) {
        $b[$i]->pos->x += $b[$i]->vel->x * $dt;
    }
}
$raw = calloc(1, ptr_sizeof("Body"));
$p = ptr_cast<Body>($raw);
tag($p, 12);
$p->pos->y = 1.5;
$p->pos->y += 1.0;
echo $p->id, " ", $p->vel->x, " ", $p->pos->y, "\n";
$n = 0;
$n++;
$p->id = $n;
echo $p->id, "\n";
free($raw);
$b = buffer_new<Body>(2);
$b[1]->vel->x = 4.0;
step($b, 2, 0.5);
echo $b[0]->pos->x, " ", $b[1]->pos->x, "\n";
"#,
    );
    assert_eq!(out, "12 0.5 2.5\n1\n0 2\n");
}

/// Verifies a packed class reached only through `ptr_cast<T>` keeps its layout after declaration
/// pruning, so field access through the typed pointer compiles and runs.
#[test]
fn test_packed_class_named_only_by_ptr_cast_is_kept() {
    let out = compile_and_run(
        r#"<?php
extern function calloc(u64 $count, u64 $size): ptr;
extern function free(ptr $p): void;
packed class Pair { public int $left; public int $right; }
$raw = calloc(1, 16);
$pair = ptr_cast<Pair>($raw);
$pair->right = 42;
$pair->left = $pair->right - 1;
echo $pair->left, ":", $pair->right, ":", ptr_get(ptr_offset($raw, 8)), "\n";
free($raw);
"#,
    );
    assert_eq!(out, "41:42:42\n");
}

/// Verifies whole-record assignment copies the inner record's bytes from a buffer element, from
/// another nested field and from a `ptr<T>`, and that the copy does not alias its source.
#[test]
fn test_nested_packed_field_whole_record_assignment_copies_bytes() {
    let out = compile_and_run(
        r#"<?php
extern function calloc(u64 $count, u64 $size): ptr;
extern function free(ptr $p): void;
packed class Vec2 { public float $x; public float $y; }
packed class Body { public Vec2 $pos; public Vec2 $vel; public int $id; }
packed class World { public int $tick; public Body $player; }
$points = buffer_new<Vec2>(1);
$points[0]->x = 1.5;
$points[0]->y = 2.5;
$b = buffer_new<Body>(2);
$b[0]->pos = $points[0];
$points[0]->x = 99.0;
echo $b[0]->pos->x, " ", $b[0]->pos->y, "\n";
$b[0]->vel = $b[0]->pos;
$b[0]->vel = $b[0]->vel;
echo $b[0]->vel->x, " ", $b[0]->vel->y, "\n";
$raw = calloc(1, ptr_sizeof("Vec2"));
$v = ptr_cast<Vec2>($raw);
$v->y = -4.0;
$b[1]->pos = $v;
echo $b[1]->pos->x, " ", $b[1]->pos->y, "\n";
free($raw);
$b[1]->id = 7;
$w = buffer_new<World>(1);
$w[0]->tick = 3;
$w[0]->player = $b[1];
$w[0]->player->pos = $b[0]->vel;
echo $w[0]->tick, " ", $w[0]->player->id, " ", $w[0]->player->pos->x, " ", $w[0]->player->pos->y, " ", $b[1]->pos->y, "\n";
"#,
    );
    assert_eq!(out, "1.5 2.5\n1.5 2.5\n0 -4\n3 7 1.5 2.5 -4\n");
}

/// Verifies a field read through a null `ptr<T>` aborts with the pointer extension's fatal
/// instead of faulting.
#[test]
fn test_typed_pointer_field_read_on_null_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
packed class Vec2 { public float $x; public float $y; }
$p = ptr_cast<Vec2>(ptr_null());
echo $p->x;
"#,
    );
    assert!(err.contains("null pointer dereference"), "{}", err);
}

/// Verifies a field write through a null `ptr<T>` aborts with the pointer extension's fatal.
#[test]
fn test_typed_pointer_field_write_on_null_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
packed class Vec2 { public float $x; public float $y; }
$p = ptr_cast<Vec2>(ptr_null());
$p->y = 1.0;
"#,
    );
    assert!(err.contains("null pointer dereference"), "{}", err);
}

/// Verifies copying a whole record from a null `ptr<T>` aborts instead of reading address zero.
#[test]
fn test_nested_packed_record_copy_from_null_pointer_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
packed class Vec2 { public float $x; public float $y; }
packed class Body { public Vec2 $pos; }
$b = buffer_new<Body>(1);
$b[0]->pos = ptr_cast<Vec2>(ptr_null());
"#,
    );
    assert!(err.contains("null pointer dereference"), "{}", err);
}
