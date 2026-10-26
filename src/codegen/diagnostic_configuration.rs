//! Purpose:
//! Seeds startup diagnostic destinations from the program's INI overrides.
//!
//! Called from:
//! - Main entry and web request reset before PHP code runs.
//!
//! Key details:
//! - Settings are per-program stores, so the shared runtime object stays configuration-neutral.
//! - Default destinations preserve existing Elephc diagnostics until the remaining INI surface is complete.

use crate::codegen::{abi, Emitter};

/// Publishes startup display/log destinations for this program or fresh web request.
pub(super) fn emit(emitter: &mut Emitter, web: bool) {
    let overrides = crate::codegen_support::ini_overrides();
    let latest = |name: &str| overrides.iter().rev().find(|(key, _)| key == name).map(|(_, value)| value.as_str());
    let display = latest("display_errors").map_or(1, display_mode);
    // PHP sends stderr display to descriptor two only in CLI/CGI/phpdbg. Web SAPIs display
    // through their response writer, which is the ordinary captured stdout path here.
    let display = if web && display == 2 { 1 } else { display };
    let log = latest("log_errors").map_or(1, |raw| i64::from(crate::opcache::directives::parse_ini_bool(raw)));
    for (symbol, value) in [("_php_display_errors", display), ("_php_log_errors", log)] {
        let result = abi::int_result_reg(emitter);
        abi::emit_load_int_immediate(emitter, result, value);
        abi::emit_store_reg_to_symbol(emitter, result, symbol, 0);
    }
}

/// Applies php_get_display_errors_mode, including its unsigned-byte numeric projection.
fn display_mode(raw: &str) -> i64 {
    if ["on", "yes", "true", "stdout"].iter().any(|name| raw.eq_ignore_ascii_case(name)) {
        return 1;
    }
    if raw.eq_ignore_ascii_case("stderr") { return 2; }
    let raw = std::ffi::CString::new(raw.split('\0').next().unwrap_or_default()).unwrap();
    // All supported target profiles have a 64-bit zend_long, matching the host strtol here.
    let mode = unsafe { libc::strtol(raw.as_ptr(), std::ptr::null_mut(), 10) } as u8;
    match mode { 0 => 0, 2 => 2, _ => 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::Target;

    /// PHP's display selector recognizes words and wraps numeric modes through uint8_t.
    #[test]
    fn display_selector_preserves_php_numeric_projection() {
        for (raw, expected) in [("stderr", 2), ("STDERR", 2), ("stdout", 1), ("On", 1),
                               ("0", 0), ("2", 2), ("258", 2), ("256", 0),
                               ("-254", 2), ("3", 1), ("garbage", 0)] {
            assert_eq!(display_mode(raw), expected, "{raw}");
        }
    }

    /// CLI stderr display differs from web response display on every supported codegen target.
    #[test]
    fn destination_seed_covers_targets_and_web_projection() {
        let saved = crate::codegen_support::ini_overrides();
        crate::codegen_support::set_ini_overrides(vec![
            ("display_errors".into(), "stderr".into()), ("log_errors".into(), "0".into()),
        ]);
        let mut pairs = Vec::new();
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let target = Target::parse(name).unwrap();
            let mut cli = Emitter::new(target);
            emit(&mut cli, false);
            let mut web = Emitter::new(target);
            emit(&mut web, true);
            pairs.push((name, cli.output(), web.output()));
        }
        crate::codegen_support::set_ini_overrides(saved);
        for (name, cli, web) in pairs {
            assert_ne!(cli, web, "{name}: web must use response capture");
            for text in [&cli, &web] {
                assert!(text.contains("_php_display_errors"), "{name}");
                assert!(text.contains("_php_log_errors"), "{name}");
            }
        }
    }
}
