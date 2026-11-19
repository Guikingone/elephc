//! Purpose:
//! Caches the emitted catalog unit texts so a rebuild can skip emitting those functions.
//!
//! Called from:
//! - `crate::codegen::generate_user_asm_from_ir_with_options_and_features()` and
//!   `crate::codegen::finalize_user_asm()`.
//!
//! Key details:
//! - The key is a digest over the post-optimization EIR of every catalog function plus the module
//!   facts codegen reads for them (target, PHP profile, runtime features, source path, class ids).
//!   Post-optimization is deliberate: the optimizer's inliner copies callee bodies into callers, so
//!   skipping a function before the optimizer would change the code of the functions that inline
//!   it. Skipping emission is the boundary that does not perturb anything else.
//! - A hit means "codegen would emit exactly these bytes", so the units are used verbatim and the
//!   three emission loops skip those functions.
//! - The stored units are the pre-visibility texts: the visibility pass runs per build, exactly as
//!   it does for a freshly emitted catalog.
//! - A cache miss or an unreadable entry is a miss, never an error: the compiler emits the catalog
//!   as usual and rewrites the entry.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ir::{print_function, Function, Module};

/// One cached catalog emission.
#[derive(Serialize, Deserialize)]
struct CatalogEmission {
    /// Names of the functions the cached units stand for.
    functions: Vec<String>,
    /// Number of function spans that moved into the units, for the timing note.
    moved_functions: usize,
    /// Symbols that still bind the units to the program, for the timing note.
    program_dependent_symbols: Vec<String>,
    /// Catalog unit texts, in emission order, before the visibility pass.
    units: Vec<String>,
}

/// A catalog emission recovered from the cache.
pub(crate) struct ReusedCatalog {
    /// Functions to skip during emission.
    pub(crate) functions: HashSet<String>,
    /// Unit texts to assemble instead of emitting them.
    pub(crate) units: Vec<String>,
    /// Number of function spans the units hold.
    pub(crate) moved_functions: usize,
    /// Symbols that still bind the units to the program.
    pub(crate) program_dependent_symbols: Vec<String>,
}

/// Returns the catalog functions of `module`, in stable order.
pub(crate) fn catalog_functions(module: &Module) -> Vec<&Function> {
    let mut functions: Vec<&Function> = module
        .functions
        .iter()
        .chain(&module.class_methods)
        .chain(&module.closures)
        .filter(|function| is_catalog_function(function))
        .collect();
    functions.sort_by(|left, right| left.name.cmp(&right.name));
    functions
}

/// Returns whether `function` belongs to the compiler-provided catalog.
///
/// Mirrors the criterion the emission loop uses: no source declaration, minus the property-default
/// thunks, which exist per user class.
pub(crate) fn is_catalog_function(function: &Function) -> bool {
    function.source_name.is_none() && !function.name.starts_with("_class_propinit_")
}

/// Returns the digest that identifies the catalog emission of `module`.
pub(crate) fn signature(module: &Module) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    feed(env!("ELEPHC_RUNTIME_BUILD_ID").as_bytes());
    feed(format!("{:?}", module.target).as_bytes());
    feed(format!("{:?}", module.required_runtime_features).as_bytes());
    feed(crate::codegen::compile_php_version().version_id().to_string().as_bytes());
    feed(module.source_path.as_deref().unwrap_or("").as_bytes());
    let mut classes: Vec<(&String, u64)> = module
        .class_infos
        .iter()
        .map(|(name, info)| (name, info.class_id))
        .collect();
    classes.sort();
    for (name, id) in classes {
        feed(name.as_bytes());
        feed(&id.to_le_bytes());
    }
    for function in catalog_functions(module) {
        feed(function.name.as_bytes());
        feed(print_function(function).as_bytes());
    }
    hash
}

/// Loads the catalog emission for `module` when one was stored for exactly this shape.
pub(crate) fn load(module: &Module) -> Option<ReusedCatalog> {
    if std::env::var_os("ELEPHC_CATALOG_REUSE").is_some_and(|value| value == "0") {
        return None;
    }
    let path = entry_path(signature(module));
    let bytes = fs::read(&path).ok()?;
    let entry: CatalogEmission = serde_json::from_slice(&bytes).ok()?;
    let functions: HashSet<String> = entry.functions.into_iter().collect();
    Some(ReusedCatalog {
        functions,
        units: entry.units,
        moved_functions: entry.moved_functions,
        program_dependent_symbols: entry.program_dependent_symbols,
    })
}

/// Stores the catalog emission of `module` so the next build can skip emitting it.
pub(crate) fn store(
    module: &Module,
    units: &[String],
    moved_functions: usize,
    program_dependent_symbols: &[String],
) {
    if units.is_empty() {
        return;
    }
    let entry = CatalogEmission {
        functions: catalog_functions(module)
            .into_iter()
            .map(|function| function.name.clone())
            .collect(),
        moved_functions,
        program_dependent_symbols: program_dependent_symbols.to_vec(),
        units: units.to_vec(),
    };
    let Ok(bytes) = serde_json::to_vec(&entry) else {
        return;
    };
    let path = entry_path(signature(module));
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&temporary, bytes).is_ok() {
        let _ = fs::rename(&temporary, &path);
    }
}

/// Returns the cache file holding the emission for `signature`.
fn entry_path(signature: u64) -> PathBuf {
    cache_root()
        .join("catalog")
        .join("units")
        .join(format!("{signature:016x}.json"))
}

/// Returns the compiler cache directory, matching the location the runtime object uses.
fn cache_root() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(path).join("elephc")
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".cache").join("elephc")
    } else {
        std::env::temp_dir().join("elephc-cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the catalog criterion excludes property-default thunks.
    #[test]
    fn catalog_criterion_excludes_property_thunks() {
        let mut function = Function::new(
            "_fn_x".to_string(),
            crate::ir::IrType::Void,
            crate::types::PhpType::Void,
        );
        function.source_name = None;
        assert!(is_catalog_function(&function));
        function.name = "_class_propinit_3".to_string();
        assert!(!is_catalog_function(&function));
        function.name = "_fn_y".to_string();
        function.source_name = Some("y".to_string());
        assert!(!is_catalog_function(&function));
    }

    /// Verifies that the entry path is keyed by the signature and lives under the catalog cache.
    #[test]
    fn entry_path_is_keyed_by_signature() {
        let path = entry_path(0xabc);
        assert!(path.ends_with("catalog/units/0000000000000abc.json"), "{path:?}");
    }
}
