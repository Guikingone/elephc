//! Purpose:
//! Checks type compatibility for unions cases.
//! Supports the central assignability predicate used by declarations, calls, returns, and assignments.
//!
//! Called from:
//! - `crate::types::checker::type_compat`
//!
//! Key details:
//! - Rules here define accepted programs, so PHP covariance, inheritance, and extension-specific constraints must stay explicit.

use crate::types::PhpType;

use super::super::Checker;

impl Checker {
    /// Flattens nested unions, removes duplicates and `PhpType::Mixed` (which absorbs all),
    /// and returns a single `PhpType` or a `PhpType::Union` with deduped members.
    pub(crate) fn normalize_union_type(&self, members: Vec<PhpType>) -> PhpType {
        let mut flat = Vec::new();
        for member in members {
            match member {
                PhpType::Union(inner) => flat.extend(inner),
                PhpType::Mixed => return PhpType::Mixed,
                other => flat.push(other),
            }
        }

        let mut deduped = Vec::new();
        for member in flat {
            // `bool` already contains the literal `false` subtype. Keep `False` only when the
            // declaration is specifically false-only (for example `int|false`).
            if member == PhpType::False && deduped.iter().any(|existing| existing == &PhpType::Bool) {
                continue;
            }
            if member == PhpType::Bool {
                deduped.retain(|existing| existing != &PhpType::False);
            }
            if !deduped.iter().any(|existing| existing == &member) {
                deduped.push(member);
            }
        }

        if deduped.len() == 1 {
            deduped.pop().expect("union member exists")
        } else {
            PhpType::Union(deduped)
        }
    }

    /// Returns true if `expected` type can accept a value of `actual` type (i.e., the
    /// assignment `expected = actual` is valid). Checks identity, Mixed, unions, arrays,
    /// associative arrays, object class/interface compatibility, `iterable`, pointers,
    /// and resources.
    pub(crate) fn type_accepts(&self, expected: &PhpType, actual: &PhpType) -> bool {
        if expected == actual {
            return true;
        }
        match expected {
            PhpType::Mixed => true,
            PhpType::Bool if matches!(actual, PhpType::False) => true,
            PhpType::Callable => match actual {
                PhpType::Object(name)
                    if name
                        .trim_start_matches('\\')
                        .eq_ignore_ascii_case("Closure") =>
                {
                    true
                }
                PhpType::Union(members) => members
                    .iter()
                    .all(|member| self.type_accepts(expected, member)),
                _ => false,
            },
            // PHP coercive mode: scalars accept Mixed with runtime narrowing.
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Str
                if matches!(actual, PhpType::Mixed) =>
            {
                true
            }
            PhpType::Union(members) => match actual {
                PhpType::Union(actual_members) => actual_members.iter().all(|actual_member| {
                    members
                        .iter()
                        .any(|expected_member| self.type_accepts(expected_member, actual_member))
                }),
                _ => members
                    .iter()
                    .any(|member| self.type_accepts(member, actual)),
            },
            PhpType::Array(expected_elem) => match actual {
                PhpType::Array(actual_elem) if matches!(actual_elem.as_ref(), PhpType::Never) => {
                    true
                }
                PhpType::Array(actual_elem) => {
                    self.type_accepts(expected_elem.as_ref(), actual_elem.as_ref())
                }
                PhpType::AssocArray { .. } => matches!(expected_elem.as_ref(), PhpType::Mixed),
                _ => false,
            },
            PhpType::AssocArray {
                key: expected_key,
                value: expected_value,
            } => match actual {
                PhpType::AssocArray {
                    key: actual_key,
                    value: actual_value,
                } => {
                    self.type_accepts(expected_key.as_ref(), actual_key.as_ref())
                        && self.type_accepts(expected_value.as_ref(), actual_value.as_ref())
                }
                PhpType::Array(actual_elem)
                    if matches!(expected_key.as_ref(), PhpType::Mixed | PhpType::Int)
                        && self.type_accepts(expected_value.as_ref(), actual_elem.as_ref()) =>
                {
                    true
                }
                _ => false,
            },
            PhpType::Object(expected_name) => match actual {
                PhpType::Object(actual_name) => {
                    expected_name.is_empty()
                        || expected_name == actual_name
                        || self.is_subclass_of(actual_name, expected_name)
                        || self.class_implements_interface(actual_name, expected_name)
                        || self.interface_extends_interface(actual_name, expected_name)
                }
                // A `Closure`/`Callable` value IS an object in PHP (every closure is a
                // `Closure` instance), so it satisfies the bare `object` pseudo-type
                // (`Object("")`, an object whose class is not statically known). This lets a
                // closure flow into an `object` slot — including as the `object` member of a
                // `string|array|object` callable union (HttpKernel `ControllerEvent::$controller`).
                // Reading such a union back yields a boxed-`Mixed` slot, so a later invocation
                // dispatches by runtime tag (the safe `lower_mixed_callable_descriptor_invoke`
                // path), never a by-offset object access. A CONCRETE class target (non-empty
                // name) still rejects a callable: elephc emits no by-name/instanceof guard there,
                // and a closure is not an instance of that class.
                PhpType::Callable if expected_name.is_empty() => true,
                _ => false,
            },
            PhpType::Iterable => match actual {
                PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => true,
                PhpType::Object(actual_name) => self.object_type_implements_iterable(actual_name),
                _ => false,
            },
            PhpType::Pointer(_) => Self::pointer_types_compatible(expected, actual),
            PhpType::Resource(_) => PhpType::resource_types_compatible(expected, actual),
            _ => false,
        }
    }

    /// Returns true when a value statically typed `actual` may flow into a position whose
    /// declared concrete target type is `expected`, under elephc's gradual-typing boundary
    /// model. This is deliberately looser than `type_accepts`/`types_compatible`: a `Mixed`
    /// source is accepted for every target (Mixed is the gradual top type), and a union
    /// source is accepted when one member matches the target and the remaining members are
    /// coercible to the target's value family. EIR codegen emits a runtime boundary guard
    /// (unbox + coercive cast, or unbox + null/heap-tag assert) for every flow accepted here.
    ///
    /// A concrete source type that is disjoint from `expected` (non-Mixed, non-union) is NOT
    /// accepted, so genuine type errors — e.g. passing a statically-`int` value to a `string`
    /// parameter — keep being reported by the strict predicates that call this as a fallback.
    pub(crate) fn gradual_boundary_accepts(&self, expected: &PhpType, actual: &PhpType) -> bool {
        match actual {
            // Mixed is the gradual top type. A boundary guard coerces the boxed payload to
            // the target representation at runtime, so any concrete target accepts it.
            PhpType::Mixed => true,
            PhpType::Union(src_members) if matches!(expected, PhpType::Union(_)) => {
                // Union source into a union target: accept when every source member is
                // acceptable to the target union (order-independent set compatibility).
                // Mixed was already normalized out of both unions.
                src_members
                    .iter()
                    .all(|member| self.type_accepts(expected, member))
            }
            PhpType::Union(src_members) => self.gradual_union_flows_into(expected, src_members),
            _ => false,
        }
    }

    /// Returns true when a union source (`src_members`) may flow into the concrete target
    /// `expected`: one member must match the target (equal, a subtype in the class hierarchy,
    /// or — for non-object members only — a supertype/numeric-widening compatible type) and
    /// every other member must be coercible to the target's value family (scalar targets accept
    /// any scalar plus null; class and array/iterable targets accept only null among the extra
    /// members).
    ///
    /// The supertype direction is deliberately excluded when both the target and the member are
    /// object types: a union member that is a base class/interface of a concrete OBJECT target
    /// is the unprovable base->derived direction (PHP TypeErrors, and elephc emits no runtime
    /// instanceof guard), so it must fall through to the loud coercible check instead of
    /// matching. Sound covariant object unions (union of subtypes into a common base) still
    /// match via the subtype direction.
    fn gradual_union_flows_into(&self, expected: &PhpType, src_members: &[PhpType]) -> bool {
        // The gradual rule only loosens flows into a single concrete target. Union and Mixed
        // targets are already handled by the strict predicates that call this as a fallback.
        if matches!(expected, PhpType::Union(_) | PhpType::Mixed) {
            return false;
        }
        let mut has_match = false;
        for member in src_members {
            // The supertype direction (`type_accepts(member, expected)`) is only sound for
            // NON-object members: a union member that is a base class/interface of a concrete
            // OBJECT target is the unprovable base->derived direction (PHP TypeErrors, and
            // elephc emits no instanceof guard), so it must fall through to the coercible check
            // (which is loud for objects) rather than matching.
            let member_matches = self.type_accepts(expected, member)
                || (self.type_accepts(member, expected)
                    && !matches!(
                        (expected, member),
                        (PhpType::Object(_), PhpType::Object(_))
                    ));
            if member_matches {
                has_match = true;
            } else if !self.gradual_other_member_coercible(expected, member) {
                return false;
            }
        }
        has_match
    }

    /// Returns true when `member` — a non-matching member of a union source — is acceptable
    /// alongside the matching member when the union flows into the concrete `target`. Null and
    /// void map to the boxed-null tag and are accepted for any target family.
    ///
    /// A concrete NON-object target physically receives a boxed cell for a union argument
    /// (`PhpType::Union(_).codegen_repr() == Mixed`) and coerces it at the call/return boundary
    /// with defined, memory-safe runtime semantics (string/array/scalar casts, or a graceful
    /// "could not be converted" fatal) — identical to the already-accepted `mixed` source — so any
    /// extra member is deferred to runtime. The single deliberate exception is `false`: a
    /// `false` alongside a scalar (`int|false`, `string|false`) stays a hard error to preserve the
    /// sentinel-return diagnostic, because a silent `false`→`0`/`""` coercion is the classic
    /// `strpos()`/`fgetc()` footgun (`0` is a valid offset).
    ///
    /// OBJECT targets are excluded entirely (`_ => false`): elephc emits no runtime instanceof
    /// guard at an object boundary, so a wrongly-typed extra member would be bit-read as the wrong
    /// object (a SIGSEGV risk) and must stay loud.
    fn gradual_other_member_coercible(&self, target: &PhpType, member: &PhpType) -> bool {
        if matches!(member, PhpType::Void | PhpType::Never | PhpType::Mixed) {
            return true;
        }
        match target {
            PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::Str
            | PhpType::Array(_)
            | PhpType::AssocArray { .. }
            | PhpType::Iterable => !matches!(member, PhpType::False),
            _ => false,
        }
    }

    /// PHP weak-mode call-argument / return coercions that the value-passing codegen boundary
    /// realizes with defined, memory-safe semantics. This is used ONLY as a fallback at
    /// call-argument and return positions (`require_compatible_call_arg_type` and the
    /// `require_compatible_return_type` fallback) — never at property or static-property stores,
    /// which emit no coercion and must keep using the strict predicates only.
    ///
    /// Accepted flows (everything else stays loud):
    /// 1. Concrete `int`/`float` → `string`. The call/return boundary emits the same
    ///    `IToStr`/`FToStr` used by `(string)$scalar`. `bool` is excluded here because its
    ///    int-like representation would stringify `false` as `"0"` rather than PHP's `""`.
    /// 2. Stringable object → `string`. A class (or ancestor) with a public `__toString()` is
    ///    coerced through the same `__toString()` dispatch as `(string)$obj`.
    /// 3. A SCALAR-ONLY union with a boxed-`Mixed` codegen representation → `string` (e.g.
    ///    `string|false`, `string|null`, `string|int`). The boundary's `__rt_mixed_cast_string`
    ///    tag dispatch realizes the exact PHP weak-mode form of whichever scalar the box holds
    ///    (`false`→"", `null`→"", `int`→decimal, …) — the shape Symfony's non-`strict_types`
    ///    boundaries rely on. See `coerces_to_string_boundary`.
    /// 4. Concrete scalar (`int`/`float`/`bool`/`string`) or Stringable object → a union whose
    ///    codegen representation is a boxed `Mixed` and that offers a compatible member. The
    ///    source is boxed into the union slot and the callee weak-coerces it at use, so no eager
    ///    conversion is emitted. A union source whose every member so flows is accepted too.
    ///
    /// Deliberately kept loud: non-Stringable objects into `string`, `array` into `string`, a
    /// union carrying any array/object member into `string` (`__rt_mixed_cast_string` would turn
    /// it into "" rather than PHP's TypeError), scalars into a scalar-free union (e.g.
    /// `array|null`), and any object mismatch against a concrete (by-offset) object target — none
    /// of which the boundary can realize safely.
    pub(crate) fn weak_boundary_coercion_accepts(
        &self,
        expected: &PhpType,
        actual: &PhpType,
    ) -> bool {
        match expected {
            PhpType::Str => self.coerces_to_string_boundary(actual),
            PhpType::Union(members) if expected.codegen_repr() == PhpType::Mixed => {
                self.scalar_or_stringable_flows_into_boxed_union(members, actual)
            }
            _ => false,
        }
    }

    /// Returns true when `actual` coerces to a plain `string` slot at a value boundary: a
    /// concrete `int`/`float`, or a Stringable object.
    ///
    /// The object case requires a CONCRETE, non-abstract static class with a resolvable
    /// `__toString` (`object_class_has_eager_tostring`): the plain-`string` boundary emits an
    /// eager `(string)$obj` cast, which resolves a DIRECT call to the class's `__toString`
    /// symbol. An abstract-base or interface static type has no such body-linked symbol, so it
    /// stays loud here (a value typed by an abstract/interface Stringable reaches a `string`
    /// slot only through a boxed union in practice — that path dispatches `__toString` virtually
    /// at use). See `weak_boundary_coercion_accepts`.
    ///
    /// A SCALAR-ONLY union whose codegen representation is a boxed `Mixed` cell (e.g.
    /// `string|false`, `string|null`, `string|int`) also coerces here: the plain-`string`
    /// boundary emits the `__rt_mixed_cast_string` tag dispatch (int→decimal, float→ftoa,
    /// bool/`false`→"1"/"", string→persisted copy, null→""), which is exactly PHP weak-mode
    /// `string`-boundary coercion. This is the shape Symfony's non-`strict_types` boundaries rely
    /// on (Dotenv/Path/Yaml `string|false` returns and parameters). It stays loud for any
    /// array/object union member — `__rt_mixed_cast_string` has no tag for those and would
    /// silently yield "" instead of PHP's TypeError — and for a TAGGED-scalar union such as
    /// `int|null` (codegen repr `TaggedScalar`, not `Mixed`), whose null case the `IToStr`
    /// coercion would mis-stringify.
    fn coerces_to_string_boundary(&self, actual: &PhpType) -> bool {
        match actual {
            PhpType::Int | PhpType::Float => true,
            PhpType::Object(name) => self.object_class_has_eager_tostring(name),
            PhpType::Union(members) if actual.codegen_repr() == PhpType::Mixed => {
                members
                    .iter()
                    .all(Self::union_member_weak_coerces_to_string)
            }
            _ => false,
        }
    }

    /// Returns true for a union member that `__rt_mixed_cast_string` dispatches to its exact PHP
    /// weak-mode `string` form: `string`, `int`, `float`, `bool`/`false`, or `null`
    /// (`void`/`never`). Array and object members are excluded — the cast has no tag for them and
    /// would silently yield "" instead of PHP's TypeError, so a union carrying one stays loud.
    fn union_member_weak_coerces_to_string(member: &PhpType) -> bool {
        matches!(
            member,
            PhpType::Str
                | PhpType::Int
                | PhpType::Float
                | PhpType::Bool
                | PhpType::False
                | PhpType::Void
                | PhpType::Never
        )
    }

    /// Returns true when `actual` may flow into a boxed-`Mixed` union target: a concrete scalar
    /// or Stringable object matching a compatible member, or a union source whose every member
    /// so flows. See `weak_boundary_coercion_accepts`.
    fn scalar_or_stringable_flows_into_boxed_union(
        &self,
        members: &[PhpType],
        actual: &PhpType,
    ) -> bool {
        match actual {
            // A CONCRETE scalar argument (including a bare `false`, which PHP coerces to `""`)
            // boxes into the union's `Mixed` slot and weak-coerces at use. This is distinct from a
            // `false` MEMBER of a union SOURCE (a `strpos()`-style sentinel), which stays loud
            // below to preserve the sentinel diagnostic.
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False | PhpType::Str => {
                union_has_scalar_member(members)
            }
            PhpType::Object(name) => {
                members.iter().any(|member| *member == PhpType::Str)
                    && self.object_class_is_stringable(name)
            }
            // A union source flows straight through only when it is ALSO a boxed `Mixed` slot:
            // the boundary copies the box pointer with no conversion. A nullable-int union source
            // (`int|null`) has a TAGGED-SCALAR representation, not `Mixed`, so copying it into a
            // `Mixed` slot would be read as a boxed pointer — a crash — and it must stay loud.
            PhpType::Union(_) if actual.codegen_repr() == PhpType::Mixed => {
                let PhpType::Union(src_members) = actual else {
                    return false;
                };
                src_members
                    .iter()
                    .all(|member| self.boxed_union_source_member_flows(members, member))
            }
            _ => false,
        }
    }

    /// Returns true when one `member` of a union source may flow into a boxed-`Mixed` union
    /// target: null/void, an exact/subtype member match, a scalar coercible to a target scalar
    /// member, or a Stringable object where the target offers `string`.
    fn boxed_union_source_member_flows(&self, target_members: &[PhpType], member: &PhpType) -> bool {
        if matches!(member, PhpType::Void | PhpType::Never | PhpType::Mixed) {
            return true;
        }
        if target_members
            .iter()
            .any(|target_member| self.type_accepts(target_member, member))
        {
            return true;
        }
        match member {
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Str => {
                union_has_scalar_member(target_members)
            }
            PhpType::Object(name) => {
                target_members.iter().any(|tm| *tm == PhpType::Str)
                    && self.object_class_is_stringable(name)
            }
            _ => false,
        }
    }

    /// Returns true if `ty` is a `PhpType::Union` that contains `PhpType::Void`.
    pub(crate) fn union_contains_void(ty: &PhpType) -> bool {
        matches!(ty, PhpType::Union(members) if members.iter().any(|member| *member == PhpType::Void))
    }

    /// Removes `PhpType::Void` from a union type, re-normalizing the result.
    /// If all members are removed, returns `PhpType::Never`. If only one member remains,
    /// returns it directly without wrapping in a union.
    pub(crate) fn strip_void_from_union(&self, ty: &PhpType) -> PhpType {
        match ty {
            PhpType::Union(members) => {
                let filtered: Vec<PhpType> = members
                    .iter()
                    .filter(|member| **member != PhpType::Void)
                    .cloned()
                    .collect();
                self.normalize_union_type(filtered)
            }
            other => other.clone(),
        }
    }

    /// Returns true if `ty` is `Int`, `Bool`, `Void`, `Str`, or a union of only those types.
    /// These types support fast integer-dispatch in `Mixed` value handling.
    pub(crate) fn type_supports_mixed_int_dispatch(&self, ty: &PhpType) -> bool {
        let _ = self;
        match ty {
            PhpType::Int | PhpType::Bool | PhpType::False | PhpType::Void | PhpType::Str => true,
            PhpType::Union(members) => members
                .iter()
                .all(|member| self.type_supports_mixed_int_dispatch(member)),
            _ => false,
        }
    }

    /// Returns true if `ty` is a union type where every member supports mixed-int dispatch.
    pub(crate) fn is_union_with_mixed_int_dispatch(&self, ty: &PhpType) -> bool {
        matches!(ty, PhpType::Union(_)) && self.type_supports_mixed_int_dispatch(ty)
    }

    /// Computes the merged type when assigning `new_ty` to a variable that already has
    /// `existing` type. Returns `Some(merged)` when types are compatible for compound assignment
    /// (e.g., `+=`), or `None` when the types cannot be merged (e.g., two incompatible scalars).
    pub(crate) fn merged_assignment_type(
        &self,
        existing: &PhpType,
        new_ty: &PhpType,
    ) -> Option<PhpType> {
        // A `Mixed` on either side dominates the merge and yields `Mixed`. This must precede the
        // `type_accepts` shortcut below: under gradual typing `type_accepts(existing, Mixed)` is
        // true for any concrete `existing`, so the shortcut would return that narrower `existing`
        // type and silently discard the widening. As a *join* (this helper backs the conditional
        // reassignment and ternary/if branch merges), the least upper bound of a concrete type and
        // a gradual `Mixed` value is `Mixed`, never the concrete side. Discarding it left a
        // reassigned local mis-typed: `$x = <string>; $x = unserialize(...);` inside a branch kept
        // `Str`, so a later `$x->method()` resolved against the string receiver and fell through to
        // the `Int` method-call fallback (Symfony DI `ParentTrait::parent()`).
        if matches!(existing, PhpType::Mixed) || matches!(new_ty, PhpType::Mixed) {
            return Some(PhpType::Mixed);
        }
        if self.type_accepts(existing, new_ty) {
            return Some(existing.clone());
        }
        if matches!(existing, PhpType::Union(_)) {
            return None;
        }
        if existing == new_ty {
            return Some(existing.clone());
        }
        if matches!(existing, PhpType::Array(inner) if matches!(inner.as_ref(), PhpType::Never))
            && matches!(new_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
        {
            return Some(new_ty.clone());
        }
        if matches!(new_ty, PhpType::Array(inner) if matches!(inner.as_ref(), PhpType::Never))
            && matches!(existing, PhpType::Array(_) | PhpType::AssocArray { .. })
        {
            return Some(existing.clone());
        }
        if *new_ty == PhpType::Void {
            return Some(existing.clone());
        }
        if *existing == PhpType::Void {
            return Some(new_ty.clone());
        }
        if matches!(existing, PhpType::Int | PhpType::Bool | PhpType::False | PhpType::Float)
            && matches!(new_ty, PhpType::Int | PhpType::Bool | PhpType::False | PhpType::Float)
        {
            return Some(existing.clone());
        }
        if Self::pointer_types_compatible(existing, new_ty) {
            return Some(match (existing, new_ty) {
                (PhpType::Pointer(Some(left)), PhpType::Pointer(Some(right))) if left == right => {
                    PhpType::Pointer(Some(left.clone()))
                }
                (PhpType::Pointer(None), PhpType::Pointer(Some(tag)))
                | (PhpType::Pointer(Some(tag)), PhpType::Pointer(None)) => {
                    PhpType::Pointer(Some(tag.clone()))
                }
                _ => PhpType::Pointer(None),
            });
        }
        if PhpType::resource_types_compatible(existing, new_ty) {
            return Some(match (existing, new_ty) {
                (PhpType::Resource(Some(left)), PhpType::Resource(Some(right)))
                    if left == right =>
                {
                    PhpType::Resource(Some(left.clone()))
                }
                (PhpType::Resource(None), PhpType::Resource(Some(kind)))
                | (PhpType::Resource(Some(kind)), PhpType::Resource(None)) => {
                    PhpType::Resource(Some(kind.clone()))
                }
                _ => PhpType::Resource(None),
            });
        }
        None
    }

    /// Computes the merged array element type when writing a value of `new_ty` into an
    /// array that already has `existing` element type. Returns `Some(merged)` for compatible
    /// types (same, `Never`, `Mixed`, or compatible objects), or `None` otherwise.
    pub(crate) fn merge_array_element_type(
        &self,
        existing: &PhpType,
        new_ty: &PhpType,
    ) -> Option<PhpType> {
        if existing == new_ty {
            return Some(existing.clone());
        }
        if matches!(existing, PhpType::Never) {
            return Some(new_ty.clone());
        }
        if matches!(new_ty, PhpType::Never) {
            return Some(existing.clone());
        }
        if matches!(existing, PhpType::Mixed) || matches!(new_ty, PhpType::Mixed) {
            return Some(PhpType::Mixed);
        }

        match (existing, new_ty) {
            (PhpType::Object(left), PhpType::Object(right)) => self.common_object_type(left, right),
            _ => None,
        }
    }

    /// Returns true when `actual` may flow into `expected` under PHP's monolithic `array`
    /// type hint: any array-family value (`Array`/`AssocArray`) satisfies any array-family
    /// declared type, regardless of the inferred element key/value types.
    ///
    /// PHP's `array` hint carries no element type and is never element-checked at runtime
    /// (verified on PHP 8.5 in both strict and coercive mode: `function f(array $x)` accepts
    /// `[1,2,3]`, `["a"=>"b"]`, and `[]` identically, and rejects only non-arrays). elephc's
    /// more specific `Array(T)`/`AssocArray{K,V}` element types are whole-program inference
    /// artifacts (from property writes, empty-`[]` defaults, or `@param array` phpdoc), not
    /// PHP constraints, so a differently-typed array actual must still be accepted here.
    ///
    /// Both sides must be array-family: a non-array actual (`Str`/`Int`/`Object`/null) into
    /// an array-family expected keeps failing through the strict predicates, exactly matching
    /// PHP's `array` hint (which TypeErrors on non-arrays, and — unlike `iterable` — even on
    /// `Traversable` objects) in both strict and coercive mode. Codegen is sound because all
    /// PHP arrays share one refcounted, per-slot-tagged runtime representation passed by
    /// pointer, so the inferred element type never changes the ABI or storage layout.
    pub(crate) fn array_family_gradual_accepts(expected: &PhpType, actual: &PhpType) -> bool {
        matches!(
            expected,
            PhpType::Array(_) | PhpType::AssocArray { .. }
        ) && matches!(actual, PhpType::Array(_) | PhpType::AssocArray { .. })
    }

    /// Returns true if `ty` is `PhpType::Array(Box::new(PhpType::Mixed))`, i.e., an
    /// untyped `array` hint without element type specialization.
    pub(crate) fn is_generic_array_hint(ty: &PhpType) -> bool {
        matches!(ty, PhpType::Array(inner) if matches!(inner.as_ref(), PhpType::Mixed))
    }

    /// If `declared_ty` is a generic array hint and `actual_ty` is a concrete array or
    /// assoc-array, returns `actual_ty` (specialization). Otherwise returns `declared_ty`.
    /// Used to sharpen untyped `array` parameters when the actual argument type is known.
    pub(crate) fn specialize_generic_array_hint(
        declared_ty: &PhpType,
        actual_ty: &PhpType,
    ) -> PhpType {
        if Self::is_generic_array_hint(declared_ty)
            && matches!(actual_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
        {
            actual_ty.clone()
        } else {
            declared_ty.clone()
        }
    }

    /// Specializes a bare PHP `array` parameter without pinning object elements to one class.
    ///
    /// PHP does not expose array element generics, so a concrete object element inferred from
    /// one call site is not a valid contract for later calls. Indexed object arrays therefore
    /// stay `Array(Mixed)`, while associative arrays retain their key/storage shape but erase an
    /// object value to `Mixed`. Scalar element types remain specialized for existing inference.
    pub(crate) fn specialize_generic_array_param_hint(
        declared_ty: &PhpType,
        actual_ty: &PhpType,
    ) -> PhpType {
        if !Self::is_generic_array_hint(declared_ty) {
            return declared_ty.clone();
        }
        match actual_ty {
            PhpType::Array(element)
                if matches!(element.as_ref(), PhpType::Object(_))
                    || matches!(element.as_ref(), PhpType::Union(members) if members
                        .iter()
                        .any(|member| matches!(member, PhpType::Object(_)))) =>
            {
                PhpType::Array(Box::new(PhpType::Mixed))
            }
            PhpType::AssocArray { key, value }
                if matches!(value.as_ref(), PhpType::Object(_))
                    || matches!(value.as_ref(), PhpType::Union(members) if members
                        .iter()
                        .any(|member| matches!(member, PhpType::Object(_)))) =>
            {
                PhpType::AssocArray {
                    key: key.clone(),
                    value: Box::new(PhpType::Mixed),
                }
            }
            PhpType::Array(_) | PhpType::AssocArray { .. } => actual_ty.clone(),
            _ => declared_ty.clone(),
        }
    }
}

/// Returns true when `members` contains at least one scalar member (`string`/`int`/`float`/
/// `bool`/`false`) that a weak-mode scalar argument can coerce to when boxed into the union's
/// `Mixed` slot. A scalar-free union (for example `array|null`) returns false so a scalar into
/// it stays a loud type error, matching PHP.
fn union_has_scalar_member(members: &[PhpType]) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            PhpType::Str | PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False
        )
    })
}
