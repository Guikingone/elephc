//! Purpose:
//! The `mysqli` connection class, as an elephc-PHP source fragment. Holds the
//! opaque `int` bridge connection handle and (in later tasks) the DSN builder,
//! connect/escape/charset/transaction methods, and the refreshed public
//! properties.
//!
//! Called from:
//! - `crate::mysqli_prelude::fragments::source_for_version` (concatenated into
//!   the injected prelude).
//!
//! Key details:
//! - `$conn = -1` means "not connected" (`mysqli_init()` / `new mysqli()` with no
//!   arguments); a successful `real_connect` stores the `elephc_pdo_open_persistent`
//!   handle.
//! - `mysqli::$reportMode` is the process-wide `mysqli_report()` store; its
//!   default is version-gated in `fragments.rs` (3 = ERROR|STRICT for >= 8.1,
//!   0 = OFF for 8.0).
//! - Method-local variables (added in later tasks) use the `$_` prefix, same
//!   checker-clash rule as the PDO prelude.

/// The `mysqli` class skeleton and the `mysqli_connect` procedural constructor
/// (fragment without a `<?php` header).
pub(super) const SRC: &str = r#"
// -- mysqli connection surface --

class mysqli {
    // Opaque elephc_pdo bridge connection handle; -1 = not connected.
    public int $conn = -1;

    // Process-wide mysqli_report() mode. The literal default is rewritten at
    // injection time for --php-version 8.0 (OFF); see fragments.rs.
    public static int $reportMode = 3;
}

// Procedural constructor. Task 4 forwards the full six-argument connect
// signature; the skeleton only proves the surface injects without leaking PDO.
function mysqli_connect(): mysqli {
    return new mysqli();
}
"#;
