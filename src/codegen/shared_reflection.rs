//! Purpose:
//! Indexes immutable Reflection metadata and caches shared object-materializer labels per module.
//! Keeps generated-code identity distinct from the fresh runtime objects each helper allocates.
//!
//! Called from:
//! - `crate::codegen::shared_state::SharedCodegenState::for_module()` initializes the state.
//! - `crate::codegen::lower_inst::objects::reflection` reserves and reuses materializer labels.
//!
//! Key details:
//! - Every materializer key includes its owner kind and exact semantic payload.
//! - The append-only label cache exposes length checkpoints for subtree and body rollback.
//! - Generator membership is frozen from the final immutable EIR module before emission.

use std::collections::{HashMap, HashSet};

use crate::ir::Module;
use crate::names::php_symbol_key;

/// Concrete builtin owner class allocated by a shared Reflection materializer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ReflectionOwnerKind {
    Class,
    Object,
    Function,
    Method,
    Property,
    Parameter,
    ClassConstant,
    Enum,
    EnumUnitCase,
    EnumBackedCase,
}

impl ReflectionOwnerKind {
    /// Converts a supported builtin Reflection class name into its typed owner identity.
    pub(super) fn from_class_name(class_name: &str) -> Option<Self> {
        match class_name {
            "ReflectionClass" => Some(Self::Class),
            "ReflectionObject" => Some(Self::Object),
            "ReflectionFunction" => Some(Self::Function),
            "ReflectionMethod" => Some(Self::Method),
            "ReflectionProperty" => Some(Self::Property),
            "ReflectionParameter" => Some(Self::Parameter),
            "ReflectionClassConstant" => Some(Self::ClassConstant),
            "ReflectionEnum" => Some(Self::Enum),
            "ReflectionEnumUnitCase" => Some(Self::EnumUnitCase),
            "ReflectionEnumBackedCase" => Some(Self::EnumBackedCase),
            _ => None,
        }
    }
}

/// Exact compile-time literal payload admitted by Reflection constructor resolution.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ReflectionLiteralOperand {
    Text(String),
    Position(i64),
}

/// Collision-free semantic identity for one generated Reflection object-materializer body.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ReflectionMaterializerKey {
    Direct {
        owner: ReflectionOwnerKind,
        operands: Vec<ReflectionLiteralOperand>,
    },
    FullClass {
        owner: ReflectionOwnerKind,
        name: String,
    },
    ShallowClass {
        owner: ReflectionOwnerKind,
        name: String,
    },
    ShallowEnum {
        owner: ReflectionOwnerKind,
        name: String,
    },
    DeclaringFunction {
        owner: ReflectionOwnerKind,
        name: String,
    },
    DeclaringMethod {
        owner: ReflectionOwnerKind,
        declaring_class: Option<String>,
        name: String,
    },
}

/// Reflection-specific immutable indexes and generated-code caches shared across a module.
pub(super) struct SharedReflectionState {
    generator_methods: HashSet<String>,
    materializers: HashMap<ReflectionMaterializerKey, String>,
    materializer_order: Vec<ReflectionMaterializerKey>,
}

impl SharedReflectionState {
    /// Creates empty storage used while `SharedCodegenState` assembles its module indexes.
    pub(super) fn empty() -> Self {
        Self {
            generator_methods: HashSet::new(),
            materializers: HashMap::new(),
            materializer_order: Vec::new(),
        }
    }

    /// Freezes generator membership from the final EIR class-method inventory.
    pub(super) fn for_module(module: &Module) -> Self {
        let generator_methods = module
            .class_methods
            .iter()
            .filter(|function| function.flags.is_generator)
            .map(|function| php_symbol_key(function.name.trim_start_matches('\\')))
            .collect();
        Self {
            generator_methods,
            materializers: HashMap::new(),
            materializer_order: Vec::new(),
        }
    }

    /// Returns whether the canonical class-method name belongs to a generator body.
    pub(super) fn method_is_generator(&self, canonical_name: &str) -> bool {
        self.generator_methods.contains(canonical_name)
    }

    /// Returns the label already reserved for an exactly matching materializer key.
    pub(super) fn materializer_label(&self, key: &ReflectionMaterializerKey) -> Option<String> {
        self.materializers.get(key).cloned()
    }

    /// Appends a materializer reservation before its body is emitted.
    pub(super) fn reserve_materializer(&mut self, key: ReflectionMaterializerKey, label: String) {
        debug_assert!(self.materializer_label(&key).is_none());
        self.materializer_order.push(key.clone());
        self.materializers.insert(key, label);
    }

    /// Returns the append-only cache length used as a rollback checkpoint.
    pub(super) fn materializer_checkpoint(&self) -> usize {
        self.materializer_order.len()
    }

    /// Removes reservations and labels emitted after the supplied checkpoint.
    pub(super) fn rollback_materializers(&mut self, checkpoint: usize) {
        debug_assert!(checkpoint <= self.materializer_order.len());
        while self.materializer_order.len() > checkpoint {
            let key = self
                .materializer_order
                .pop()
                .expect("materializer rollback length was checked");
            self.materializers.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::Target;
    use crate::ir::{Function, IrType};
    use crate::types::PhpType;

    /// Builds a minimal class method for immutable generator-index tests.
    fn method(name: &str, is_generator: bool) -> Function {
        let mut function = Function::new(name.to_string(), IrType::Void, PhpType::Void);
        function.flags.is_generator = is_generator;
        function
    }

    /// Verifies generator membership preserves the previous canonical full-name lookup.
    #[test]
    fn generator_index_uses_canonical_full_method_names() {
        let mut module = Module::new(Target::detect_host());
        module.class_methods = vec![
            method("Ns\\Thing::YieldItems", true),
            method("Ns\\Thing::Plain", false),
            method("\\Other\\Thing::MixedCase", true),
        ];
        let state = SharedReflectionState::for_module(&module);

        assert!(state.method_is_generator("ns\\thing::yielditems"));
        assert!(state.method_is_generator("other\\thing::mixedcase"));
        assert!(!state.method_is_generator("ns\\thing::plain"));
    }

    /// Verifies owner kinds and payload variants cannot alias while exact keys do reuse labels.
    #[test]
    fn materializer_keys_keep_owner_and_payload_identity() {
        let module = Module::new(Target::detect_host());
        let mut state = SharedReflectionState::for_module(&module);
        let class_key = ReflectionMaterializerKey::ShallowClass {
            owner: ReflectionOwnerKind::Class,
            name: "Example".to_string(),
        };
        let enum_key = ReflectionMaterializerKey::ShallowClass {
            owner: ReflectionOwnerKind::Enum,
            name: "Example".to_string(),
        };
        let full_key = ReflectionMaterializerKey::FullClass {
            owner: ReflectionOwnerKind::Class,
            name: "Example".to_string(),
        };

        state.reserve_materializer(class_key.clone(), "class_label".to_string());
        assert_eq!(
            state.materializer_label(&class_key).as_deref(),
            Some("class_label")
        );
        assert!(state.materializer_label(&enum_key).is_none());
        assert!(state.materializer_label(&full_key).is_none());
    }

    /// Verifies length rollback removes a failed subtree without disturbing earlier siblings.
    #[test]
    fn materializer_checkpoint_rolls_back_only_new_entries() {
        let module = Module::new(Target::detect_host());
        let mut state = SharedReflectionState::for_module(&module);
        let sibling = ReflectionMaterializerKey::FullClass {
            owner: ReflectionOwnerKind::Class,
            name: "Sibling".to_string(),
        };
        state.reserve_materializer(sibling.clone(), "sibling".to_string());
        let checkpoint = state.materializer_checkpoint();
        state.reserve_materializer(
            ReflectionMaterializerKey::FullClass {
                owner: ReflectionOwnerKind::Class,
                name: "Failed".to_string(),
            },
            "failed".to_string(),
        );

        state.rollback_materializers(checkpoint);
        assert_eq!(state.materializer_label(&sibling).as_deref(), Some("sibling"));
        assert_eq!(state.materializer_checkpoint(), checkpoint);
    }
}
