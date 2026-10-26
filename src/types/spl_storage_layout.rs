//! Purpose:
//! Shares the compiler-private physical slot names used by SplObjectStorage.
//!
//! Called from:
//! - The builtin SPL declaration builder and native comparison metadata emitter.
//!
//! Key details:
//! - NUL-prefixed names cannot be declared as ordinary PHP properties.
//! - Native ownership includes these slots while PHP property views omit them.

pub(crate) const OBJECTS_PROPERTY: &str = "\0elephc_spl_objects";
pub(crate) const INFOS_PROPERTY: &str = "\0elephc_spl_infos";
pub(crate) const POSITION_PROPERTY: &str = "\0elephc_spl_position";
