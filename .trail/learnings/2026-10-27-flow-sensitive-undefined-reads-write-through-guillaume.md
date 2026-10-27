---
title: "Flow-sensitive undefined reads, write-through inits and literal GLOBALS"
author: Guillaume
date: 2026-10-27
branch: work/spl-opcache-main-integration
cost: { minutes: 1, tokens: 3843 }
---

## Decided
- `undefined_reads.rs` runs a second, flow-sensitive pass per scope after the whole-scope pass. `maybe` = names some path may have written: branches merge by union, `while`/`for`/`do` pre-add all body writes, a `try` body runs in order (catch/finally see all its writes), `foreach`'s subject is read before the loop's key/value writes, `unset($x)` kills `$x` (not in main for names any body binds with `global`).
- Inside one statement reads go in evaluation order (`rewrite_ordered`): an assignment's value is read before its variable target is written; calls keep the conservative view (every variable argument may be by-reference, so it counts as written).
- A statement that writes through a name nothing defined (`$o->p = 1`, `$a--`, `$b->y++`, `$c =& $a`, compound `$a[] .= x` on ArrayPush/ArrayAssign) gets `$x = <UndefinedVariable>` (warns) or `$x = null` (silent: reference source, plain property write) inserted before it. php: `$obj->a = 3` on undefined is silent + Error; `$a--` warns.
- Loops: the body is rewritten on a clone with the entry state; names of the loop's writes that come out undefined are hoisted before the loop (warning only when the body certainly runs: do/while, `for(;;)`).
- `$GLOBALS['lit']` (new `globals_array.rs`): main scope -> `$lit`; function/method/closure -> `$lit` + `global $lit;` at the body top, unless the scope already uses `$lit`.
- Main scope gets `global $x;` for names only functions/closures bind with `global` (main never writes them): the checker had no type for main's read. argc/argv excluded (broke eval ownership).
- Index expressions under isset/empty/unset/`??` are ordinary reads (`rewrite_probe`).
- Property write on a statically-null receiver (`PhpType::Void`) lowers to the RHS then php's `Attempt to assign property "p" on null` (instance_property_writes.rs).

## Tried and ruled out
- Treating a statement's own writes as preceding its reads hid `$x = isset($x) ? $x : []` and `foreach ($a as $a)`.
- `$a[] op= v` desugars to a temp + prelude statements inside the Assignment; not handled (fuzz-only shape).

## Watch out
- The harness `ProgramOutput::stdout` strips php-diagnostic-shaped lines (including `  thrown in F on line N`); assert on `stdout_raw` for those.
- Lowering emits class methods from `ClassInfo::method_decls`, a copy the post-check AST rewrite never touched: rewrite those too (`undefined_constants::rewrite_methods`).
- `undefined_constants::walk_scope_exprs_mut` stays in one variable scope (skips function/class bodies and non-arrow closures); `walk_program_exprs_mut` enters everything.
