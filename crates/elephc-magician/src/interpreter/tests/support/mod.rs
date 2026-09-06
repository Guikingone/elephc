//! Purpose:
//! Shared fake runtime support for interpreter unit tests.
//! The fixtures allocate opaque runtime cells and implement `RuntimeValueOps`
//! without linking generated runtime hooks.
//!
//! Called from:
//! - `crate::interpreter::tests::*` focused test modules.
//!
//! Key details:
//! - Fake handles are stable integer-backed pointers used only inside tests.
//! - Output, warnings, and releases are recorded for assertions.

use std::collections::HashMap;
use std::ffi::c_void;

use crate::value::RuntimeCell;

use super::super::*;

mod array_ops;
mod cell_ops;
mod conversions;
mod lifecycle_ops;
mod numeric_ops;
mod object_ops;
mod runtime_ops;

/// Test-only array key representation for fake indexed and associative arrays.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum FakeKey {
    Int(i64),
    String(String),
}

/// Test-only runtime value representation used behind opaque cell handles.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum FakeValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<RuntimeCellHandle>),
    Assoc(Vec<(FakeKey, RuntimeCellHandle)>),
    Object(Vec<(String, RuntimeCellHandle)>),
    Iterator { len: i64, position: i64 },
    Resource(i64),
    InvokerRefCell(usize),
}

/// First PHP resource id the fake registry mints, matching the runtime's cursor.
///
/// `elephc::codegen_support::runtime::resource_ids` starts `_resource_id_next` at
/// 5 because PHP 8.5.6 reserves 1, 2 and 3 for the standard streams and 4 for the
/// CLI's own handle; the first `fopen()` in a program reports 5.
pub(super) const FAKE_FIRST_RESOURCE_ID: i64 = 5;

/// Highest payload the fake registry answers as `payload + 1` without minting.
///
/// Mirrors `STD_STREAM_MAX_PAYLOAD` in the runtime emitter: descriptors 0, 1 and 2
/// are STDIN/STDOUT/STDERR and PHP fixes their ids at 1, 2 and 3.
pub(super) const FAKE_STD_STREAM_MAX_PAYLOAD: i64 = 2;

/// Test runtime hooks that allocate stable fake handles and record echo output.
#[derive(Default)]
pub(super) struct FakeOps {
    pub(super) next_id: usize,
    pub(super) values: HashMap<usize, FakeValue>,
    /// Fake mirror of the runtime resource-id registry: native payload to PHP id.
    ///
    /// WHY THIS EXISTS. The fake used to answer `payload + 1` for every resource
    /// coercion, which was the pre-registry runtime invariant and stopped being
    /// true when `resource_ids.rs` landed. That made every test here structurally
    /// blind to resource numbering: eval hands out payloads from a counter of its
    /// own, so `payload + 1` produced a plausible small consecutive sequence no
    /// matter what the real runtime would have printed. Modelling the registry —
    /// bind-if-absent at creation, standard-stream shortcut, ids never reused —
    /// makes a fake coercion agree with what a compiled program actually prints.
    pub(super) resource_ids: HashMap<i64, i64>,
    /// Next never-used PHP resource id, lazily initialized to `FAKE_FIRST_RESOURCE_ID`.
    pub(super) resource_id_next: i64,
    /// Resource payloads that must NEVER receive a PHP resource id.
    ///
    /// Fake mirror of resource KIND 5, the eval-owned inert hash context. PHP 8's
    /// `hash_init()` returns a `HashContext` OBJECT, so it draws from the object-handle
    /// space and consumes nothing from the resource counter; the real runtime models
    /// this by having `__rt_mixed_from_value` skip `__rt_resource_id_of` for kind 5.
    /// Without this set the fake would bind an id for every hash context — `alloc`
    /// binds one for EVERY `FakeValue::Resource` — and every magician test would stay
    /// structurally unable to observe the very bug this models, because the fake's
    /// counter would keep advancing exactly as the buggy runtime's did.
    pub(super) inert_resources: std::collections::HashSet<i64>,
    pub(super) object_classes: HashMap<usize, String>,
    /// Live reference count per fake cell, so a release can tell FINAL from one of several.
    ///
    /// WHY THIS EXISTS. The fake used to answer "final" for every object release and to make
    /// `retain` a no-op, which left every ownership question structurally unaskable here: a test
    /// could be green while asserting the opposite of PHP, and a genuine refcount fix looked like
    /// a no-op. Counting makes the three behaviours that depend on a second live reference
    /// observable — an `IteratorAggregate` temporary dying with `getIterator()`, a generator
    /// method's receiver outliving the call that built it, and any release of a cell the releaser
    /// never owned.
    pub(super) refcounts: HashMap<usize, i64>,
    /// Whether a release that drives a count below zero fails the test instead of being recorded.
    ///
    /// Off by default so the whole suite does not have to be corrected in one step. A test opts
    /// in with `count_references()`, and the modules that are ABOUT ownership run counted.
    ///
    /// `ELEPHC_FAKE_COUNT_REFERENCES` turns it on for every fixture at once, which is how a module
    /// is surveyed before its own tests opt in.
    ///
    /// Six tests still over-release under that survey and are left uncounted deliberately, one
    /// family at a time rather than silenced:
    /// `builtins_arrays_iterators::{execute_program_dispatches_iterator_apply_object_builtin,
    /// execute_program_iterator_apply_dispatches_object_method_array}`,
    /// `builtins_class_metadata::property_values::execute_program_reflects_eval_parameter_declaring_class`,
    /// `classes::basics::execute_program_supports_legacy_var_properties`,
    /// `classes::promoted_references::execute_program_aliases_by_reference_promoted_static_and_nested_properties`,
    /// and `core::execute_context_function_persists_static_local_inside_catch`.
    pub(super) counted_mode: bool,
    /// Releases that drove a count below zero, recorded whether or not counting is enforced.
    pub(super) over_releases: Vec<FakeOverRelease>,
    pub(super) output: String,
    pub(super) releases: Vec<RuntimeCellHandle>,
    pub(super) warnings: Vec<String>,
    pub(super) fail_array_set_call: Option<usize>,
    pub(super) array_set_calls: usize,
    pub(super) ob_stack: Vec<FakeObLevel>,
    pub(super) ob_implicit_flush: bool,
}

/// One release that took a fake cell's reference count below zero.
///
/// The count and the value's shape are both recorded because "handle 41 went to -1" alone does
/// not say what was released; the shape is usually enough to recognize the site.
#[derive(Clone, Debug)]
pub(super) struct FakeOverRelease {
    pub(super) handle: usize,
    pub(super) count_after: i64,
    pub(super) value: String,
}

/// One fake output-buffer level: captured text plus the ob_start metadata the
/// status/introspection builtins report.
#[derive(Default)]
pub(super) struct FakeObLevel {
    pub(super) buffer: String,
    pub(super) name: String,
    pub(super) chunk_size: i64,
    pub(super) flags: i64,
}

impl FakeOps {
    /// Allocates one fake runtime cell and returns its opaque handle.
    ///
    /// This is the fake's counterpart of `__rt_mixed_from_value`, so it is also
    /// where a resource payload acquires its PHP id: boxing is the one point every
    /// resource passes through in the real runtime, and binding here reproduces
    /// both halves of that contract — creation order decides the id, and re-boxing
    /// a payload that already has one (`$b = $a`) keeps it.
    pub(super) fn alloc(&mut self, value: FakeValue) -> RuntimeCellHandle {
        if let FakeValue::Resource(payload) = value {
            self.bind_resource_id(payload);
        }
        self.next_id += 1;
        let id = self.next_id;
        self.values.insert(id, value);
        self.refcounts.insert(id, 1);
        RuntimeCellHandle::from_raw(id as *mut RuntimeCell)
    }

    /// Turns on reference counting for this fixture.
    ///
    /// A release that drives a count below zero then FAILS the test, naming the handle and the
    /// value it held. That is the point: a path that gives back a cell it never owned is exactly
    /// what silently destroyed live objects, and each one has to surface on its own.
    pub(super) fn count_references(&mut self) {
        self.counted_mode = true;
    }

    /// Returns whether this fixture enforces reference counting.
    ///
    /// `ELEPHC_FAKE_COUNT_REFERENCES` turns it on for every fixture, which is how a module is
    /// SURVEYED before its own tests opt in: the failures name the releasing sites, and the
    /// module is switched over once they are fixed.
    pub(super) fn counting_enforced(&self) -> bool {
        self.counted_mode || std::env::var_os("ELEPHC_FAKE_COUNT_REFERENCES").is_some()
    }

    /// Returns one fake cell's live reference count.
    pub(super) fn refcount(&self, value: RuntimeCellHandle) -> i64 {
        self.refcounts
            .get(&(value.as_ptr() as usize))
            .copied()
            .unwrap_or(0)
    }

    /// Returns every release that took a count below zero, in the order they happened.
    pub(super) fn over_releases(&self) -> &[FakeOverRelease] {
        &self.over_releases
    }

    /// Adds `delta` to one cell's count and reports the result.
    pub(super) fn adjust_refcount(&mut self, value: RuntimeCellHandle, delta: i64) -> i64 {
        let id = value.as_ptr() as usize;
        let count = self.refcounts.entry(id).or_insert(0);
        *count += delta;
        *count
    }

    /// Records a payload as inert, so no PHP resource id is ever bound to it.
    ///
    /// The fake counterpart of boxing with resource kind 5. Must be called BEFORE the
    /// `alloc` that creates the cell, because `alloc` is where binding happens.
    pub(super) fn mark_resource_inert(&mut self, payload: i64) {
        self.inert_resources.insert(payload);
    }

    /// Binds a fresh PHP resource id to a native payload that does not have one.
    ///
    /// Mirrors `__rt_resource_id_of`: the standard stream descriptors answer
    /// `payload + 1` without consuming the counter, every other payload takes the
    /// next never-used id, and ids are never reused. Inert payloads (kind 5, the eval
    /// hash context) are skipped entirely, which is what keeps `hash_init()` from
    /// shifting the ids of resources created after it.
    fn bind_resource_id(&mut self, payload: i64) {
        if self.inert_resources.contains(&payload) {
            return;
        }
        if payload <= FAKE_STD_STREAM_MAX_PAYLOAD || self.resource_ids.contains_key(&payload) {
            return;
        }
        if self.resource_id_next == 0 {
            self.resource_id_next = FAKE_FIRST_RESOURCE_ID;
        }
        self.resource_ids.insert(payload, self.resource_id_next);
        self.resource_id_next += 1;
    }

    /// Returns the PHP resource id bound to a native payload.
    ///
    /// Panics rather than falling back to arithmetic on the payload: an unbound
    /// payload means a resource cell reached a display path without going through
    /// `alloc`, and a silent fallback here is precisely the drift that made the
    /// old `payload + 1` model look correct.
    ///
    /// AN INERT PAYLOAD PANICS TOO, and deliberately with a different message. The
    /// real runtime does NOT panic there: `__rt_resource_id_of` still mints lazily for
    /// a kind-5 cell that reaches a display path, which is what guarantees no path can
    /// ever print a raw address. The fake cannot do that because this method — and all
    /// three of its callers in `super::conversions` — take `&self`. Widening them to
    /// `&mut self` is the honest fix if a test ever needs it; a silent fallback here
    /// would re-introduce exactly the blindness this fake was rewritten to remove.
    ///
    /// A NEGATIVE PAYLOAD IS A CLOSED HANDLE AND ANSWERS `-payload`, mirroring the
    /// `tbnz x0, #63` / `js` arm the real helper grew (`resource_ids.rs`) when `fclose`,
    /// `pclose` and `closedir` started stamping `-id` into the box. Without this arm the
    /// fake fell straight into the standard-stream shortcut below, because every negative
    /// value is `<= 2`: a `FakeValue::Resource(-5)` cell answered `-4`, so a unit test for
    /// the closed-resource display path would have asserted
    /// `resource(-4) of type (Unknown)` and passed.
    pub(super) fn fake_resource_id(&self, payload: i64) -> i64 {
        if self.inert_resources.contains(&payload) {
            panic!(
                "inert resource payload {payload} reached a display path; the runtime \
                 mints lazily here, so make `fake_resource_id` take `&mut self` and \
                 bind on demand rather than adding a fallback"
            );
        }
        if payload < 0 {
            return -payload;
        }
        if payload <= FAKE_STD_STREAM_MAX_PAYLOAD {
            return payload + 1;
        }
        *self
            .resource_ids
            .get(&payload)
            .expect("fake resource payload was never bound to an id")
    }

    /// Reads a fake runtime cell by opaque handle.
    pub(super) fn get(&self, handle: RuntimeCellHandle) -> FakeValue {
        let id = handle.as_ptr() as usize;
        self.values.get(&id).cloned().expect("fake cell missing")
    }

    /// Converts a fake runtime cell into a normalized fake PHP array key.
    pub(super) fn key(&self, handle: RuntimeCellHandle) -> Result<FakeKey, EvalStatus> {
        let value = self.get(handle);
        match value {
            FakeValue::Int(value) => Ok(FakeKey::Int(value)),
            FakeValue::String(value) => eval_numeric_string_array_key(value.as_bytes())
                .map(FakeKey::Int)
                .map_or_else(|| Ok(FakeKey::String(value)), Ok),
            FakeValue::Bytes(value) => eval_numeric_string_array_key(&value)
                .map(FakeKey::Int)
                .map_or_else(
                    || {
                        Ok(FakeKey::String(
                            String::from_utf8_lossy(&value).into_owned(),
                        ))
                    },
                    Ok,
                ),
            FakeValue::Null => Ok(FakeKey::String(String::new())),
            value => Ok(FakeKey::Int(self.fake_int(&value))),
        }
    }

    /// Allocates a fake runtime cell for an existing PHP array key.
    pub(super) fn alloc_key(&mut self, key: &FakeKey) -> Result<RuntimeCellHandle, EvalStatus> {
        match key {
            FakeKey::Int(value) => self.int(*value),
            FakeKey::String(value) => self.string(value),
        }
    }

    /// Finds a fake object property by insertion-order name.
    pub(super) fn object_property(
        properties: &[(String, RuntimeCellHandle)],
        name: &str,
    ) -> Option<RuntimeCellHandle> {
        properties
            .iter()
            .find_map(|(property, value)| (property == name).then_some(*value))
    }

    /// Configures one fake array-set call to fail for cleanup-path tests.
    pub(super) fn fail_array_set_call(&mut self, call_index: usize) {
        self.fail_array_set_call = Some(call_index);
        self.array_set_calls = 0;
    }
}

/// Verifies the fixture counts references, so a release with another reference live is not final.
///
/// This is what the fixture could not answer before: `final_object_identity_for_release` said
/// "final" for every object, which made a destructor fire while a second reference was still
/// live and made a real refcount fix look like a no-op.
#[test]
fn a_second_reference_makes_a_release_not_final() {
    let mut values = FakeOps::default();
    values.count_references();
    let object = values.alloc(FakeValue::Object(Vec::new()));
    values.object_classes.insert(object.as_ptr() as usize, "C".to_string());
    assert_eq!(values.refcount(object), 1);

    let second = values.retain(object).expect("retain the fake object");
    assert_eq!(values.refcount(object), 2);
    assert_eq!(
        values
            .final_object_identity_for_release(second)
            .expect("ask whether the release is final"),
        None,
        "a release with another reference live must not be final",
    );

    values.release(second).expect("give the second reference back");
    assert_eq!(values.refcount(object), 1);
    assert!(
        values
            .final_object_identity_for_release(object)
            .expect("ask whether the last release is final")
            .is_some(),
        "the last release must be final",
    );
}

/// Verifies an over-release is recorded even when counting is not enforced.
///
/// The record is what lets a whole module be surveyed before any of it is enforced.
#[test]
fn an_over_release_is_recorded_without_being_enforced() {
    // The survey switch enforces counting for every fixture, which is the opposite of what this
    // test is about, so it has nothing to check while that switch is on.
    if std::env::var_os("ELEPHC_FAKE_COUNT_REFERENCES").is_some() {
        return;
    }
    let mut values = FakeOps::default();
    let cell = values.alloc(FakeValue::Int(7));
    values.release(cell).expect("give the only reference back");
    assert!(values.over_releases().is_empty());

    values.release(cell).expect("release a cell nobody owns");
    let recorded = values.over_releases();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].handle, cell.as_ptr() as usize);
    assert_eq!(recorded[0].count_after, -1);
    assert_eq!(recorded[0].value, "Int(7)");
}

/// Test native invoker that returns the descriptor pointer as a runtime cell.
pub(super) unsafe extern "C" fn fake_native_return_descriptor(
    descriptor: *mut c_void,
    _args: *mut RuntimeCell,
) -> *mut RuntimeCell {
    descriptor.cast()
}
