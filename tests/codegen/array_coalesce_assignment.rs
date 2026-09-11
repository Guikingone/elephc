//! Purpose:
//! End-to-end regressions for array-element null-coalesce assignment.
//!
//! Called from:
//! - `tests/codegen/mod.rs` through the codegen integration suite.

use crate::support::*;

/// Array defaults established with `??=` survive a parent constructor call and
/// assignment into a declared typed property.
#[test]
fn test_array_element_coalesce_defaults_survive_parent_constructor_storage() {
    let out = compile_and_run(
        r#"<?php
class CoalesceDefaultsBase {
    protected array $options;

    public function __construct(array $options) {
        $options['debug_var_name'] ??= 'APP_DEBUG';
        $this->options = $options;
    }

    public function debugVarName(): string {
        return $this->options['debug_var_name'];
    }
}

class CoalesceDefaultsChild extends CoalesceDefaultsBase {
    public function __construct(array $options) {
        $options['env_var_name'] ??= 'APP_ENV';
        parent::__construct($options);
    }

    public function envVarName(): string {
        return $this->options['env_var_name'];
    }
}

$runtime = new CoalesceDefaultsChild([]);
echo $runtime->envVarName(), ':', $runtime->debugVarName();
"#,
    );
    assert_eq!(out, "APP_ENV:APP_DEBUG");
}
