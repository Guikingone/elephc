//! Purpose:
//! Renders runtime environment overrides and OPcache INI helpers.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Only reporting directives accept runtime overrides; compiled behavior stays frozen.

#[allow(unused_imports)]
use super::*;

/// The RUNTIME per-directive environment-override helper block, written in elephc-PHP and baked
/// verbatim (it carries no per-target data — the per-directive facts are the arguments its call
/// sites pass). Injected exactly once per binary alongside whichever surface needs it; see
/// [`render_opcache_env_helpers`] for the injection rules and the reference-PHP non-parity note.
///
/// The block is four layers:
///
/// 1. `__elephc_opcache_env($u, $d)` — the LOOKUP. `$u` is the `__` spelling
///    (`ELEPHC_INI_opcache__save_comments`), `$d` the dotted one
///    (`ELEPHC_INI_opcache.save_comments`); the dotted form is consulted ONLY when the `__` form
///    is empty. Both names are rendered in RUST (`directive_env_var_names`) and passed as
///    literals, so no runtime string surgery is on the path and the derivation is unit-testable.
/// 2. `__elephc_ini_scan($v)` — the SCANNER, the PHP mirror of
///    `crate::opcache::directives::ini_scanner_value`. It rewrites the boolean-alias barewords
///    (`on`/`true`/`yes` → `'1'`, `off`/`false`/`no`/`none`/`null` → `''`) for EVERY directive
///    type, and — exactly as in Rust — it runs BEFORE every normalizer and before the raw-string
///    surface reports anything, so `ELEPHC_INI_opcache__preferred_memory_model=on` reports `'1'`.
/// 3. The NORMALIZERS — the PHP mirror of `crate::opcache::directives::parse_ini_override`, one
///    `_val` converter per type plus, for the ONE type that can still refuse a value, an `_ok`
///    predicate. Only `_pct` needs the predicate: `__elephc_ini_bool_val` and
///    `__elephc_ini_quantity` mirror handlers that CANNOT FAIL (`zend_ini_parse_bool` and
///    `zend_ini_parse_quantity`), so there is nothing for an `_ok` half to answer. The split
///    exists because a single function cannot return both "did it parse" and the parsed value
///    without a union return.
/// 4. `__elephc_opcache_env_bool` / `_int` / `_float` / `_pct` / `_str` (the TYPED surface that
///    feeds `opcache_get_configuration()['directives']`) and `__elephc_opcache_env_raw` (the RAW
///    STRING surface that feeds `ini_get()` / `ini_get_all()`). Both consult the same lookup, the
///    same scanner and the same `_ok` predicate, which is what makes the two surfaces move
///    TOGETHER: a value the typed side stores is the SCANNED value the raw side reports, and the
///    one value `_pct` rejects leaves BOTH at the compile-time default.
///
/// TWO OVERFLOW NARROWINGS, both unreachable for a real directive value and both documented
/// rather than modelled: `__elephc_ini_quantity` accumulates in PHP integers (the Rust side
/// carries a `u128` so it can reproduce `strtoul`'s ULONG_MAX-on-overflow result), and it does
/// not carry the quantity DIAGNOSTICS — `ini_override_warnings` emits those at compile time,
/// where reference PHP emits them, and a compiled binary has no startup phase to warn in.
///
/// EMPTY MEANS UNSET. `getenv` reports a missing variable as an empty string in elephc's runtime
/// (`__rt_getenv` returns `(ptr 0, len 0)` on a NULL from libc), so an environment variable set
/// to the empty string is indistinguishable from an unset one and is treated as unset — the
/// compile-time value stays. `--ini opcache.error_log=` (compile time) still stores the empty
/// string; only the runtime path has this floor. It is documented rather than worked around
/// because no mechanism can distinguish the two through `getenv`.
///
/// WHITESPACE: the normalizers use PHP's `trim()`, whose default charlist is
/// `" \t\n\r\0\x0B"`, against a Rust side that uses `str::trim` (`" \t\n\r\x0B\x0C"` plus
/// Unicode). The two disagree only on a leading/trailing NUL or form feed, which no directive
/// value carries.
pub(super) const ENV_OVERRIDE_HELPERS: &str = r#"function __elephc_opcache_env(string $u, string $d): string {
    $v = (string) getenv($u);
    if ($v !== '') { return $v; }
    return (string) getenv($d);
}
function __elephc_ini_scan(string $v): string {
    $l = strtolower(trim($v));
    if ($l === 'on' || $l === 'true' || $l === 'yes') { return '1'; }
    if ($l === 'off' || $l === 'false' || $l === 'no' || $l === 'none' || $l === 'null') { return ''; }
    return $v;
}
function __elephc_ini_bool_val(string $v): bool {
    $l = strtolower($v);
    if ($l === 'true' || $l === 'yes' || $l === 'on') { return true; }
    return __elephc_ini_atoi($v) !== 0;
}
function __elephc_ini_isspace(string $c): bool {
    $o = ord($c);
    return $o === 32 || ($o >= 9 && $o <= 13);
}
function __elephc_ini_digit(string $c, int $radix): int {
    $o = ord($c);
    $d = -1;
    if ($o >= 48 && $o <= 57) { $d = $o - 48; }
    if ($o >= 97 && $o <= 122) { $d = $o - 87; }
    if ($o >= 65 && $o <= 90) { $d = $o - 55; }
    if ($d < 0 || $d >= $radix) { return -1; }
    return $d;
}
function __elephc_ini_quantity(string $v): int {
    $n = strlen($v);
    if ($n === 0) { return 0; }
    $s = 0;
    while ($s < $n && __elephc_ini_isspace(substr($v, $s, 1))) { $s = $s + 1; }
    $e = $n;
    while ($e > $s && __elephc_ini_isspace(substr($v, $e - 1, 1))) { $e = $e - 1; }
    if ($s >= $e) { return 0; }
    $neg = substr($v, $s, 1) === '-';
    $i = $s;
    $c = substr($v, $i, 1);
    if ($c === '-' || $c === '+') { $i = $i + 1; }
    if ($i >= $e) { return 0; }
    $o = ord(substr($v, $i, 1));
    if ($o < 48 || $o > 57) { return 0; }
    $radix = 10;
    if ($o === 48) {
        $radix = 8;
        if ($i + 1 < $e) {
            $p = strtolower(substr($v, $i + 1, 1));
            if ($p === 'x') { $radix = 16; $i = $i + 2; }
            if ($p === 'b') { $radix = 2; $i = $i + 2; }
        }
    }
    if ($i >= $e) { return 0; }
    if (__elephc_ini_digit(substr($v, $i, 1), $radix) < 0) { return 0; }
    $acc = 0;
    while ($i < $e) {
        $d = __elephc_ini_digit(substr($v, $i, 1), $radix);
        if ($d < 0) { break; }
        $acc = $acc * $radix + $d;
        $i = $i + 1;
    }
    if ($neg) { $acc = -$acc; }
    if ($i >= $e) { return $acc; }
    $last = strtolower(substr($v, $e - 1, 1));
    if ($last === 'k') { return $acc * 1024; }
    if ($last === 'm') { return $acc * 1048576; }
    if ($last === 'g') { return $acc * 1073741824; }
    return $acc;
}
function __elephc_ini_atoi(string $v): int {
    $s = ltrim($v);
    $n = strlen($s);
    $i = 0;
    $neg = false;
    $c = substr($s, 0, 1);
    if ($c === '-') { $neg = true; $i = 1; }
    if ($c === '+') { $i = 1; }
    $acc = 0;
    $seen = 0;
    while ($i < $n) {
        $o = ord(substr($s, $i, 1));
        if ($o < 48 || $o > 57) { break; }
        if ($seen < 18) { $acc = $acc * 10 + ($o - 48); }
        $seen = $seen + 1;
        $i = $i + 1;
    }
    if ($neg) { return -$acc; }
    return $acc;
}
function __elephc_ini_pct_ok(string $v): bool {
    $p = __elephc_ini_atoi($v);
    return $p > 0 && $p <= 50;
}
function __elephc_ini_pct_val(string $v): float {
    return __elephc_ini_atoi($v) / 100.0;
}
function __elephc_opcache_env_bool(string $u, string $d, bool $def): bool {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return __elephc_ini_bool_val(__elephc_ini_scan($v));
}
function __elephc_opcache_env_int(string $u, string $d, int $def): int {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return __elephc_ini_quantity(__elephc_ini_scan($v));
}
function __elephc_opcache_env_float(string $u, string $d, float $def): float {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return (float) trim(__elephc_ini_scan($v));
}
function __elephc_opcache_env_trunc(string $u, string $d, int $def): int {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return (int) (float) trim(__elephc_ini_scan($v));
}
function __elephc_opcache_env_pct(string $u, string $d, float $def): float {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    $s = __elephc_ini_scan($v);
    if (__elephc_ini_pct_ok($s)) { return __elephc_ini_pct_val($s); }
    return $def;
}
function __elephc_opcache_env_str(string $u, string $d, string $def): string {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    $s = __elephc_ini_scan($v);
    return $s;
}
function __elephc_opcache_env_raw(string $u, string $d, string $t, string $def): string {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    $s = __elephc_ini_scan($v);
    if ($t === 'p') { if (__elephc_ini_pct_ok($s)) { return $s; } return $def; }
    return $s;
}
"#;

/// Returns the RUNTIME environment-override helper block ([`ENV_OVERRIDE_HELPERS`]).
///
/// WHY THIS EXISTS AT ALL. Every `opcache.*` directive is compiled into the binary, so the
/// natural analogue of `php -d` is elephc's compile-time `--ini KEY=VALUE`. That leaves no way to
/// re-point a directive on an ALREADY-BUILT binary, which is exactly what a deployment needs
/// (`ELEPHC_INI_opcache__save_comments=0 ./app`). Reference PHP has no per-directive environment
/// override to copy — VERIFIED on 8.5.6 that `PHP_INI_opcache_jit`, `opcache_jit` and
/// `opcache.jit` in the environment all leave `ini_get('opcache.jit')` at the compiled default —
/// so this is a documented elephc EXTENSION, not a parity feature. Precedence:
/// baked default → `--ini` (compile time) → `ELEPHC_INI_*` (runtime, wins).
///
/// WHY IT IS PHP RATHER THAN RUST. A plain CLI binary links NO Rust staticlib — every elephc
/// runtime is an opt-in bridge selected in `crate::linker` — so a Rust-side override table would
/// force every binary to link one (killing pay-for-use) or need a hand-written `__rt_*` helper in
/// assembly for four targets. `getenv` is already a first-class codegen builtin with a CONCRETE
/// `Str` EIR result type, available identically on CLI and `--web`, so baking the lookup as PHP
/// costs nothing and works on every target the compiler already supports.
///
/// INJECTED EXACTLY ONCE. The block declares plain functions, so a second copy is a redeclaration
/// error. Ownership mirrors `render_opcache_ini_helpers`: under `--web` the web prelude bakes it
/// (see `crate::web_prelude`), and `inject_if_used` emits it only when NOT web. On CLI it is
/// emitted when `opcache_get_configuration` is injected (its directives array calls the typed
/// helpers — including through the RESTRICTED template's dead array exit, which still has to
/// resolve) or when the `opcache.*` INI dispatcher is injected (its raw-string arms call
/// `__elephc_opcache_env_raw`), and never twice.
pub(crate) fn render_opcache_env_helpers() -> String {
    ENV_OVERRIDE_HELPERS.to_string()
}

/// Renders the PHP expression that yields directive `name`'s effective TYPED value at runtime:
/// the compile-time literal for a directive outside the runtime-override scope
/// ([`directive_runtime_overridable`]), otherwise a call into the typed environment helper with
/// the two environment-variable spellings and the compile-time value as the default.
///
/// `value` is the EFFECTIVE compile-time value (defaults with `--ini` already applied), which is
/// what makes the precedence chain baked default → `--ini` → env fall out for free: the env
/// helper's `$def` argument IS the `--ini`-resolved value, so an unset or invalid environment
/// variable reproduces today's output exactly.
pub(super) fn render_directive_value_expr(name: &str, value: &DirectiveValue) -> String {
    let literal = render_directive_value(value);
    if !directive_runtime_overridable(name) {
        return literal;
    }
    let (under, dotted) = directive_env_var_names(name);
    let helper = match directive_env_type_code(name, value) {
        'b' => "__elephc_opcache_env_bool",
        'i' => "__elephc_opcache_env_int",
        'p' => "__elephc_opcache_env_pct",
        'f' => "__elephc_opcache_env_float",
        // `opcache.jit_prof_threshold` in the 8.2 profile ONLY: a `zend_strtod` READ whose value is
        // REPORTED truncated to an int (php-src 8.2 uses `add_assoc_long` on a `double` field).
        // See `crate::opcache::directives::JIT_PROF_THRESHOLD`.
        't' => "__elephc_opcache_env_trunc",
        _ => "__elephc_opcache_env_str",
    };
    format!(
        "{helper}({}, {}, {literal})",
        render_php_single_quoted(&under),
        render_php_single_quoted(&dotted),
    )
}

/// Renders the shared `opcache.*` INI helper functions for the compile target, baked from the
/// version-keyed directive table so CLI and `--web` share one source of truth:
///
/// - `__elephc_opcache_ini_string(string $option): string|false` — the RAW INI STRING for an
///   `opcache.*` directive (what `ini_get` reports), or `false` for a non-opcache key.
/// - `__elephc_opcache_ini_access(string $option): int` — the `PHP_INI_*` access bitmask
///   (`7`/`4`) for an `opcache.*` directive, or `-1` for a non-opcache key.
/// - `__elephc_opcache_ini_keys(): array` — the `opcache.*` directive names SORTED ASCENDING.
/// - `__elephc_opcache_ini_all_details(): array` — the whole block as
///   `['global_value' => rawstr, 'local_value' => rawstr, 'access' => N]` entries.
/// - `__elephc_opcache_ini_all_plain(): array` — the whole block as flat raw strings.
///
/// The raw strings and access levels come from `directive_ini_string` / `directive_access`
/// (byte-verified against reference PHP 8.5.6), so this is a pure projection of the same table
/// that backs `opcache_get_configuration()`. Rendered as `if`-chains (matching the session INI
/// dispatcher's proven shape) so no `switch`/`match` lowering is on the path.
///
/// KEY ORDER: `ini_get_all` reports its keys SORTED ASCENDING (reference PHP 8.5.6), so the
/// rendered key list is a sorted COPY. `opcache_directives()` itself keeps REGISTRATION order,
/// which is what `opcache_get_configuration()['directives']` reports and is byte-correct there
/// — it must not be reordered. Only this projection sorts.
///
/// TWO ALL-HELPERS, NOT ONE `$details` LOOP: a function that writes an ARRAY-LITERAL value on
/// one branch and a SCALAR on the other into the SAME array slot inside one loop miscompiles in
/// elephc's codegen — SIGSEGV or heap exhaustion, with no diagnostic. (Reproduced: a single
/// dual-shape `ini_get_all` loop made `ini_get_all(null, false)` exit 139 with no output, and
/// crashed the `--web` worker into an empty HTTP reply.) The rule is ONE VALUE SHAPE PER
/// FUNCTION, so the `$details` branch is resolved by the CALLER picking a helper.
pub(crate) fn render_opcache_ini_helpers(
    php_version: PhpVersion,
    overrides: &[(String, String)],
) -> String {
    let version_id = php_version.version_id();
    let directives = opcache_directives(version_id);

    // __elephc_opcache_ini_string: raw INI string per opcache key; false for anything else. The
    // raw string is the user's `--ini` override verbatim when validly overridden, else the
    // default projection (`effective_directive_ini_string`). Access levels and the key list do
    // not vary with overrides, so only the string arm consults them.
    let mut string_arms = String::new();
    for (name, value) in &directives {
        let raw = effective_directive_ini_string(name, value, overrides);
        // RUNTIME env override (`ELEPHC_INI_*`) for the reporting-only directives: the arm returns
        // a call that yields the environment value VERBATIM when it parses for the directive's
        // type and the compile-time raw string otherwise. Excluded directives keep the plain
        // literal — see `directive_runtime_overridable` for why honoring them here would make the
        // binary contradict its own `opcache_get_status()`.
        let arm = if directive_runtime_overridable(name) {
            // The type code is read off the DEFAULT value: `parse_ini_override` preserves the
            // `DirectiveValue` variant, so a `--ini` override never changes a directive's type.
            let (under, dotted) = directive_env_var_names(name);
            format!(
                "__elephc_opcache_env_raw({}, {}, '{}', {})",
                render_php_single_quoted(&under),
                render_php_single_quoted(&dotted),
                directive_env_type_code(name, value),
                render_php_single_quoted(&raw),
            )
        } else {
            render_php_single_quoted(&raw)
        };
        string_arms.push_str(&format!(
            "    if ($option === {}) {{ return {arm}; }}\n",
            render_php_single_quoted(name),
        ));
    }

    // __elephc_opcache_ini_null: whether ini_get_all() reports this directive's global_value /
    // local_value as PHP `null` rather than a string. Reference PHP does that for exactly the
    // directives php-src registers with a C NULL default AND that were never assigned a value —
    // `opcache.file_cache` is the only one in the block (see `directive_ini_null_default`).
    //
    // THE `?string` RETURN HINT ON `__elephc_opcache_ini_detail_value` IS LOAD-BEARING. Without an
    // explicit union hint elephc infers the function's return as plain `Str` and COERCES the
    // `return null` to `''` — reproduced: the same body typed `?string` var_dumps `NULL` and typed
    // implicitly var_dumps `string(0) ""`, which is exactly the bug being fixed here.
    //
    // A COMPILE-TIME `--ini opcache.file_cache=<v>` ASSIGNS it, so the arm collapses to `false`
    // (reference reports `''`/`''` for `-d opcache.file_cache=` and `'/x'`/`'/x'` for
    // `-d opcache.file_cache=/x`; only the untouched run reports NULL/NULL). Otherwise the arm
    // consults the RUNTIME environment override with the same "empty means unset" rule the rest of
    // the `ELEPHC_INI_*` surface uses, so `ELEPHC_INI_opcache__file_cache=/x` flips it to a string
    // exactly as `-d` would.
    let null_arms: Vec<String> = directives
        .iter()
        .filter(|(name, _)| directive_ini_null_default(name))
        .map(|(name, _)| {
            let condition = if latest_ini_override(overrides, name).is_some() {
                "false".to_string()
            } else {
                let (under, dotted) = directive_env_var_names(name);
                format!(
                    "__elephc_opcache_env({}, {}) === ''",
                    render_php_single_quoted(&under),
                    render_php_single_quoted(&dotted),
                )
            };
            format!(
                "    if ($option === {}) {{ return {condition}; }}\n",
                render_php_single_quoted(name),
            )
        })
        .collect();
    let null_arms = null_arms.concat();

    // __elephc_opcache_ini_access: 7 for the PHP_INI_ALL directives, 4 for the rest, and -1
    // for a non-opcache key (detected by the string dispatcher returning false).
    let all_conditions: Vec<String> = directives
        .iter()
        .filter(|(name, _)| directive_access(name) == 7)
        .map(|(name, _)| format!("$option === {}", render_php_single_quoted(name)))
        .collect();
    let all_expr = all_conditions.join("\n        || ");

    // __elephc_opcache_ini_keys: the directive-name list for ini_get_all, SORTED ASCENDING to
    // match reference PHP's ini_get_all key order. This is a sorted COPY: the table backing
    // opcache_get_configuration() keeps registration order and is left untouched.
    let mut ini_keys: Vec<&str> = directives.iter().map(|(name, _)| *name).collect();
    ini_keys.sort_unstable();
    let mut keys_literal = String::from("[");
    for (index, name) in ini_keys.iter().enumerate() {
        if index > 0 {
            keys_literal.push_str(", ");
        }
        keys_literal.push_str(&render_php_single_quoted(name));
    }
    keys_literal.push(']');

    format!(
        "function __elephc_opcache_ini_string(string $option): string|false {{\n\
         {string_arms}    return false;\n\
         }}\n\
         function __elephc_opcache_ini_null(string $option): bool {{\n\
         {null_arms}    return false;\n\
         }}\n\
         function __elephc_opcache_ini_detail_value(string $option): ?string {{\n\
         \x20   if (__elephc_opcache_ini_null($option)) {{ return null; }}\n\
         \x20   $__elephc_raw = (string) __elephc_opcache_ini_string($option);\n\
         \x20   return $__elephc_raw;\n\
         }}\n\
         function __elephc_opcache_ini_access(string $option): int {{\n\
         \x20   if (__elephc_opcache_ini_string($option) === false) {{ return -1; }}\n\
         \x20   if ({all_expr}) {{ return 7; }}\n\
         \x20   return 4;\n\
         }}\n\
         function __elephc_opcache_ini_keys(): array {{\n\
         \x20   return {keys_literal};\n\
         }}\n\
         function __elephc_opcache_ini_all_details(): array {{\n\
         \x20   $__elephc_all = [];\n\
         \x20   foreach (__elephc_opcache_ini_keys() as $__elephc_k) {{\n\
         \x20       $__elephc_v = __elephc_opcache_ini_detail_value($__elephc_k);\n\
         \x20       $__elephc_all[$__elephc_k] = ['global_value' => $__elephc_v, 'local_value' => $__elephc_v, 'access' => __elephc_opcache_ini_access($__elephc_k)];\n\
         \x20   }}\n\
         \x20   return $__elephc_all;\n\
         }}\n\
         function __elephc_opcache_ini_all_plain(): array {{\n\
         \x20   $__elephc_all = [];\n\
         \x20   foreach (__elephc_opcache_ini_keys() as $__elephc_k) {{\n\
         \x20       $__elephc_all[$__elephc_k] = __elephc_opcache_ini_detail_value($__elephc_k);\n\
         \x20   }}\n\
         \x20   return $__elephc_all;\n\
         }}\n"
    )
}
