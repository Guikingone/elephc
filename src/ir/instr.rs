//! Purpose:
//! Defines EIR instructions, opcodes, immediates, and instruction identifiers.
//!
//! Called from:
//! - `crate::ir::builder`, `crate::ir::validator`, `crate::ir::print`, and
//!   future lowering/codegen passes.
//!
//! Key details:
//! - Each opcode exposes a conservative default effect set. Call-like opcodes
//!   may be refined by builders once semantic metadata is available.

use crate::ir::effects::Effects;
use crate::ir::function::{FunctionId, LocalSlotId};
use crate::ir::module::DataId;
use crate::ir::runtime_call::RuntimeCallTarget;
use crate::ir::types::{IrHeapKind, IrType};
use crate::ir::value::{Ownership, ValueId};
use crate::span::Span;
use crate::types::PhpType;

/// Function-local identifier for an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstId(u32);

impl InstId {
    /// Creates an instruction identifier from its raw zero-based table index.
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw zero-based table index represented by this identifier.
    pub fn as_raw(self) -> u32 {
        self.0
    }
}

/// Instruction payload stored in a function-level instruction table.
#[derive(Debug, Clone)]
pub struct Instruction {
    pub op: Op,
    pub operands: Vec<ValueId>,
    pub immediate: Option<Immediate>,
    pub result: Option<ValueId>,
    pub result_type: IrType,
    pub result_php_type: PhpType,
    pub result_ownership: Ownership,
    pub effects: Effects,
    pub span: Option<Span>,
    /// Optimization-pass provenance: set when a pass rewrote this instruction
    /// (const-fold) or moved it (LICM), so source maps can explain assembly
    /// that no longer matches the source shape. `None` for instructions
    /// lowered directly from the AST. A one-byte enum rather than a string:
    /// `Instruction` sits in the recursive lowering paths' stack frames, and
    /// growing it measurably shrinks the headroom before test threads overflow.
    pub origin: Option<PassOrigin>,
}

/// Optimization pass recorded as an instruction's provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassOrigin {
    ConstFold,
    Licm,
}

impl PassOrigin {
    /// Returns the lower-case spelling used by source maps and the EIR printer.
    pub fn name(self) -> &'static str {
        match self {
            PassOrigin::ConstFold => "const_fold",
            PassOrigin::Licm => "licm",
        }
    }
}

impl Instruction {
    /// Creates a new instruction payload with all semantic metadata attached.
    pub fn new(
        op: Op,
        operands: Vec<ValueId>,
        immediate: Option<Immediate>,
        result: Option<ValueId>,
        result_type: IrType,
        result_php_type: PhpType,
        result_ownership: Ownership,
        effects: Effects,
        span: Option<Span>,
    ) -> Self {
        Self {
            op,
            operands,
            immediate,
            result,
            result_type,
            result_php_type,
            result_ownership,
            effects,
            span,
            origin: None,
        }
    }

    /// Returns true when this instruction has no SSA result value.
    pub fn is_void(&self) -> bool {
        self.result.is_none() || self.result_type.is_void()
    }
}

/// Literal or metadata operand attached to an opcode.
#[derive(Debug, Clone, PartialEq)]
pub enum Immediate {
    I64(i64),
    F64(f64),
    Bool(bool),
    Data(DataId),
    LocalSlot(LocalSlotId),
    LocalSlotPair {
        first: LocalSlotId,
        second: LocalSlotId,
    },
    GlobalName(DataId),
    FunctionRef(FunctionId),
    BuiltinRef(BuiltinId),
    RuntimeRef(RuntimeId),
    RuntimeCall(RuntimeCallTarget),
    ExternRef(u32),
    ClassRef(u32),
    EnumCaseRef {
        enum_id: u32,
        case_id: u32,
    },
    MethodRef {
        class: u32,
        method: u32,
    },
    PropertyRef {
        class: u32,
        property: u32,
    },
    FieldRef {
        layout: u32,
        field: u32,
    },
    FunctionVariantRef {
        group: u32,
        variant: u32,
    },
    HeapKind(IrHeapKind),
    MixedTag(u8),
    TypePredicate(PhpTypePredicate),
    MixedNumericOp(MixedNumericOp),
    StrBitOp(StrBitKind),
    CmpPredicate(CmpPredicate),
    CastTarget(IrType),
    TypeName(DataId),
    Capacity(u32),
    WidthBytes(u8),
}

/// Runtime arithmetic operation carried by `Op::MixedNumericBinop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MixedNumericOp {
    Add,
    Sub,
    Mul,
}

/// PHP runtime type category tested by the backend-neutral `TypePredicate` opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhpTypePredicate {
    Array,
    Bool,
    Float,
    Int,
    Iterable,
    Object,
    Resource,
    Scalar,
    String,
}

impl PhpTypePredicate {
    /// Returns the stable textual spelling used by the EIR printer.
    pub const fn as_eir(self) -> &'static str {
        match self {
            Self::Array => "array",
            Self::Bool => "bool",
            Self::Float => "float",
            Self::Int => "int",
            Self::Iterable => "iterable",
            Self::Object => "object",
            Self::Resource => "resource",
            Self::Scalar => "scalar",
            Self::String => "string",
        }
    }
}

impl MixedNumericOp {
    /// Returns the lower-case textual spelling used by the EIR printer.
    pub fn as_eir(self) -> &'static str {
        match self {
            MixedNumericOp::Add => "add",
            MixedNumericOp::Sub => "sub",
            MixedNumericOp::Mul => "mul",
        }
    }
}

/// PHP bytewise string operator carried by `Op::StrBitwise`.
///
/// PHP applies `&`/`|`/`^` bytewise when both operands are strings, producing a
/// string result rather than an integer. The variant is passed to
/// `__rt_str_bitwise` as a mode immediate so a single opcode and one runtime
/// helper cover all three operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrBitKind {
    And,
    Or,
    Xor,
}

impl StrBitKind {
    /// Returns the lower-case textual spelling used by the EIR printer.
    pub fn as_eir(self) -> &'static str {
        match self {
            StrBitKind::And => "and",
            StrBitKind::Or => "or",
            StrBitKind::Xor => "xor",
        }
    }

    /// Returns the numeric mode passed to `__rt_str_bitwise` (0=And, 1=Or, 2=Xor).
    ///
    /// The runtime helper (`src/codegen/runtime/strings/str_bitwise.rs`) branches on
    /// exactly this numbering, so the two must stay in lockstep.
    pub fn as_mode(self) -> i64 {
        match self {
            StrBitKind::And => 0,
            StrBitKind::Or => 1,
            StrBitKind::Xor => 2,
        }
    }
}

/// Comparison predicate for integer and floating-point compare opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmpPredicate {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
    Olt,
    Ole,
    Ogt,
    Oge,
}

/// Stable identifier for a builtin entry in the future IR metadata table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuiltinId(pub u32);

/// Stable identifier for a runtime helper entry in the future IR metadata table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeId(pub u32);

/// EIR opcode family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    ConstI64,
    ConstF64,
    ConstStr,
    ConstNull,
    ConstBool,
    ConstClassName,
    /// Resolves the concrete runtime class name for `$value::class`. A raw object operand reads
    /// its class id directly; a boxed `Mixed`/union operand is tag-checked and throws a catchable
    /// `TypeError` when its runtime value is not an object. Result: `Str`.
    ObjectClassName,
    ConstEnumCase,
    LoadCalledClassId,
    DataAddr,
    LoadLocal,
    StoreLocal,
    UnsetLocal,
    LoadRefCell,
    StoreRefCell,
    PromoteLocalRefCell,
    AliasLocalRefCell,
    ReleaseLocalRefCell,
    /// Binds a target local slot as an OWNING alias to a pre-existing external kind-6 refcounted
    /// reference cell: increfs the cell and registers a refcount-aware scope-exit release
    /// (`__rt_ref_cell_decref`). Operand: the cell pointer (SSA value); immediate:
    /// `LocalSlotPair { first: target slot, second: hidden owner slot }`. Distinct from
    /// `PromoteLocalRefCell` (which allocs + moves a value in) and `BindRefCellPtr` (non-owning).
    AdoptRefCell,
    /// Promotes a hash element to a kind-6 reference cell in place (`$x = &$arr[$k]`) and yields the
    /// shared cell pointer. Operands: the hash, then the key. Backed by `__rt_hash_ref_element`,
    /// which may relocate the hash; the backend writes the returned hash back to the array local.
    HashRefElement,
    /// Binds a hash element as a reference alias of an existing kind-6 cell
    /// (`self::$a[$dir] = &self::$a[$k]`) and yields the possibly-relocated hash. Operands: the
    /// hash, the key, then the cell pointer. Backed by `__rt_hash_bind_ref_element`, which increfs
    /// the cell, releases any prior value at the key, and writes the cell into `hash[key]` with
    /// value-tag 11 (Reference). Distinct from `HashRefElement` (which *produces* a cell from an
    /// element); this *consumes* a cell into an element.
    HashBindRefElement,
    /// Appends an EXISTING kind-6 reference cell as a new element at a hash's next automatic integer
    /// key (`$a[] = &$var`, `$a[$k][] = &$var`) and yields the possibly-relocated hash. Operands: the
    /// hash, then the shared cell pointer (`$var`'s persistent cell, from `LocalRefEnsure`). Backed by
    /// `__rt_hash_ref_append_element`, which increfs the cell (the new element owns a share) and
    /// appends it with value-tag 11 (Reference). The backend writes the relocated hash back to the
    /// array local. Distinct from `HashBindRefElement` (binds an existing cell at an EXPLICIT key) —
    /// this appends at the next int key. The cell is NOT freshly allocated: Zend keeps ONE cell per
    /// referenced variable, shared across every bind, so binding a fresh cell would diverge the alias.
    HashRefAppendElement,
    /// Get-or-promotes a local's PERSISTENT kind-6 reference cell for `&$var` and yields the cell
    /// pointer. Immediate: `LocalSlotPair { first: the visible local slot, second: the hidden owner
    /// slot }`; no operands. Backed by `__rt_ref_cell_ensure`: it reads the slot word, and when it is
    /// already a kind-6 cell reuses it (idempotent — a loop body's single `&$var` promotes on the
    /// first iteration and reuses thereafter), otherwise allocates a fresh cell that MOVES the value
    /// in. The backend stores the cell into both slots and marks the local a promoted ref-cell owner
    /// (so later reads/writes dereference it and scope-exit releases via `__rt_ref_cell_decref`).
    LocalRefEnsure,
    ReleaseLocalSlot,
    LoadGlobal,
    StoreGlobal,
    LoadStaticLocal,
    StoreStaticLocal,
    InitStaticLocal,
    /// Reads a static local's once-flag as a `Bool` without mutating it or touching the value
    /// slot. No operands. Immediate: the static local's slot. Emitted as the condition of the
    /// once-guard `CondBr` that `crate::ir_lower::stmt::lower_static_var` wraps around the whole
    /// initializer evaluation — the flag-true arm skips straight past `<init>`'s instructions
    /// (and `InitStaticLocal` itself) instead of only skipping the final store, so a
    /// side-effecting or heap-allocating `<init>` runs exactly once across calls. Contrast with
    /// `IncludeOnceGuard`, which marks its flag before running its guarded body (fine for
    /// include-cycle prevention); a static initializer must stay unmarked until `InitStaticLocal`
    /// finishes storing, so a reentrant call mid-`<init>` (e.g. `<init>` recursing into the same
    /// function) still observes "uninitialized" — matching PHP's own `static $x; $x ??= <init>;`
    /// reentrancy behavior (php-verified: nested calls each re-evaluate `<init>` independently;
    /// the outermost completed store wins last).
    StaticLocalInitialized,
    LoadStaticProperty,
    /// Loads a static property selected by a runtime name string (`self::${$expr}`).
    /// Operand: the runtime property-name (a `Str` value). Immediate: the receiver class name
    /// data id; codegen enumerates that class's declared static properties and dispatches on the
    /// runtime name via `__rt_str_eq`, loading the matching global symbol. An unmatched name
    /// fatals ("Access to undefined static property").
    LoadDynamicStaticProperty,
    StoreStaticProperty,
    /// Stores a value into a static property selected by a runtime name string
    /// (`self::${$expr} = v`). Operands: `[name, value]` — the runtime property-name (a `Str`)
    /// and the value. Immediate: the receiver class name data id; codegen enumerates that
    /// class's declared static properties, dispatches on the runtime name via `__rt_str_eq`, and
    /// stores into the matching global symbol (releasing the previous value like
    /// `StoreStaticProperty`). An unmatched name fatals ("Access to undefined static property").
    StoreDynamicStaticProperty,
    LoadReflectionStaticProperty,
    StoreReflectionStaticProperty,
    ReflectionStaticPropertyInitialized,
    IAdd,
    ISub,
    IMul,
    ICheckedAdd,
    ICheckedSub,
    ICheckedMul,
    IDiv,
    ISDiv,
    ISMod,
    IPow,
    INeg,
    IBitAnd,
    IBitOr,
    IBitXor,
    IBitNot,
    IShl,
    IShrA,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FPow,
    FNeg,
    MixedNumericBinop,
    ICmp,
    FCmp,
    StrEq,
    StrCmp,
    StrLooseEq,
    StrictEq,
    StrictNotEq,
    LooseEq,
    LooseNotEq,
    Spaceship,
    IsNull,
    IsTruthy,
    TypePredicate,
    IsEmpty,
    InstanceOf,
    IToF,
    FToI,
    IToStr,
    FToStr,
    BoolToStr,
    StrToI,
    StrToF,
    StrToNumber,
    ResourceToStr,
    Cast,
    /// PHP `(object)` cast. Takes a single boxed `Mixed` operand and produces a
    /// freshly allocated owned `stdClass`: arrays become property maps, scalars
    /// become a `scalar` property, `null` becomes an empty object, and an
    /// object payload is returned (retained) unchanged.
    ObjectCast,
    /// PHP `(array)` cast. Takes a single boxed `Mixed` operand and produces a
    /// freshly allocated owned boxed-Mixed `array<mixed>`: an indexed array is
    /// rebuilt element-by-element, a scalar becomes a single-element `[value]`
    /// array, `null` becomes an empty array, and associative-array/object payloads
    /// fatal (their string keys cannot fit the int-indexed result type).
    ArrayCast,
    MixedBox,
    InvokerRefArg,
    MixedUnbox,
    MixedTagOf,
    ArrayToMixed,
    HashToMixed,
    MixedCastBool,
    MixedCastInt,
    MixedCastFloat,
    MixedCastString,
    StrConcat,
    /// PHP bytewise string operator (`&`/`|`/`^` with two string operands).
    /// Carries an `Immediate::StrBitOp` mode; produces a concat-scratch string
    /// via `__rt_str_bitwise` (And/Xor → min length, Or → max length + tail copy).
    StrBitwise,
    /// PHP runtime-polymorphic bitwise operator (`&`/`|`/`^`) when at least one
    /// operand is a dynamic `Mixed`/union value and the other is a string or also
    /// dynamic, so the string-vs-integer choice can only be made at runtime.
    /// Carries an `Immediate::StrBitOp` mode; both operands are boxed to `Mixed`
    /// and `__rt_mixed_bitwise` dispatches: both strings → bytewise string result,
    /// array/object operand → TypeError fatal, otherwise integer bitwise. Produces
    /// a freshly boxed `Mixed` cell (int or string payload).
    MixedBitwise,
    /// PHP runtime-polymorphic unary bitwise NOT (`~$x`) on a dynamic `Mixed`/union operand
    /// whose runtime payload could be a string, so the string-vs-integer choice can only be
    /// made at runtime. The single operand is boxed to `Mixed` and `__rt_mixed_bitwise_not`
    /// dispatches: string → bytewise NOT string result (each byte `~b`), array/object operand →
    /// TypeError fatal, otherwise integer NOT (`~i`). Produces a freshly boxed `Mixed` cell.
    MixedBitwiseNot,
    StrLen,
    StrPersist,
    StrCharAt,
    /// PHP string offset assignment (`$s[$i] = $c`). Reads the source string, the
    /// integer byte offset, and the replacement string; writes the replacement's
    /// FIRST byte at `$i` (right-padding with spaces when `$i >= strlen`), and
    /// produces a fresh concat-scratch string. Never mutates the source in place,
    /// so aliases are copy-on-write safe. Negative offsets index from the end;
    /// out-of-range negatives and an empty replacement are no-ops (source copied
    /// through unchanged).
    StrOffsetSet,
    StrInterpolate,
    ConcatReset,
    WriteStrStdout,
    ArrayNew,
    HashNew,
    ArrayLen,
    HashLen,
    ArrayGet,
    ArrayGetSilent,
    HashGet,
    HashGetSilent,
    ArrayIsset,
    HashIsset,
    ArrayElemAddr,
    ArraySet,
    HashSet,
    HashUnset,
    ArrayPush,
    MixedArrayAppend,
    HashAppend,
    ArrayEnsureUnique,
    HashEnsureUnique,
    ArrayCloneShallow,
    HashCloneShallow,
    ArrayUnion,
    HashUnion,
    ArrayHashUnion,
    HashArrayUnion,
    HashSpread,
    ArrayToHash,
    /// Converts a boxed `Mixed`/union array into a freshly cloned owned hash at the
    /// gradual-typing boundary. The source array is never mutated (it is shallow-cloned
    /// for tag-5 hashes, or rebuilt for tag-4 indexed arrays); a non-array payload takes
    /// a runtime `TypeError` fatal. The single owned result is released by the consumer.
    MixedToHash,
    ArraySetMixedKey,
    ArrayGetMixedKey,
    ArrayGetMixedKeySilent,
    ArrayKeyExists,
    OffsetExists,
    OffsetUnset,
    ListUnpack,
    IterStart,
    IterCurrentKey,
    IterCurrentValue,
    IterCurrentValueRef,
    IterNext,
    IterEnd,
    IteratorMethodCall,
    SplRuntimeCall,
    ObjectNew,
    ObjectClone,
    EvalObjectNew,
    ObjectCloneShallow,
    DynamicObjectNew,
    DynamicObjectNewMixed,
    DynamicObjectNewWithoutConstructorMixed,
    PropGet,
    PropInitialized,
    PropSet,
    /// Loads the raw reference-cell pointer stored in a reference property's slot,
    /// without dereferencing it. Used to alias a local to `$obj->prop` and to return
    /// `$this->prop` by reference. Operand: object; immediate: property name data id.
    LoadPropRefCell,
    /// Loads the raw reference-cell pointer of a DYNAMIC-named reference property, without
    /// dereferencing it. Used to alias a local to `$x = &$obj->$name` (write-through). Operands:
    /// object, then the runtime property-name string; NO immediate. The receiver class's
    /// array-typed properties were promoted to reference properties by the checker, so codegen
    /// dispatches on the runtime name across those declared slots and yields the matching cell
    /// pointer. Same borrowed-cell result ownership as `LoadPropRefCell`.
    LoadDynamicPropRefCell,
    /// Loads the address of a static property's global storage as a ref-cell pointer,
    /// without dereferencing it. Used to alias a local to `$x = &self::$n` (write-through).
    /// No operands; immediate: `Class::prop` label data id (same shape as `LoadStaticProperty`).
    /// Late static binding (`static::$n`) resolves the concrete class at bind time by the
    /// runtime called-class id, so the bound address is fixed at the point of `=&`.
    LoadStaticPropRefCell,
    /// Promotes an indexed-array element to a reference cell and returns the cell
    /// pointer. Used to alias a local to `$a[idx]` (`$b =& $a[0]`). The returned pointer
    /// addresses the element's inline storage within the array; the local aliases it
    /// non-owning (the array owns the storage). Operands: array, index. No immediate.
    LoadArrayElemRefCell,
    /// Binds a local slot as a non-owning reference alias to a ref-cell pointer value.
    /// Operand: the cell pointer (SSA value); immediate: target local slot. The local
    /// does not own the cell (no release at scope exit); the owner is the object/source.
    BindRefCellPtr,
    /// Binds a reference property's slot to a ref-cell pointer (`$obj->prop = &$src`).
    /// Operands: the target object, then a value denoting the source cell pointer (a
    /// `load_prop_ref_cell` result, or a `load_ref_cell`/`load_local` of the source's
    /// ref-cell local). Immediate: target property name data id. The target property must
    /// be a reference property; the source owns the cell so the property aliases it.
    BindPropRefCell,
    DynamicPropGet,
    DynamicPropSet,
    NullsafePropGet,
    NullsafeMethodCall,
    MethodLookup,
    MethodCall,
    StaticMethodCall,
    EvalStaticMethodCall,
    /// Coerces a PHP numeric string operand to its integer value for an int-backed enum
    /// `from()`/`tryFrom()` call. Operand: the string. Immediate: data id of the PHP
    /// `TypeError` message thrown when the string is not numeric. Result: `I64`.
    EnumBackingStringToInt,
    /// Coerces a `Mixed` (dynamically-typed) operand to the integer backing value for an
    /// int-backed enum `from()`/`tryFrom()` call, dispatching on the runtime tag: int/bool
    /// forward the payload, float truncates, null becomes 0, a numeric string coerces (a
    /// non-numeric string throws `TypeError`), and array/object/resource/callable throw
    /// `TypeError`. Operand: the Mixed value. Immediate: data id of the PHP `TypeError`
    /// message prefix (`"E::from(): Argument #1 ($value) must be of type int, "`), to which
    /// codegen appends the runtime type word. Result: `I64`.
    EnumBackingMixedToInt,
    ClassConstant,
    ScopedConstantGet,
    ClassAttrNames,
    ClassAttrArgs,
    ClassGetAttributes,
    InstanceOfDynamic,
    Call,
    BuiltinCall,
    FunctionVariantCall,
    ClosureBind,
    LanguageConstructCall,
    EvalLiteralCall,
    EvalScopeGet,
    EvalScopeSet,
    EvalFunctionCall,
    EvalFunctionCallArray,
    EvalFunctionExists,
    EvalClassExists,
    EvalConstantExists,
    EvalConstantFetch,
    RuntimeCall,
    ExternCall,
    ClosureNew,
    ClosureCapture,
    ClosureCall,
    ExprCall,
    FirstClassCallableNew,
    CallableArrayNew,
    CallableDescriptorInvoke,
    PipeCall,
    PtrCast,
    PtrRead,
    PtrWrite,
    PtrReadString,
    PtrWriteString,
    PtrOffset,
    PtrCheckNonnull,
    BufferNew,
    BufferLen,
    BufferGet,
    BufferSet,
    BufferFree,
    PackedFieldGet,
    PackedFieldSet,
    ExternGlobalLoad,
    ExternGlobalStore,
    EchoValue,
    PrintValue,
    WriteStdout,
    VarDump,
    PrintR,
    ErrorSuppressBegin,
    ErrorSuppressEnd,
    Warn,
    ThrowException,
    /// Constructs and throws a catchable `\TypeError` for a checked-downcast-on-return
    /// guard mismatch. Operand: the mismatched value (read-only, for its runtime type
    /// name; released as part of this op since it is never returned to the caller).
    /// Immediate: a `Data` id for the compile-time message prefix (`"F(): Return value
    /// must be of type D, "`); the runtime-looked-up actual type name and a fixed
    /// `" returned"` suffix complete the message, matching PHP's own return-type
    /// `TypeError` wording. Never returns (see `crate::ir_lower::stmt::return_type_guard`).
    ///
    /// The operand may be a raw object pointer OR a boxed `Mixed`; codegen reads its static
    /// type to pick the type-name table (`get_class` vs the runtime-tag table) and the release
    /// helper (`__rt_decref_object` vs `__rt_decref_mixed`). The RELEASE is unconditional either
    /// way — that is the ownership policy this op exists to carry.
    ///
    /// An OPTIONAL second operand carries the message suffix. The return position supplies none
    /// and keeps the baked `" returned"` tail; a PROPERTY STORE of an owning temporary reuses this
    /// op — same ownership policy, since nothing else owns that value once the store is skipped —
    /// with a suffix naming the property and its declared type. The suffix is what varies; the
    /// release is not, which is why this stays one op and not two.
    ThrowCheckedReturnTypeError,
    /// Constructs and throws a catchable `\TypeError` for a checked-downcast guard mismatch at a
    /// position where the guarded value is STILL OWNED BY SOMEONE ELSE (a call argument, whose
    /// caller-side local owns it; a property store, whose source expression does). Operands: the
    /// mismatched value (read-only, for its runtime type name) and the message suffix string.
    /// Immediate: a `Data` id for the compile-time message prefix. DOES NOT release the operand —
    /// that is the whole reason this is a separate op from `ThrowCheckedReturnTypeError` rather
    /// than a flag on it: a single op with two ownership policies is how a double free comes
    /// back. Never returns (see `crate::ir_lower::checked_downcast`).
    ThrowCheckedTypeError,
    ThrowError,
    ThrowErrorValue,
    TryPushHandler,
    TryPopHandler,
    CatchCurrent,
    CatchBind,
    FinallyEnter,
    FinallyExit,
    FiberRuntimeCall,
    GeneratorNew,
    GeneratorYield,
    GeneratorYieldFrom,
    GeneratorReturn,
    IncludeOnceMark,
    IncludeOnceGuard,
    FunctionVariantMark,
    FunctionVariantDispatch,
    Acquire,
    Release,
    GcCollect,
    Move,
    Borrow,
    EnsureOwned,
    Nop,
}

impl Op {
    /// Returns the conservative default effect set for this opcode.
    pub fn default_effects(self) -> Effects {
        use Effects as E;
        use Op::*;
        match self {
            ConstI64
            | ConstF64
            | ConstStr
            | ConstNull
            | ConstBool
            | ConstClassName
            | DataAddr
            | IAdd
            | ISub
            | IMul
            | IPow
            | INeg
            | IBitAnd
            | IBitOr
            | IBitXor
            | IBitNot
            | IShl
            | IShrA
            | FAdd
            | FSub
            | FMul
            | FDiv
            | FPow
            | FNeg
            | ICmp
            | FCmp
            | StrLen
            | IToF
            | FToI
            | BoolToStr
            | StrToI
            | StrToF
            | StrToNumber
            | MixedTagOf
            | IsEmpty
            | FunctionVariantDispatch
            | PtrCast
            | PtrOffset
            | Move
            | Borrow
            | Nop => E::PURE,
            IDiv | ISDiv | ISMod | PtrCheckNonnull => E::MAY_FATAL,
            ICheckedAdd | ICheckedSub | ICheckedMul => E::ALLOC_HEAP | E::READS_HEAP,
            ConstEnumCase => E::ALLOC_HEAP,
            LoadCalledClassId => E::READS_LOCAL,
            LoadLocal | LoadRefCell | LoadStaticLocal | ClosureCapture => E::READS_LOCAL,
            StoreLocal | UnsetLocal | StoreRefCell | ListUnpack | FinallyEnter | FinallyExit => {
                E::WRITES_LOCAL
            }
            PromoteLocalRefCell => {
                E::READS_LOCAL | E::WRITES_LOCAL | E::ALLOC_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP
            }
            AliasLocalRefCell => E::READS_LOCAL | E::WRITES_LOCAL,
            ReleaseLocalRefCell => {
                E::READS_LOCAL | E::WRITES_LOCAL | E::WRITES_HEAP | E::REFCOUNT_OP
            }
            ReleaseLocalSlot => E::READS_LOCAL | E::WRITES_HEAP | E::REFCOUNT_OP,
            AdoptRefCell => E::WRITES_LOCAL | E::WRITES_HEAP | E::REFCOUNT_OP,
            HashRefElement => {
                E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::WRITES_LOCAL | E::REFCOUNT_OP
            }
            HashBindRefElement | HashRefAppendElement => {
                E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::WRITES_LOCAL | E::REFCOUNT_OP
            }
            LocalRefEnsure => {
                E::READS_LOCAL
                    | E::WRITES_LOCAL
                    | E::READS_HEAP
                    | E::WRITES_HEAP
                    | E::ALLOC_HEAP
                    | E::REFCOUNT_OP
            }
            LoadGlobal
            | LoadStaticProperty
            | LoadStaticPropRefCell
            | LoadReflectionStaticProperty
            | ReflectionStaticPropertyInitialized
            | ScopedConstantGet
            | ClassAttrNames
            | ClassAttrArgs
            | ClassGetAttributes
            | CatchCurrent
            | StaticLocalInitialized => E::READS_GLOBAL,
            CatchBind => E::READS_GLOBAL | E::WRITES_GLOBAL,
            StoreGlobal
            | StoreStaticLocal
            | StoreStaticProperty
            | StoreReflectionStaticProperty
            | InitStaticLocal
            | IncludeOnceMark
            | FunctionVariantMark
            | TryPushHandler
            | TryPopHandler => E::WRITES_GLOBAL,
            IncludeOnceGuard => E::READS_GLOBAL | E::WRITES_GLOBAL,
            IToStr | FToStr | ResourceToStr | StrConcat | StrBitwise | StrCharAt | StrInterpolate
            | MixedCastString | VarDump | PrintR => E::ALLOC_CONCAT,
            // Reads the source string bytes and writes the mutated copy into the
            // shared concat scratch; the empty-replacement/out-of-range cases warn.
            StrOffsetSet => E::READS_HEAP | E::ALLOC_CONCAT | E::MAY_WARN,
            ConcatReset => E::WRITES_GLOBAL,
            Cast => E::READS_HEAP | E::ALLOC_CONCAT | E::MAY_WARN | E::MAY_FATAL,
            // `(object)` reads the boxed source, allocates a fresh stdClass and its
            // property hash, and retains the inserted/passed-through payloads.
            ObjectCast => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP,
            // `(array)` reads the boxed source, allocates a fresh boxed-Mixed array,
            // and retains every boxed element it appends.
            ArrayCast => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            // Reads both boxed operands, allocates a fresh boxed-Mixed result
            // (int or persisted string), retains/persists the payload, and fatals
            // when a runtime array/object operand is paired with a bitwise operator.
            MixedBitwise => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            // Reads the boxed operand, allocates a fresh boxed-Mixed result (int or persisted
            // NOT-string), retains/persists the payload, and fatals when the runtime payload is
            // an array/object operand.
            MixedBitwiseNot => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            InvokerRefArg => E::READS_LOCAL | E::ALLOC_HEAP,
            MixedBox | ArrayToMixed | HashToMixed | ArrayNew | HashNew | ObjectNew
            | ClosureNew | FirstClassCallableNew | CallableArrayNew | BufferNew | GeneratorNew => {
                E::ALLOC_HEAP
            }
            // `clone` reads heap-backed properties, allocates fresh, retains payloads,
            // may invoke a user `__clone()` that throws/emits — conservatively may-throw.
            ObjectClone => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_THROW,
            IsNull | IsTruthy | TypePredicate | MixedUnbox | MixedCastBool | MixedCastInt | MixedCastFloat | ArrayGetSilent
            | HashGetSilent
            | ArrayIsset | HashIsset | BufferGet | BufferLen | PackedFieldGet | PtrRead
            | PtrReadString => {
                E::READS_HEAP | E::MAY_FATAL
            }
            ArrayGet | HashGet => E::READS_HEAP | E::MAY_FATAL | E::MAY_WARN,
            StrPersist | ArrayEnsureUnique | HashEnsureUnique | ArrayCloneShallow
            | HashCloneShallow | ObjectCloneShallow => {
                E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP
            }
            ArrayLen | HashLen => E::READS_HEAP | E::MAY_FATAL,
            ArrayKeyExists | OffsetExists | PropGet | PropInitialized | LoadPropRefCell => {
                E::READS_HEAP
            }
            // Reads the receiver heap to dispatch on the runtime property name; an unmatched
            // name compiles to a runtime fatal, so it is conservatively may-fatal.
            LoadDynamicPropRefCell => E::READS_HEAP | E::MAY_FATAL,
            // Reads a static property's global storage selected by a runtime name; an unmatched
            // name compiles to a runtime fatal, so it is conservatively may-fatal.
            LoadDynamicStaticProperty => E::READS_HEAP | E::MAY_FATAL,
            // Writes a static property's global storage selected by a runtime name; reads the heap
            // to dispatch, releases the previous refcounted value, and fatals on an unmatched name.
            StoreDynamicStaticProperty => {
                E::WRITES_HEAP | E::READS_HEAP | E::MAY_FATAL | E::REFCOUNT_OP
            }
            LoadArrayElemRefCell => E::READS_HEAP | E::MAY_FATAL,
            BindRefCellPtr => E::WRITES_LOCAL,
            BindPropRefCell => E::READS_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP,
            ArraySet | HashSet | HashUnset | ArrayPush | HashAppend | OffsetUnset | PropSet
            | DynamicPropSet | BufferSet | BufferFree | PackedFieldSet | PtrWrite
            | PtrWriteString => E::WRITES_HEAP | E::MAY_FATAL | E::REFCOUNT_OP,
            MixedArrayAppend => E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::MAY_FATAL | E::REFCOUNT_OP,
            ArrayElemAddr | ArraySetMixedKey => {
                E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::MAY_FATAL | E::REFCOUNT_OP
            }
            ArrayGetMixedKey => E::READS_HEAP | E::ALLOC_HEAP | E::MAY_FATAL | E::MAY_WARN,
            ArrayGetMixedKeySilent => E::READS_HEAP | E::ALLOC_HEAP | E::MAY_FATAL,
            ArrayUnion | HashUnion | ArrayHashUnion | HashArrayUnion | ArrayToHash => {
                E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP
            }
            // Unboxes a Mixed array and clones/rebuilds it into an owned hash; a
            // non-array payload fatals at the boundary.
            MixedToHash => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            HashSpread => E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP,
            IterStart | IterCurrentKey | IterCurrentValue | IteratorMethodCall
            | SplRuntimeCall | DynamicObjectNew | DynamicObjectNewMixed
            | DynamicObjectNewWithoutConstructorMixed | DynamicPropGet | NullsafePropGet
            | NullsafeMethodCall | MethodLookup | MethodCall | StaticMethodCall
            | InstanceOfDynamic | MixedNumericBinop | LooseEq | LooseNotEq | Spaceship => {
                E::READS_HEAP | E::MAY_DEOPT
            }
            IterCurrentValueRef | IterNext | IterEnd | GeneratorYield | GeneratorYieldFrom | GeneratorReturn => {
                E::READS_HEAP | E::WRITES_HEAP | E::MAY_DEOPT
            }
            StrEq | StrCmp | StrLooseEq | StrictEq | StrictNotEq | InstanceOf => E::READS_HEAP,
            EnumBackingStringToInt | EnumBackingMixedToInt => {
                E::READS_HEAP | E::ALLOC_HEAP | E::MAY_THROW
            }
            EvalFunctionExists | EvalClassExists | EvalConstantExists => E::READS_GLOBAL,
            EvalScopeGet => E::READS_HEAP | E::MAY_FATAL,
            EvalScopeSet => E::READS_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            EvalConstantFetch => {
                E::READS_GLOBAL | E::READS_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP | E::MAY_FATAL
            }
            Call
            | FunctionVariantCall
            | BuiltinCall
            | ClosureBind
            | LanguageConstructCall
            | EvalLiteralCall
            | EvalFunctionCall
            | EvalFunctionCallArray
            | EvalObjectNew
            | EvalStaticMethodCall
            | RuntimeCall
            | ClosureCall
            | ExprCall
            | CallableDescriptorInvoke
            | PipeCall
            | FiberRuntimeCall => E::all().difference(E::REFCOUNT_OP),
            ExternCall | ExternGlobalLoad | ExternGlobalStore => {
                E::READS_HEAP | E::WRITES_HEAP | E::READS_PROCESS | E::WRITES_PROCESS | E::MAY_THROW
            }
            EchoValue | WriteStrStdout | WriteStdout | Warn => E::OUTPUT,
            PrintValue => E::OUTPUT,
            ErrorSuppressBegin | ErrorSuppressEnd => E::READS_GLOBAL | E::WRITES_GLOBAL,
            ThrowException => E::MAY_THROW | E::WRITES_GLOBAL,
            // Reads the mismatched object's header for its runtime class name, allocates
            // the message and the `TypeError` object, releases the mismatched object
            // (never returned to the caller), then publishes and unwinds.
            ThrowCheckedReturnTypeError => {
                E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_THROW | E::WRITES_GLOBAL
            }
            // Same message building and unwinding as the return variant, minus the release: the
            // mismatched value keeps its owner, so no refcount is touched.
            ThrowCheckedTypeError => {
                E::READS_HEAP | E::ALLOC_HEAP | E::MAY_THROW | E::WRITES_GLOBAL
            }
            ThrowError | ThrowErrorValue => {
                E::MAY_THROW
                    | E::READS_GLOBAL
                    | E::WRITES_GLOBAL
                    | E::ALLOC_HEAP
                    | E::WRITES_HEAP
            }
            ObjectClassName => {
                E::READS_HEAP
                    | E::MAY_THROW
                    | E::READS_GLOBAL
                    | E::WRITES_GLOBAL
                    | E::ALLOC_HEAP
                    | E::WRITES_HEAP
            }
            Acquire | Release | EnsureOwned => E::REFCOUNT_OP | E::WRITES_HEAP,
            GcCollect => E::READS_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP,
            ClassConstant => E::MAY_DEOPT,
        }
    }

    /// Returns true when the builder may replace the conservative default effects.
    pub fn allows_effect_refinement(self) -> bool {
        matches!(
            self,
            Op::Call
                | Op::FunctionVariantCall
                | Op::ClosureBind
                | Op::LanguageConstructCall
                | Op::EvalLiteralCall
                | Op::EvalFunctionCall
                | Op::EvalFunctionCallArray
                | Op::EvalObjectNew
                | Op::EvalStaticMethodCall
                | Op::RuntimeCall
                | Op::ExternCall
                | Op::MethodCall
                | Op::StaticMethodCall
                | Op::ClosureCall
                | Op::ExprCall
                | Op::CallableDescriptorInvoke
                | Op::PipeCall
                | Op::IteratorMethodCall
                | Op::SplRuntimeCall
                | Op::FiberRuntimeCall
        )
    }

    /// Returns the lower-case textual opcode spelling.
    pub fn name(self) -> &'static str {
        use Op::*;
        match self {
            ConstI64 => "const_i64",
            ConstF64 => "const_f64",
            ConstStr => "const_str",
            ConstNull => "const_null",
            ConstBool => "const_bool",
            ConstClassName => "const_class_name",
            ObjectClassName => "object_class_name",
            ConstEnumCase => "const_enum_case",
            LoadCalledClassId => "load_called_class_id",
            DataAddr => "data_addr",
            LoadLocal => "load_local",
            StoreLocal => "store_local",
            UnsetLocal => "unset_local",
            LoadRefCell => "load_ref_cell",
            StoreRefCell => "store_ref_cell",
            PromoteLocalRefCell => "promote_local_ref_cell",
            AliasLocalRefCell => "alias_local_ref_cell",
            ReleaseLocalRefCell => "release_local_ref_cell",
            AdoptRefCell => "adopt_ref_cell",
            HashRefElement => "hash_ref_element",
            HashBindRefElement => "hash_bind_ref_element",
            HashRefAppendElement => "hash_ref_append_element",
            LocalRefEnsure => "local_ref_ensure",
            ReleaseLocalSlot => "release_local_slot",
            LoadGlobal => "load_global",
            StoreGlobal => "store_global",
            LoadStaticLocal => "load_static_local",
            StoreStaticLocal => "store_static_local",
            InitStaticLocal => "init_static_local",
            StaticLocalInitialized => "static_local_initialized",
            LoadStaticProperty => "load_static_property",
            LoadDynamicStaticProperty => "load_dynamic_static_property",
            StoreDynamicStaticProperty => "store_dynamic_static_property",
            StoreStaticProperty => "store_static_property",
            LoadReflectionStaticProperty => "load_reflection_static_property",
            StoreReflectionStaticProperty => "store_reflection_static_property",
            ReflectionStaticPropertyInitialized => "reflection_static_property_initialized",
            IAdd => "iadd",
            ISub => "isub",
            IMul => "imul",
            ICheckedAdd => "ichecked_add",
            ICheckedSub => "ichecked_sub",
            ICheckedMul => "ichecked_mul",
            IDiv => "idiv",
            ISDiv => "isdiv",
            ISMod => "ismod",
            IPow => "ipow",
            INeg => "ineg",
            IBitAnd => "ibit_and",
            IBitOr => "ibit_or",
            IBitXor => "ibit_xor",
            IBitNot => "ibit_not",
            IShl => "ishl",
            IShrA => "ishr_a",
            FAdd => "fadd",
            FSub => "fsub",
            FMul => "fmul",
            FDiv => "fdiv",
            FPow => "fpow",
            FNeg => "fneg",
            MixedNumericBinop => "mixed_numeric_binop",
            ICmp => "icmp",
            FCmp => "fcmp",
            StrEq => "str_eq",
            StrCmp => "str_cmp",
            StrLooseEq => "str_loose_eq",
            StrictEq => "strict_eq",
            StrictNotEq => "strict_not_eq",
            LooseEq => "loose_eq",
            LooseNotEq => "loose_not_eq",
            Spaceship => "spaceship",
            IsNull => "is_null",
            IsTruthy => "is_truthy",
            TypePredicate => "type_predicate",
            IsEmpty => "is_empty",
            InstanceOf => "instance_of",
            IToF => "i_to_f",
            FToI => "f_to_i",
            IToStr => "i_to_str",
            FToStr => "f_to_str",
            BoolToStr => "bool_to_str",
            StrToI => "str_to_i",
            StrToF => "str_to_f",
            StrToNumber => "str_to_number",
            ResourceToStr => "resource_to_str",
            Cast => "cast",
            ObjectCast => "object_cast",
            ArrayCast => "array_cast",
            MixedBox => "mixed_box",
            InvokerRefArg => "invoker_ref_arg",
            MixedUnbox => "mixed_unbox",
            MixedTagOf => "mixed_tag_of",
            ArrayToMixed => "array_to_mixed",
            HashToMixed => "hash_to_mixed",
            MixedCastBool => "mixed_cast_bool",
            MixedCastInt => "mixed_cast_int",
            MixedCastFloat => "mixed_cast_float",
            MixedCastString => "mixed_cast_string",
            StrConcat => "str_concat",
            StrBitwise => "str_bitwise",
            MixedBitwise => "mixed_bitwise",
            MixedBitwiseNot => "mixed_bitwise_not",
            StrLen => "str_len",
            StrPersist => "str_persist",
            StrCharAt => "str_char_at",
            StrOffsetSet => "str_offset_set",
            StrInterpolate => "str_interpolate",
            ConcatReset => "concat_reset",
            WriteStrStdout => "write_str_stdout",
            ArrayNew => "array_new",
            HashNew => "hash_new",
            ArrayLen => "array_len",
            HashLen => "hash_len",
            ArrayGet => "array_get",
            ArrayGetSilent => "array_get_silent",
            HashGet => "hash_get",
            HashGetSilent => "hash_get_silent",
            ArrayIsset => "array_isset",
            HashIsset => "hash_isset",
            ArrayElemAddr => "array_elem_addr",
            ArraySet => "array_set",
            HashSet => "hash_set",
            HashUnset => "hash_unset",
            ArrayPush => "array_push",
            MixedArrayAppend => "mixed_array_append",
            HashAppend => "hash_append",
            ArrayEnsureUnique => "array_ensure_unique",
            HashEnsureUnique => "hash_ensure_unique",
            ArrayCloneShallow => "array_clone_shallow",
            HashCloneShallow => "hash_clone_shallow",
            ArrayUnion => "array_union",
            HashUnion => "hash_union",
            ArrayHashUnion => "array_hash_union",
            HashArrayUnion => "hash_array_union",
            HashSpread => "hash_spread",
            ArrayToHash => "array_to_hash",
            MixedToHash => "mixed_to_hash",
            ArraySetMixedKey => "array_set_mixed_key",
            ArrayGetMixedKey => "array_get_mixed_key",
            ArrayGetMixedKeySilent => "array_get_mixed_key_silent",
            ArrayKeyExists => "array_key_exists",
            OffsetExists => "offset_exists",
            OffsetUnset => "offset_unset",
            ListUnpack => "list_unpack",
            IterStart => "iter_start",
            IterCurrentKey => "iter_current_key",
            IterCurrentValue => "iter_current_value",
            IterCurrentValueRef => "iter_current_value_ref",
            IterNext => "iter_next",
            IterEnd => "iter_end",
            IteratorMethodCall => "iterator_method_call",
            SplRuntimeCall => "spl_runtime_call",
            ObjectNew => "object_new",
            ObjectClone => "object_clone",
            EvalObjectNew => "eval_object_new",
            ObjectCloneShallow => "object_clone_shallow",
            DynamicObjectNew => "dynamic_object_new",
            DynamicObjectNewMixed => "dynamic_object_new_mixed",
            DynamicObjectNewWithoutConstructorMixed => {
                "dynamic_object_new_without_constructor_mixed"
            }
            PropGet => "prop_get",
            PropInitialized => "prop_initialized",
            PropSet => "prop_set",
            LoadPropRefCell => "load_prop_ref_cell",
            LoadDynamicPropRefCell => "load_dynamic_prop_ref_cell",
            LoadStaticPropRefCell => "load_static_prop_ref_cell",
            LoadArrayElemRefCell => "load_array_elem_ref_cell",
            BindRefCellPtr => "bind_ref_cell_ptr",
            BindPropRefCell => "bind_prop_ref_cell",
            DynamicPropGet => "dynamic_prop_get",
            DynamicPropSet => "dynamic_prop_set",
            NullsafePropGet => "nullsafe_prop_get",
            NullsafeMethodCall => "nullsafe_method_call",
            MethodLookup => "method_lookup",
            MethodCall => "method_call",
            StaticMethodCall => "static_method_call",
            EvalStaticMethodCall => "eval_static_method_call",
            EnumBackingStringToInt => "enum_backing_string_to_int",
            EnumBackingMixedToInt => "enum_backing_mixed_to_int",
            ClassConstant => "class_constant",
            ScopedConstantGet => "scoped_constant_get",
            ClassAttrNames => "class_attr_names",
            ClassAttrArgs => "class_attr_args",
            ClassGetAttributes => "class_get_attributes",
            InstanceOfDynamic => "instance_of_dynamic",
            Call => "call",
            BuiltinCall => "builtin_call",
            FunctionVariantCall => "function_variant_call",
            ClosureBind => "closure_bind",
            LanguageConstructCall => "language_construct_call",
            EvalLiteralCall => "eval_literal_call",
            EvalScopeGet => "eval_scope_get",
            EvalScopeSet => "eval_scope_set",
            EvalFunctionCall => "eval_function_call",
            EvalFunctionCallArray => "eval_function_call_array",
            EvalFunctionExists => "eval_function_exists",
            EvalClassExists => "eval_class_exists",
            EvalConstantExists => "eval_constant_exists",
            EvalConstantFetch => "eval_constant_fetch",
            RuntimeCall => "runtime_call",
            ExternCall => "extern_call",
            ClosureNew => "closure_new",
            ClosureCapture => "closure_capture",
            ClosureCall => "closure_call",
            ExprCall => "expr_call",
            FirstClassCallableNew => "first_class_callable_new",
            CallableArrayNew => "callable_array_new",
            CallableDescriptorInvoke => "callable_descriptor_invoke",
            PipeCall => "pipe_call",
            PtrCast => "ptr_cast",
            PtrRead => "ptr_read",
            PtrWrite => "ptr_write",
            PtrReadString => "ptr_read_string",
            PtrWriteString => "ptr_write_string",
            PtrOffset => "ptr_offset",
            PtrCheckNonnull => "ptr_check_nonnull",
            BufferNew => "buffer_new",
            BufferLen => "buffer_len",
            BufferGet => "buffer_get",
            BufferSet => "buffer_set",
            BufferFree => "buffer_free",
            PackedFieldGet => "packed_field_get",
            PackedFieldSet => "packed_field_set",
            ExternGlobalLoad => "extern_global_load",
            ExternGlobalStore => "extern_global_store",
            EchoValue => "echo_value",
            PrintValue => "print_value",
            WriteStdout => "write_stdout",
            VarDump => "var_dump",
            PrintR => "print_r",
            ErrorSuppressBegin => "error_suppress_begin",
            ErrorSuppressEnd => "error_suppress_end",
            Warn => "warn",
            ThrowException => "throw_exception",
            ThrowCheckedReturnTypeError => "throw_checked_return_type_error",
            ThrowCheckedTypeError => "throw_checked_type_error",
            ThrowError => "throw_error",
            ThrowErrorValue => "throw_error_value",
            TryPushHandler => "try_push_handler",
            TryPopHandler => "try_pop_handler",
            CatchCurrent => "catch_current",
            CatchBind => "catch_bind",
            FinallyEnter => "finally_enter",
            FinallyExit => "finally_exit",
            FiberRuntimeCall => "fiber_runtime_call",
            GeneratorNew => "generator_new",
            GeneratorYield => "generator_yield",
            GeneratorYieldFrom => "generator_yield_from",
            GeneratorReturn => "generator_return",
            IncludeOnceMark => "include_once_mark",
            IncludeOnceGuard => "include_once_guard",
            FunctionVariantMark => "function_variant_mark",
            FunctionVariantDispatch => "function_variant_dispatch",
            Acquire => "acquire",
            Release => "release",
            GcCollect => "gc_collect",
            Move => "move",
            Borrow => "borrow",
            EnsureOwned => "ensure_owned",
            Nop => "nop",
        }
    }
}

#[cfg(test)]
mod tests {
    /// `Instruction` is built by value inside the recursive AST->EIR lowering
    /// paths, so its size feeds every lowering stack frame. Growing it past
    /// main's 112 bytes shrank the headroom enough that 2 MiB test threads
    /// overflowed on linux-aarch64. Keep provenance and future metadata inside
    /// the existing padding.
    #[test]
    fn instruction_stays_112_bytes() {
        assert!(std::mem::size_of::<super::Instruction>() <= 112);
    }
}
