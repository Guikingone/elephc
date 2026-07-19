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
    LocalSlotPair { first: LocalSlotId, second: LocalSlotId },
    GlobalName(DataId),
    FunctionRef(FunctionId),
    BuiltinRef(BuiltinId),
    RuntimeRef(RuntimeId),
    ExternRef(u32),
    ClassRef(u32),
    EnumCaseRef { enum_id: u32, case_id: u32 },
    MethodRef { class: u32, method: u32 },
    PropertyRef { class: u32, property: u32 },
    FieldRef { layout: u32, field: u32 },
    FunctionVariantRef { group: u32, variant: u32 },
    HeapKind(IrHeapKind),
    MixedTag(u8),
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
    ConstEnumCase,
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
    IAdd,
    ISub,
    IMul,
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
    StrLen,
    StrPersist,
    StrCharAt,
    StrInterpolate,
    ConcatReset,
    WriteStrStdout,
    ArrayNew,
    HashNew,
    ArrayLen,
    HashLen,
    ArrayGet,
    HashGet,
    ArrayIsset,
    HashIsset,
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
    ArrayToHash,
    /// Converts a boxed `Mixed`/union array into a freshly cloned owned hash at the
    /// gradual-typing boundary. The source array is never mutated (it is shallow-cloned
    /// for tag-5 hashes, or rebuilt for tag-4 indexed arrays); a non-array payload takes
    /// a runtime `TypeError` fatal. The single owned result is released by the consumer.
    MixedToHash,
    ArraySetMixedKey,
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
    DynamicObjectNew,
    DynamicObjectNewMixed,
    PropGet,
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
    ClassConstant,
    ScopedConstantGet,
    ClassAttrNames,
    ClassAttrArgs,
    ClassGetAttributes,
    InstanceOfDynamic,
    Call,
    FunctionVariantCall,
    BuiltinCall,
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
    /// guard mismatch. Operand: the mismatched object value (read-only, for its runtime
    /// class name; released as part of this op since it is never returned to the caller).
    /// Immediate: a `Data` id for the compile-time message prefix (`"F(): Return value
    /// must be of type D, "`); the runtime-looked-up actual class name and a fixed
    /// `" returned"` suffix complete the message, matching PHP's own return-type
    /// `TypeError` wording. Never returns (see `crate::ir_lower::stmt::return_type_guard`).
    ThrowCheckedReturnTypeError,
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
            ConstI64 | ConstF64 | ConstStr | ConstNull | ConstBool | ConstClassName
            | DataAddr | IAdd | ISub | IMul | IPow | INeg | IBitAnd | IBitOr | IBitXor
            | IBitNot | IShl | IShrA | FAdd | FSub | FMul | FDiv | FPow | FNeg | ICmp
            | FCmp | StrLen | IToF | FToI | BoolToStr | StrToI | StrToF | StrToNumber
            | MixedTagOf | IsNull | IsTruthy | IsEmpty | FunctionVariantDispatch | PtrCast
            | PtrOffset | Move | Borrow | Nop => E::PURE,
            IDiv | ISDiv | ISMod | PtrCheckNonnull => E::MAY_FATAL,
            ConstEnumCase => E::ALLOC_HEAP,
            LoadLocal | LoadRefCell | LoadStaticLocal | ClosureCapture => E::READS_LOCAL,
            StoreLocal | UnsetLocal | StoreRefCell | ListUnpack | CatchBind | FinallyEnter
            | FinallyExit => E::WRITES_LOCAL,
            PromoteLocalRefCell => {
                E::READS_LOCAL | E::WRITES_LOCAL | E::ALLOC_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP
            },
            AliasLocalRefCell => E::READS_LOCAL | E::WRITES_LOCAL,
            ReleaseLocalRefCell => E::READS_LOCAL | E::WRITES_LOCAL | E::WRITES_HEAP | E::REFCOUNT_OP,
            AdoptRefCell => E::WRITES_LOCAL | E::WRITES_HEAP | E::REFCOUNT_OP,
            HashRefElement => {
                E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::WRITES_LOCAL | E::REFCOUNT_OP
            }
            HashBindRefElement | HashRefAppendElement => {
                E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::WRITES_LOCAL | E::REFCOUNT_OP
            }
            LocalRefEnsure => {
                E::READS_LOCAL | E::WRITES_LOCAL | E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP
            }
            LoadGlobal | LoadStaticProperty | LoadStaticPropRefCell | ScopedConstantGet | ClassAttrNames
            | ClassAttrArgs | ClassGetAttributes | CatchCurrent | StaticLocalInitialized => E::READS_GLOBAL,
            StoreGlobal | StoreStaticLocal | StoreStaticProperty | InitStaticLocal | IncludeOnceMark
            | FunctionVariantMark | TryPushHandler | TryPopHandler => E::WRITES_GLOBAL,
            IncludeOnceGuard => E::READS_GLOBAL | E::WRITES_GLOBAL,
            IToStr | FToStr | ResourceToStr | StrConcat | StrBitwise | StrCharAt | StrInterpolate
            | MixedCastString | VarDump | PrintR => E::ALLOC_CONCAT,
            ConcatReset => E::WRITES_GLOBAL,
            Cast => E::READS_HEAP | E::ALLOC_CONCAT | E::MAY_WARN | E::MAY_FATAL,
            // `(object)` reads the boxed source, allocates a fresh stdClass and its
            // property hash, and retains the inserted/passed-through payloads.
            ObjectCast => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP,
            // `(array)` reads the boxed source, allocates a fresh boxed-Mixed array,
            // and retains every boxed element it appends.
            ArrayCast => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            InvokerRefArg => E::READS_LOCAL | E::ALLOC_HEAP,
            MixedBox | ArrayToMixed | HashToMixed | ArrayNew | HashNew | ObjectNew
            | ClosureNew | FirstClassCallableNew | CallableArrayNew | BufferNew | GeneratorNew => {
                E::ALLOC_HEAP
            }
            // `clone` reads the operand's heap-backed properties, allocates a fresh
            // object (and a fresh Mixed cell when the operand is boxed), retains
            // refcounted/string property payloads, and may invoke a user `__clone()`
            // that throws or emits output — so it is conservatively may-throw.
            ObjectClone => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_THROW,
            MixedUnbox | MixedCastBool | MixedCastInt | MixedCastFloat | ArrayGet | HashGet
            | ArrayIsset | HashIsset | BufferGet | BufferLen | PackedFieldGet | PtrRead
            | PtrReadString => {
                E::READS_HEAP | E::MAY_FATAL
            }
            StrPersist | ArrayEnsureUnique | HashEnsureUnique | ArrayCloneShallow
            | HashCloneShallow => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP,
            ArrayLen | HashLen | ArrayKeyExists | OffsetExists | PropGet | LoadPropRefCell => {
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
            BindRefCellPtr => E::WRITES_LOCAL,
            BindPropRefCell => E::READS_HEAP | E::WRITES_HEAP | E::REFCOUNT_OP,
            ArraySet | HashSet | HashUnset | ArrayPush | HashAppend | OffsetUnset | PropSet
            | DynamicPropSet | BufferSet | BufferFree | PackedFieldSet | PtrWrite
            | PtrWriteString => E::WRITES_HEAP | E::MAY_FATAL | E::REFCOUNT_OP,
            MixedArrayAppend => E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::MAY_FATAL | E::REFCOUNT_OP,
            ArraySetMixedKey => E::READS_HEAP | E::WRITES_HEAP | E::ALLOC_HEAP | E::MAY_FATAL | E::REFCOUNT_OP,
            ArrayUnion | HashUnion | ArrayHashUnion | HashArrayUnion | ArrayToHash => {
                E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP
            }
            // Unboxes a Mixed array and clones/rebuilds it into an owned hash; a
            // non-array payload fatals at the boundary.
            MixedToHash => E::READS_HEAP | E::ALLOC_HEAP | E::REFCOUNT_OP | E::MAY_FATAL,
            IterStart | IterCurrentKey | IterCurrentValue | IteratorMethodCall
            | SplRuntimeCall | DynamicObjectNew | DynamicObjectNewMixed | DynamicPropGet | NullsafePropGet
            | NullsafeMethodCall | MethodLookup | MethodCall | StaticMethodCall
            | InstanceOfDynamic | MixedNumericBinop | LooseEq | LooseNotEq | Spaceship => {
                E::READS_HEAP | E::MAY_DEOPT
            }
            IterCurrentValueRef | IterNext | IterEnd | GeneratorYield | GeneratorYieldFrom | GeneratorReturn => {
                E::READS_HEAP | E::WRITES_HEAP | E::MAY_DEOPT
            }
            StrEq | StrCmp | StrLooseEq | StrictEq | StrictNotEq | InstanceOf => E::READS_HEAP,
            Call | FunctionVariantCall | BuiltinCall | RuntimeCall | ClosureCall | ExprCall
            | CallableDescriptorInvoke | PipeCall | FiberRuntimeCall => {
                E::all().difference(E::REFCOUNT_OP)
            }
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
                | Op::BuiltinCall
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
            ConstEnumCase => "const_enum_case",
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
            IAdd => "iadd",
            ISub => "isub",
            IMul => "imul",
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
            StrLen => "str_len",
            StrPersist => "str_persist",
            StrCharAt => "str_char_at",
            StrInterpolate => "str_interpolate",
            ConcatReset => "concat_reset",
            WriteStrStdout => "write_str_stdout",
            ArrayNew => "array_new",
            HashNew => "hash_new",
            ArrayLen => "array_len",
            HashLen => "hash_len",
            ArrayGet => "array_get",
            HashGet => "hash_get",
            ArrayIsset => "array_isset",
            HashIsset => "hash_isset",
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
            ArrayToHash => "array_to_hash",
            MixedToHash => "mixed_to_hash",
        ArraySetMixedKey => "array_set_mixed_key",
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
            DynamicObjectNew => "dynamic_object_new",
            DynamicObjectNewMixed => "dynamic_object_new_mixed",
            PropGet => "prop_get",
            PropSet => "prop_set",
            LoadPropRefCell => "load_prop_ref_cell",
            LoadDynamicPropRefCell => "load_dynamic_prop_ref_cell",
            LoadStaticPropRefCell => "load_static_prop_ref_cell",
            BindRefCellPtr => "bind_ref_cell_ptr",
            BindPropRefCell => "bind_prop_ref_cell",
            DynamicPropGet => "dynamic_prop_get",
            DynamicPropSet => "dynamic_prop_set",
            NullsafePropGet => "nullsafe_prop_get",
            NullsafeMethodCall => "nullsafe_method_call",
            MethodLookup => "method_lookup",
            MethodCall => "method_call",
            StaticMethodCall => "static_method_call",
            ClassConstant => "class_constant",
            ScopedConstantGet => "scoped_constant_get",
            ClassAttrNames => "class_attr_names",
            ClassAttrArgs => "class_attr_args",
            ClassGetAttributes => "class_get_attributes",
            InstanceOfDynamic => "instance_of_dynamic",
            Call => "call",
            FunctionVariantCall => "function_variant_call",
            BuiltinCall => "builtin_call",
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
