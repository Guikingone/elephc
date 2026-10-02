//! Purpose:
//! Finds the static methods that only a DESCENDANT of the current class declares, for a
//! `static::method()` call made from that class.
//!
//! Called from:
//! - `crate::types::checker::inference::objects::methods` (type inference of the call).
//! - `crate::ir_lower::expr::static_method_calls` (lowering of the call).
//!
//! Key details:
//! - PHP resolves `static::m()` against the late-static class when the call runs. A method the
//!   lexical class never declared (a protected `getFacadeAccessor()` that only each subclass
//!   defines) is therefore reachable, and visible: a protected method is accessible from any
//!   scope related to its declaring class in either direction.
//! - Both passes must agree on the target list, so it is computed in one place. The list is sorted
//!   so the emitted dispatch ladder is deterministic.
//! - No stub is synthesized in the parent: that would change visibility, `method_exists()` and the
//!   `__callStatic` fallback. The call site dispatches on the late-static class name instead.

use crate::fast_hash::FastMap;
use crate::parser::ast::Visibility;
use crate::types::ClassInfo;

/// Returns the classes below `current` that make `method_key` callable as `static::method()`.
///
/// Empty when `current` itself already has the method (static or instance), because the ordinary
/// static-vtable dispatch covers that case. Private methods are skipped: from the parent's scope
/// PHP treats them as inaccessible and falls back to `__callStatic`.
pub(crate) fn child_only_static_targets(
    classes: &FastMap<String, ClassInfo>,
    current: &str,
    method_key: &str,
) -> Vec<String> {
    let Some(current_info) = classes.get(current) else {
        return Vec::new();
    };
    if current_info.static_methods.contains_key(method_key)
        || current_info.methods.contains_key(method_key)
    {
        return Vec::new();
    }
    let mut targets = classes
        .iter()
        .filter(|(name, info)| {
            name.as_str() != current
                && info.static_methods.contains_key(method_key)
                && !matches!(
                    info.static_method_visibilities.get(method_key),
                    Some(Visibility::Private)
                )
                && descends_from(classes, name, current)
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    targets.sort();
    targets
}

/// Returns whether `class_name` has `ancestor` somewhere on its parent chain.
fn descends_from(classes: &FastMap<String, ClassInfo>, class_name: &str, ancestor: &str) -> bool {
    let mut cursor = classes.get(class_name).and_then(|info| info.parent.as_deref());
    // The depth cap keeps a malformed inheritance cycle from spinning.
    for _ in 0..256 {
        let Some(parent) = cursor else {
            return false;
        };
        if parent == ancestor {
            return true;
        }
        cursor = classes.get(parent).and_then(|info| info.parent.as_deref());
    }
    false
}
