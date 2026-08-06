//! Purpose:
//! Describes the call sequence for invoking a hosted extension function, as a
//! value the code generator can consume and tests can assert on.
//!
//! Called from:
//! - Codegen, when lowering a call to a function provided by a hosted extension.
//!
//! Key details:
//! - Layout comes from `ZEND_CALL_ARG`: the `zend_execute_data` occupies the
//!   first `frame_slot` zval-sized slots, and argument N sits at
//!   `frame_slot + N - 1`. Argument count lives in `This.u2.num_args`. This is
//!   the frame the prototype hand-built to call `zif_simdjson_decode`.
//! - `zif_*` symbols from a C++ extension are C++-mangled; from a C extension
//!   they are not. Getting this wrong is a link error, not a runtime surprise.
//! - Describing the sequence separately from emitting it keeps the decisions
//!   assertable without running a compiler.

/// How a hosted function is reached at the ABI level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallPlan {
    /// The extension's entry point, as the linker sees it.
    pub symbol: String,
    /// zval-sized slots occupied by `zend_execute_data` before argument 0.
    pub frame_slot: usize,
    /// Declared arguments actually passed.
    pub arg_count: usize,
    /// Total zval-sized slots to reserve for the frame.
    pub frame_slots_total: usize,
}

impl CallPlan {
    /// Slot index of argument `n`, one-based as `ZEND_CALL_ARG` counts.
    pub fn arg_slot(&self, n: usize) -> Option<usize> {
        if n == 0 || n > self.arg_count {
            return None;
        }
        Some(self.frame_slot + n - 1)
    }
}

/// zval-sized slots the execute_data header occupies, mirroring
/// `ZEND_CALL_FRAME_SLOT` = ceil(sizeof(zend_execute_data) / sizeof(zval)).
pub fn frame_slot(execute_data_size: usize, zval_size: usize) -> usize {
    execute_data_size.div_ceil(zval_size)
}

/// The exported name of a PHP-visible function. `PHP_FUNCTION(foo)` expands to
/// `zif_foo`, which C++ translation units then mangle.
pub fn entry_symbol(function: &str, extension_is_cxx: bool) -> String {
    let base = format!("zif_{function}");
    if extension_is_cxx {
        // Itanium ABI: _Z <len> <name> P18_zend_execute_dataP12_zval_struct
        format!(
            "_Z{}{}P18_zend_execute_dataP12_zval_struct",
            base.len(),
            base
        )
    } else {
        base
    }
}

/// Builds the plan for one call site.
pub fn plan_call(
    function: &str,
    arg_count: usize,
    extension_is_cxx: bool,
    execute_data_size: usize,
    zval_size: usize,
) -> CallPlan {
    let frame_slot = frame_slot(execute_data_size, zval_size);
    CallPlan {
        symbol: entry_symbol(function, extension_is_cxx),
        frame_slot,
        arg_count,
        // At least one slot even with no arguments: the frame is still a frame.
        frame_slots_total: frame_slot + arg_count.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 64-bit shapes: zend_execute_data is 80 bytes, zval is 16.
    const ED: usize = 80;
    const ZV: usize = 16;

    #[test]
    fn frame_slot_rounds_up() {
        assert_eq!(frame_slot(80, 16), 5);
        // A header that does not divide evenly still gets a whole slot.
        assert_eq!(frame_slot(81, 16), 6);
    }

    /// The exact layout the prototype used to call zif_simdjson_decode.
    #[test]
    fn arguments_follow_the_frame_header() {
        let plan = plan_call("simdjson_decode", 3, true, ED, ZV);
        assert_eq!(plan.arg_slot(1), Some(5));
        assert_eq!(plan.arg_slot(2), Some(6));
        assert_eq!(plan.arg_slot(3), Some(7));
        assert_eq!(plan.frame_slots_total, 8);
    }

    #[test]
    fn out_of_range_arguments_have_no_slot() {
        let plan = plan_call("apcu_enabled", 0, false, ED, ZV);
        assert_eq!(plan.arg_slot(1), None, "no arguments were passed");
        assert_eq!(plan.arg_slot(0), None, "ZEND_CALL_ARG is one-based");
    }

    /// A zero-argument call still needs a frame.
    #[test]
    fn a_frame_is_reserved_even_with_no_arguments() {
        let plan = plan_call("apcu_clear_cache", 0, false, ED, ZV);
        assert!(plan.frame_slots_total > plan.frame_slot);
    }

    /// C extensions export the plain name; getting this wrong is a link error.
    #[test]
    fn c_extensions_export_the_plain_symbol() {
        assert_eq!(entry_symbol("apcu_fetch", false), "zif_apcu_fetch");
    }

    /// simdjson is C++, and its real symbol is
    /// _Z19zif_simdjson_decodeP18_zend_execute_dataP12_zval_struct — verified
    /// against `nm` on the built extension during the prototype.
    #[test]
    fn cxx_extensions_export_a_mangled_symbol() {
        assert_eq!(
            entry_symbol("simdjson_decode", true),
            "_Z19zif_simdjson_decodeP18_zend_execute_dataP12_zval_struct"
        );
    }

    #[test]
    fn mangled_length_tracks_the_name() {
        // zif_ds_vector_push is 18 characters.
        assert!(entry_symbol("ds_vector_push", true).starts_with("_Z18zif_ds_vector_push"));
    }
}
