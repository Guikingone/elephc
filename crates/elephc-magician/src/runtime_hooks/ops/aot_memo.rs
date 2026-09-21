//! Purpose:
//! Memoizes the generated runtime's AOT reflection lookups, which are linear table scans.
//!
//! Called from:
//! - `super::reflection`, around each `__elephc_eval_reflection_*` bridge call.
//!
//! Key details:
//! - Only compile-time-constant answers are cached; nothing here observes runtime state.

use std::cell::RefCell;
use std::collections::HashMap;
use std::thread::LocalKey;

/// One memo table: a case-folded `class\0member` key to the answer the generated table gave.
pub(super) type MemoTable<V> = RefCell<HashMap<String, V>>;

/// Declares a thread-local memo table for one reflection query.
macro_rules! memo_table {
    ($name:ident, $value:ty) => {
        thread_local! {
            pub(super) static $name: MemoTable<$value> = RefCell::new(HashMap::new());
        }
    };
}

memo_table!(METHOD_FLAGS, Option<u64>);
memo_table!(METHOD_DECLARING_CLASS, Option<String>);
memo_table!(PROPERTY_FLAGS, Option<u64>);
memo_table!(PROPERTY_DECLARING_CLASS, Option<String>);
memo_table!(CLASS_FLAGS, Option<u64>);
memo_table!(CANONICAL_CLASS_NAME, Option<String>);
// Whole member-name lists, keyed by class and an `AotMemberNameKind` tag. The value is owned Rust
// strings copied out of the generated array: no runtime handle and no arena pointer survives here,
// so nothing in this table can outlive -- or alias -- the request storage the `--web` boundary
// wipes.
memo_table!(MEMBER_NAMES, Vec<String>);

/// Builds the case-folded lookup key, because PHP resolves both halves case-insensitively.
///
/// The NUL separator cannot occur in a PHP class or member name, so `A\0b` and `A\0` + `b`
/// are the only way two distinct pairs could collide, and neither is expressible.
fn memo_key(class_name: &str, member_name: &str) -> String {
    let mut key = String::with_capacity(class_name.len() + member_name.len() + 1);
    key.extend(class_name.chars().map(|c| c.to_ascii_lowercase()));
    key.push('\0');
    key.extend(member_name.chars().map(|c| c.to_ascii_lowercase()));
    key
}

/// Whether the memo is active, so one binary can be measured with it and without it.
///
/// A memo that removes work can still lose, and comparing two *builds* cannot show that: the
/// generated program differs between them, which is how a 25% win first read as a 2x regression
/// here. One binary answering both ways is the only clean A/B, and it stays for the next one --
/// `ELEPHC_AOT_REFLECTION_MEMO=0` restores the unmemoized lookups.
fn memo_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        !matches!(
            std::env::var("ELEPHC_AOT_REFLECTION_MEMO").as_deref(),
            Ok("0") | Ok("off") | Ok("false")
        )
    })
}

/// Answers one AOT reflection query from the memo, calling the generated table at most once.
///
/// Every `__elephc_eval_reflection_*` entry point walks its metadata table row by row and calls
/// `__rt_strcasecmp` on each candidate. The table is a compile-time constant -- one row per
/// method or property the binary carries -- so a whole-framework build turns every query into a
/// scan of tens of thousands of rows. Profiling a Symfony `--web` worker put **39% of a rendered
/// request** inside those five scans and the case-insensitive compares they drive.
///
/// The tables cannot change while the process runs: they live in read-only generated data, and
/// the bridge entry points read nothing else. That makes each query a pure function of its two
/// names, so the first answer is the only answer, and it is kept for the life of the thread.
/// Classes declared at runtime are absent from the tables and stay absent, so a cached miss is
/// as durable as a cached hit -- the callers that must also see runtime classes consult runtime
/// state themselves, before or after asking here.
///
/// Errors are never cached: an `EvalStatus` reports the state of the *call*, not of the table.
pub(super) fn memoized<V, E>(
    cache: &'static LocalKey<MemoTable<V>>,
    class_name: &str,
    member_name: &str,
    compute: impl FnOnce() -> Result<V, E>,
) -> Result<V, E>
where
    V: Clone,
{
    if !memo_enabled() {
        return compute();
    }
    let key = memo_key(class_name, member_name);
    // The borrow ends before `compute` runs: the bridge call below re-enters this crate, and a
    // borrow held across it would be a panic rather than a miss.
    if let Some(hit) = cache.with(|table| table.borrow().get(&key).cloned()) {
        return Ok(hit);
    }
    let value = compute()?;
    cache.with(|table| {
        table.borrow_mut().insert(key, value.clone());
    });
    Ok(value)
}
