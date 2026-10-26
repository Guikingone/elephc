//! Purpose:
//! Defines the core `PhpType` model used throughout checking and code generation.
//! Captures PHP scalar, compound, object, callable, FFI, pointer, and internal runtime shapes.
//!
//! Called from:
//! - `crate::types::checker`
//! - `crate::codegen`
//!
//! Key details:
//! - Internal types such as Mixed and runtime resources encode codegen/runtime contracts, not just PHP syntax.

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Callable used in match arms, constructed when closures are added
/// PHP runtime type.
pub enum PhpType {
    Int,
    Float,
    Str,
    Bool,
    /// The PHP literal `false` subtype. Runtime representation is identical to `Bool`.
    False,
    Void,
    Never,
    Iterable,
    Mixed,
    Array(Box<PhpType>),
    AssocArray {
        key: Box<PhpType>,
        value: Box<PhpType>,
    },
    Buffer(Box<PhpType>),
    Callable,
    Object(String),
    Packed(String),
    Pointer(Option<String>), // None = opaque ptr, Some("Class") = typed ptr<Class>
    Resource(Option<String>), // None = generic resource, Some("stream") = file/stdio stream
    Union(Vec<PhpType>),
    /// Codegen-internal inline nullable scalar: two words `{payload, tag}` with no heap
    /// allocation. The tag reuses the runtime value tag scheme (0 = int, 8 = null), so the
    /// pair is word-compatible with a boxed Mixed cell. The checker never produces this
    /// type; codegen funnels construct it from `int|null` unions only under
    /// `NullRepr::Tagged`. Under the legacy sentinel opt-out it never exists.
    TaggedScalar,
}

impl PhpType {
    /// Returns true when this nominal object type denotes PHP's built-in `Closure` class.
    ///
    /// Closure values use Elephc's callable descriptor representation even when flow
    /// narrowing reaches them through an `instanceof Closure` object guard.
    pub fn is_closure_object(&self) -> bool {
        matches!(
            self,
            PhpType::Object(name)
                if name.trim_start_matches('\\').eq_ignore_ascii_case("Closure")
        )
    }

    /// Returns whether a runtime property name of this type converts to a string the way php's
    /// `$object->{$name}` does: strings and integers directly, `null`, booleans and floats through
    /// their string form, and a union only when every member does. Arrays, objects and resources
    /// are rejected, as php raises on them.
    pub(crate) fn is_dynamic_property_name(&self) -> bool {
        match self {
            PhpType::Str
            | PhpType::Int
            | PhpType::Mixed
            | PhpType::Void
            // The element of an empty-array literal (`foreach ([] as $column)`): the body is
            // unreachable at run time, so any name it uses is vacuously valid.
            | PhpType::Never
            | PhpType::Bool
            | PhpType::False
            | PhpType::Float
            | PhpType::TaggedScalar => true,
            PhpType::Union(members) => members.iter().all(PhpType::is_dynamic_property_name),
            _ => false,
        }
    }

    /// Represents PHP's unrestricted array contract with a boxed packed-or-hash payload.
    /// Unlike a concrete storage type, this declaration cannot narrow to one call site's layout.
    pub(crate) fn php_array() -> Self {
        Self::Union(vec![
            Self::Array(Box::new(Self::Mixed)),
            Self::AssocArray { key: Box::new(Self::Mixed), value: Box::new(Self::Mixed) },
        ])
    }

    /// Identifies the exact non-null PHP array contract, not a union that also permits scalars.
    /// Returns the representation a `new static` candidate's parameter is compared with.
    ///
    /// A declared `array` is the boxed packed-or-hash union, whose representation is `Mixed`; it
    /// still holds only arrays, so it is compared as one. Otherwise
    /// `HttpException::fromStatusCode()` planned `new static($statusCode, ...)` against
    /// `MethodNotAllowedHttpException(array $allow, ...)` and refused the int.
    pub(crate) fn late_bound_param_storage(&self) -> PhpType {
        if self.is_php_array() {
            PhpType::Array(Box::new(PhpType::Mixed))
        } else {
            self.codegen_repr()
        }
    }

    pub(crate) fn is_php_array(&self) -> bool {
        matches!(self, Self::Union(members) if members.len() == 2
            && members.iter().any(|member| matches!(member, Self::Array(value) if **value == Self::Mixed))
            && members.iter().any(|member| matches!(member, Self::AssocArray { key, value }
                if **key == Self::Mixed && **value == Self::Mixed)))
    }

    /// Returns a `PhpType::Resource(Some("stream"))` representing a stream resource.
    pub fn stream_resource() -> PhpType {
        PhpType::Resource(Some("stream".to_string()))
    }

    /// Returns true if `expected` is compatible with `actual` for resource type matching.
    /// A typed resource (Some) is compatible with a generic resource (None), and two typed
    /// resources are compatible when their kind strings match.
    pub fn resource_types_compatible(expected: &PhpType, actual: &PhpType) -> bool {
        match (expected, actual) {
            (PhpType::Resource(None), PhpType::Resource(_))
            | (PhpType::Resource(_), PhpType::Resource(None)) => true,
            (PhpType::Resource(Some(expected)), PhpType::Resource(Some(actual))) => {
                expected == actual
            }
            _ => false,
        }
    }

    /// Returns true when a null property default must be materialized into a slot of
    /// this type (and the literal-default emitters support doing so).
    ///
    /// `Void`, nullable unions, `Mixed`, and object slots encode null distinctly, so the
    /// default write is required and supported. Every other slot either has no null
    /// encoding (plain scalars, strings, arrays — those slots are always overwritten
    /// before an observable read when refinement rebound them) or reads zero-initialized
    /// storage as null already (callable/pointer-shaped slots), so the null default is
    /// skipped for them.
    pub fn null_property_default_required(&self) -> bool {
        match self {
            PhpType::Void | PhpType::Mixed | PhpType::TaggedScalar | PhpType::Object(_) => true,
            PhpType::Union(members) => members.iter().any(|member| matches!(member, PhpType::Void)),
            PhpType::Int
            | PhpType::Float
            | PhpType::Str
            | PhpType::Bool
            | PhpType::False
            | PhpType::Never
            | PhpType::Iterable
            | PhpType::Array(_)
            | PhpType::AssocArray { .. }
            | PhpType::Buffer(_)
            | PhpType::Callable
            | PhpType::Packed(_)
            | PhpType::Pointer(_)
            | PhpType::Resource(_) => false,
        }
    }

    /// True for the type of an EMPTY array literal — an array whose element type is the
    /// `Never` placeholder, meaning "nothing has ever been stored here".
    ///
    /// The distinction matters wherever storage OUTLIVES the expression that typed it. For a
    /// plain local, `$a = []; $a[] = 'x';` is flow-sensitive and the second statement retypes
    /// the binding, so `array<never>` is only ever read where it is true. For a `static` local
    /// the initializer runs on the FIRST call only, while the declaration re-types the name on
    /// EVERY call — so `array<never>` there is a claim about what an earlier call left behind,
    /// which the declaration cannot know. `array<never>`'s element slots are zero width, and
    /// codegen trusts that: `array_shift`/`array_pop` lower "read the removed payload" to
    /// `mov x11, #0`, producing a silent `NULL` where php returns the element.
    pub fn is_empty_array_literal(&self) -> bool {
        match self.codegen_repr() {
            PhpType::Array(elem) => matches!(elem.codegen_repr(), PhpType::Never | PhpType::Void),
            PhpType::AssocArray { value, .. } => {
                matches!(value.codegen_repr(), PhpType::Never | PhpType::Void)
            }
            _ => false,
        }
    }

    /// The storage an empty-array literal must take when it seeds a slot that outlives the call.
    ///
    /// BOXED ELEMENTS, NOT A BOXED SLOT. Widening the whole slot to `Mixed` would put a boxed
    /// cell in the static's symbol, and a `mixed &$param` argument is then handed that cell as
    /// its reference — which the callee's `store_ref_cell` releases and replaces without the
    /// symbol ever learning, turning `sort($staticQueue)` from a stale answer into a DESTROYED
    /// one. `array<mixed>` keeps the slot a plain container pointer, which is the representation
    /// `Op::ArrayToMixed` already produces for a static (`local_slot_kind_is_convertible` lists
    /// `LocalKind::StaticLocal`) and which `static_local_value_type_matches` already accepts an
    /// `Array(Never)` initializer for.
    pub fn gradual_empty_array_storage() -> PhpType {
        PhpType::Array(Box::new(PhpType::Mixed))
    }

    /// Size in bytes on the stack.
    pub fn stack_size(&self) -> usize {
        match self {
            PhpType::Bool | PhpType::False => 8,
            PhpType::Int => 8,
            PhpType::Float => 8,
            PhpType::Str => 16,
            PhpType::Void => 8,              // null sentinel stored as 8 bytes
            PhpType::Never => 0,             // never materialized; functions with :never do not return
            PhpType::Iterable => 8,          // type-erased pointer (array|Traversable)
            PhpType::Mixed => 8,             // pointer to heap-tagged mixed cell
            PhpType::Array(_) => 8,          // pointer to heap
            PhpType::AssocArray { .. } => 8, // pointer to heap
            PhpType::Buffer(_) => 8,         // pointer to buffer header
            PhpType::Callable => 8,          // callable descriptor address
            PhpType::Object(_) => 8,         // pointer to heap
            PhpType::Packed(_) => 8,         // metadata-only nominal type, usually accessed by pointer
            PhpType::Pointer(_) => 8,        // 64-bit address
            PhpType::Resource(_) => 8,       // runtime resource id / native handle
            PhpType::Union(_) => 8,          // boxed runtime-tagged payload (same storage as Mixed)
            PhpType::TaggedScalar => 16,     // inline nullable scalar: payload word + tag word
        }
    }

    /// Number of registers used to pass this type as an argument.
    pub fn register_count(&self) -> usize {
        match self {
            PhpType::Bool | PhpType::False => 1,
            PhpType::Int => 1,
            PhpType::Float => 1,
            PhpType::Str => 2,
            PhpType::Void => 0,
            PhpType::Never => 0,
            PhpType::Iterable => 1,
            PhpType::Mixed => 1,
            PhpType::Array(_) => 1,
            PhpType::AssocArray { .. } => 1,
            PhpType::Buffer(_) => 1,
            PhpType::Callable => 1,
            PhpType::Object(_) => 1,
            PhpType::Packed(_) => 1,
            PhpType::Pointer(_) => 1,
            PhpType::Resource(_) => 1,
            PhpType::Union(_) => 1,
            PhpType::TaggedScalar => 2,
        }
    }

    /// Returns true if this type uses a floating-point register (d0-d7).
    pub fn is_float_reg(&self) -> bool {
        matches!(self, PhpType::Float)
    }

    /// Returns true for heap values whose lifetime is tracked with runtime refcounts.
    pub fn is_refcounted(&self) -> bool {
        matches!(
            self,
            PhpType::Iterable
                | PhpType::Mixed
                | PhpType::Array(_)
                | PhpType::AssocArray { .. }
                | PhpType::Object(_)
                | PhpType::Union(_)
        ) && !self.is_closure_object()
    }

    /// Lower high-level checker-only types to the runtime representation used by codegen.
    /// `Iterable` keeps its own runtime shape (raw heap pointer dispatched via the heap-kind tag),
    /// so it is no longer collapsed to `Mixed` here.
    pub fn codegen_repr(&self) -> PhpType {
        match self {
            PhpType::Union(members)
                if crate::codegen::sentinels::null_repr_is_tagged()
                    && nullable_int_union_members(members) =>
            {
                PhpType::TaggedScalar
            }
            PhpType::Union(_) => PhpType::Mixed,
            PhpType::False => PhpType::Bool,
            PhpType::Resource(_) => PhpType::Int,
            PhpType::Object(_) if self.is_closure_object() => PhpType::Callable,
            PhpType::Never => PhpType::Void, // never should not be materialized; fallback to void sentinel
            _ => self.clone(),
        }
    }

    /// Returns whether a reference cell can be read with either payload representation.
    ///
    /// Object class names do not change pointer storage; semantic return compatibility is
    /// checked separately. Container element layouts still matter even when both outer
    /// values are pointers, so comparing only their EIR heap kind would be too permissive.
    pub(crate) fn reference_payload_compatible(&self, expected: &PhpType) -> bool {
        match (self.codegen_repr(), expected.codegen_repr()) {
            (PhpType::Object(_), PhpType::Object(_)) => true,
            (PhpType::Array(actual), PhpType::Array(expected)) => {
                actual.reference_payload_compatible(&expected)
            }
            (
                PhpType::AssocArray { key: actual_key, value: actual_value },
                PhpType::AssocArray { key: expected_key, value: expected_value },
            ) => actual_key.reference_payload_compatible(&expected_key)
                && actual_value.reference_payload_compatible(&expected_value),
            (actual, expected) => actual == expected,
        }
    }

    /// Returns true if this is an indexed array of a scalar (int/float/bool) element type.
    ///
    /// The hash-based builtins accept such indexed inputs by converting them to integer-keyed
    /// hashes; scalar elements are copied by value, so the converted temporaries are safe to
    /// free. String/heap element indexed inputs are a follow-up (they hit x86-specific converter
    /// and clone-shallow issues), so the checker restricts indexed inputs to scalar elements.
    pub fn is_scalar_indexed_array(&self) -> bool {
        matches!(
            self,
            PhpType::Array(elem)
                if matches!(**elem, PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False)
        )
    }

    /// Returns the hash key type this array type contributes: `Int` for an indexed array,
    /// the declared key for an associative array, `Int` otherwise.
    pub fn hash_key_type(&self) -> PhpType {
        match self {
            PhpType::Array(_) => PhpType::Int,
            PhpType::AssocArray { key, .. } => (**key).clone(),
            _ => PhpType::Int,
        }
    }

    /// Returns the hash value type this array type contributes: the element type for an indexed
    /// array, the declared value for an associative array, `Mixed` otherwise.
    pub fn hash_value_type(&self) -> PhpType {
        match self {
            PhpType::Array(elem) => (**elem).clone(),
            PhpType::AssocArray { value, .. } => (**value).clone(),
            _ => PhpType::Mixed,
        }
    }

    /// Widens two types to a common type: the type itself when both agree, else `Mixed`.
    pub fn widen(a: PhpType, b: PhpType) -> PhpType {
        if a == b {
            a
        } else {
            PhpType::Mixed
        }
    }

    /// Widens element types contributed by alternative array-producing branches.
    ///
    /// `Never` is the placeholder element type of an empty array literal, so it
    /// contributes no runtime value and defers to the populated branch. The
    /// `false` subtype and `bool` share the boolean contract; every other
    /// disagreement requires boxed `Mixed` element storage.
    pub fn widen_array_branch_element(a: PhpType, b: PhpType) -> PhpType {
        if a == b {
            return a;
        }
        if a == PhpType::Never {
            return b;
        }
        if b == PhpType::Never {
            return a;
        }
        if matches!(
            (&a, &b),
            (PhpType::Bool, PhpType::False) | (PhpType::False, PhpType::Bool)
        ) {
            return PhpType::Bool;
        }
        PhpType::Mixed
    }

    /// Computes the result hash type for a two-input hash builtin (the `array_replace` /
    /// `array_diff_assoc` family). The key and value each widen to `Mixed` when the two inputs
    /// disagree, so a `foreach` over the result performs the correct runtime key/value dispatch
    /// when an indexed input is mixed with a string-keyed associative input.
    pub fn two_input_hash_result(t1: &PhpType, t2: &PhpType) -> PhpType {
        PhpType::AssocArray {
            key: Box::new(PhpType::widen(t1.hash_key_type(), t2.hash_key_type())),
            value: Box::new(PhpType::widen(t1.hash_value_type(), t2.hash_value_type())),
        }
    }
}

/// Returns true for an `int|null` union that can use the inline tagged-scalar representation.
fn nullable_int_union_members(members: &[PhpType]) -> bool {
    let mut has_int = false;
    let mut has_null = false;
    for member in members {
        match member {
            PhpType::Int => has_int = true,
            PhpType::Void | PhpType::Never => has_null = true,
            _ => return false,
        }
    }
    has_int && has_null
}

impl fmt::Display for PhpType {
    /// Formats the type as a human-readable string using PHP-style syntax (e.g., `int`, `array<int>`,
    /// `resource<stream>`, `ptr<MyClass>`). Used for error messages and debug output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_php_array() {
            return write!(f, "array");
        }
        match self {
            PhpType::Int => write!(f, "int"),
            PhpType::Float => write!(f, "float"),
            PhpType::Str => write!(f, "string"),
            PhpType::Bool => write!(f, "bool"),
            PhpType::False => write!(f, "false"),
            PhpType::Void => write!(f, "null"),
            PhpType::Never => write!(f, "never"),
            PhpType::Iterable => write!(f, "iterable"),
            PhpType::Mixed => write!(f, "mixed"),
            PhpType::Array(inner) => write!(f, "array<{}>", inner),
            PhpType::AssocArray { key, value } => write!(f, "array<{}, {}>", key, value),
            PhpType::Buffer(inner) => write!(f, "buffer<{}>", inner),
            PhpType::Callable => write!(f, "callable"),
            PhpType::Object(name) => write!(f, "{}", name),
            PhpType::Packed(name) => write!(f, "packed {}", name),
            PhpType::Pointer(Some(name)) => write!(f, "ptr<{}>", name),
            PhpType::Pointer(None) => write!(f, "ptr"),
            PhpType::Resource(Some(kind)) => write!(f, "resource<{}>", kind),
            PhpType::Resource(None) => write!(f, "resource"),
            PhpType::TaggedScalar => write!(f, "int|null"),
            PhpType::Union(members) => {
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, "|")?;
                    }
                    write!(f, "{}", member)?;
                }
                Ok(())
            }
        }
    }
}

/// Maps variable names to their resolved types.
pub type TypeEnv = HashMap<String, PhpType>;
