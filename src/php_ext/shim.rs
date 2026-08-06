//! Purpose:
//! Carries the Zend-engine stand-in that hosted extensions link against, as
//! embedded C sources compiled by the extension recipe.
//!
//! Called from:
//! - The `php-ext` recipe, which compiles these alongside the extension's own
//!   translation units and archives them together.
//!
//! Key details:
//! - Embedded C rather than a bridge crate. The bridge crates
//!   (`elephc-tz`, `elephc-crypto`, …) are deliberately pure Rust with no
//!   dependencies; the project's precedent for C is
//!   `native_deps/recipes/pcre2_shim.c`, embedded with `include_str!` and built
//!   by a recipe. This follows that, not the crate pattern.
//! - Split by concern so a target that needs less links less: `core` is the
//!   allocator, strings, hashtables and zvals; `classes` adds the object model;
//!   `arena` is the request/persistent split; `bailout` is the fatal-error
//!   contract.
//! - Verified outside the tree first: four binaries link real PECL object files
//!   (simdjson, ds) against these and match a real PHP oracle. See
//!   `spec-php-ext-shim` on the `spec/php-ext-shim` branch.

/// Allocator, `zend_string`, hashtables, zval lifetimes, argument parsing.
pub const CORE: &str = include_str!("shim/zend_shim.c");

/// Class registration, object handlers, property access, conversions —
/// everything a class-based extension needs beyond the core.
pub const CLASSES: &str = include_str!("shim/zend_shim_ds.c");

/// Request-scoped arena and persistent (`pemalloc`) allocations. This is what
/// makes a fatal survivable: teardown reclaims whatever the extension leaked.
pub const ARENA: &str = include_str!("shim/zend_shim_arena.c");

/// `zend_bailout`: the longjmp fatal-error contract, plus the host-side
/// protected frame that catches it.
pub const BAILOUT: &str = include_str!("shim/zend_shim_bailout.c");

/// Every shim translation unit, in link order.
pub const ALL: &[(&str, &str)] = &[
    ("elephc_zend_core.c", CORE),
    ("elephc_zend_classes.c", CLASSES),
    ("elephc_zend_arena.c", ARENA),
    ("elephc_zend_bailout.c", BAILOUT),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_unit_is_present_and_non_empty() {
        for (name, source) in ALL {
            assert!(
                source.len() > 500,
                "{name} looks truncated ({} bytes)",
                source.len()
            );
        }
    }

    /// The three ownership rules the prototype found the hard way, each of which
    /// caused a real crash or silent corruption before it was fixed. They are
    /// pinned here so a future edit cannot quietly drop one.
    #[test]
    fn ownership_contracts_are_still_honoured() {
        assert!(
            CORE.contains("GC_ADDREF(key)"),
            "zend_hash_update must take a reference on the key: callers do \
             zend_string_init -> update -> release, and without this the key is \
             freed under the table"
        );
        assert!(
            CORE.contains("IS_STR_INTERNED"),
            "global strings must be interned, or zend_string_release_ex frees them"
        );
        // zend_std_write_property lives in the core unit, not the class surface:
        // stdClass property writes are reachable without any class registration.
        assert!(
            CORE.contains("Z_ADDREF_P(value)"),
            "zend_std_write_property must take a reference on the value: \
             callers call zval_ptr_dtor_nogc immediately afterwards"
        );
    }

    /// Static linking catches missing symbols, never wrong behaviour — so an
    /// unexercised path must abort rather than return something plausible.
    #[test]
    fn unimplemented_paths_abort_rather_than_degrade() {
        assert!(CORE.contains("abort()"), "core must fail loudly");
        assert!(CLASSES.contains("abort()"), "class surface must fail loudly");
        assert!(
            !CORE.contains("return NULL; /* TODO"),
            "no silent stubs in the core shim"
        );
    }

    /// The fatal path only works if the arena reclaims what the extension leaked.
    #[test]
    fn bailout_and_arena_are_coupled() {
        assert!(BAILOUT.contains("SETJMP"), "the host installs a protected frame");
        assert!(BAILOUT.contains("LONGJMP"), "zend_bailout transfers control");
        assert!(
            ARENA.contains("shim_request_end"),
            "arena teardown is the cleanup mechanism that makes bailout survivable"
        );
        assert!(
            ARENA.contains("persistent"),
            "MINIT-time structures must outlive request teardown"
        );
    }
}
