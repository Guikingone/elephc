//! Purpose:
//! Tests decoding nested object and array defaults from generated callable ABI
//! records.
//!
//! Called from:
//! - `cargo test -p elephc-magician` through Rust's test harness.
//!
//! Key details:
//! - Nested, named, and maximum-count records are covered with bounded encodings.

use super::*;

/// Deferred errors register as context-owned defaults and invalid indexes or buffers fail closed.
#[test]
fn register_native_deferred_default_errors_preserves_signatures() {
    let mut context = ElephcEvalContext::new();
    let key = b"Known::value";
    let class = b"Known";
    let message = b"Cannot access parent in this scope";
    unsafe {
        assert_eq!(__elephc_eval_register_native_method(&mut context, key.as_ptr(), key.len() as u64, 1), 1);
        assert_eq!(__elephc_eval_register_native_static_method(&mut context, key.as_ptr(), key.len() as u64, 1), 1);
        assert_eq!(__elephc_eval_register_native_constructor(&mut context, class.as_ptr(), class.len() as u64, 1), 1);
        assert_eq!(__elephc_eval_register_native_method_param_default_error(&mut context, key.as_ptr(), key.len() as u64, 0, message.as_ptr(), message.len() as u64), 1);
        assert_eq!(__elephc_eval_register_native_static_method_param_default_error(&mut context, key.as_ptr(), key.len() as u64, 0, message.as_ptr(), message.len() as u64), 1);
        assert_eq!(__elephc_eval_register_native_constructor_param_default_error(&mut context, class.as_ptr(), class.len() as u64, 0, message.as_ptr(), message.len() as u64), 1);
        assert_eq!(__elephc_eval_register_native_method_param_default_error(&mut context, key.as_ptr(), key.len() as u64, 1, message.as_ptr(), message.len() as u64), 0);
        assert_eq!(__elephc_eval_register_native_method_param_default_error(&mut context, key.as_ptr(), key.len() as u64, 0, std::ptr::null(), 1), 0);
        assert_eq!(__elephc_eval_register_native_constructor_param_default_error(std::ptr::null_mut(), class.as_ptr(), class.len() as u64, 0, message.as_ptr(), message.len() as u64), 0);
    }
    let expected = NativeCallableDefault::DeferredError(String::from_utf8(message.to_vec()).unwrap());
    assert_eq!(context.native_method_signature("Known", "value").unwrap().param_default(0), Some(&expected));
    assert_eq!(context.native_static_method_signature("Known", "value").unwrap().param_default(0), Some(&expected));
    assert_eq!(context.native_constructor_signature("Known").unwrap().param_default(0), Some(&expected));
}

/// Verifies native AOT object defaults can carry nested object constructor args.
#[test]
fn register_native_object_default_decodes_nested_objects() {
    let mut ctx = ElephcEvalContext::new();
    let class = b"KnownClass";
    let nested_args = vec![NativeCallableObjectDefaultArg::positional(
        NativeCallableDefault::Object {
            class_name: "InnerDefault".to_string(),
            args: vec![NativeCallableObjectDefaultArg::positional(
                NativeCallableDefault::String("leaf".to_string()),
            )],
        },
    )];
    let expected_default = NativeCallableDefault::Object {
        class_name: "OuterDefault".to_string(),
        args: nested_args.clone(),
    };
    let spec = native_object_default_record("OuterDefault", &nested_args);

    let constructor_registered = unsafe {
        __elephc_eval_register_native_constructor(&mut ctx, class.as_ptr(), class.len() as u64, 1)
    };
    let default_registered = unsafe {
        __elephc_eval_register_native_constructor_param_default_object(
            &mut ctx,
            class.as_ptr(),
            class.len() as u64,
            0,
            spec.as_ptr(),
            spec.len() as u64,
        )
    };

    assert_eq!(constructor_registered, 1);
    assert_eq!(default_registered, 1);
    assert_eq!(
        ctx.native_constructor_signature("knownclass")
            .expect("constructor metadata")
            .param_default(0),
        Some(&expected_default)
    );
}

/// Verifies native AOT object defaults can carry named constructor args.
#[test]
fn register_native_object_default_decodes_named_args() {
    let mut ctx = ElephcEvalContext::new();
    let class = b"KnownClass";
    let args = vec![
        NativeCallableObjectDefaultArg::named(
            "right",
            NativeCallableDefault::String("R".to_string()),
        ),
        NativeCallableObjectDefaultArg::named(
            "left",
            NativeCallableDefault::String("L".to_string()),
        ),
    ];
    let expected_default = NativeCallableDefault::Object {
        class_name: "NamedDefault".to_string(),
        args: args.clone(),
    };
    let spec = native_object_default_record("NamedDefault", &args);

    let constructor_registered = unsafe {
        __elephc_eval_register_native_constructor(&mut ctx, class.as_ptr(), class.len() as u64, 1)
    };
    let default_registered = unsafe {
        __elephc_eval_register_native_constructor_param_default_object(
            &mut ctx,
            class.as_ptr(),
            class.len() as u64,
            0,
            spec.as_ptr(),
            spec.len() as u64,
        )
    };

    assert_eq!(constructor_registered, 1);
    assert_eq!(default_registered, 1);
    assert_eq!(
        ctx.native_constructor_signature("knownclass")
            .expect("constructor metadata")
            .param_default(0),
        Some(&expected_default)
    );
}

/// Verifies native AOT object defaults decode the full u8 constructor argument range.
#[test]
fn register_native_object_default_decodes_full_u8_arg_count() {
    let mut ctx = ElephcEvalContext::new();
    let class = b"KnownClass";
    let args = (0..TEST_MAX_NATIVE_OBJECT_DEFAULT_ARGS)
        .map(|index| {
            let value = NativeCallableDefault::String(format!("arg{}", index));
            NativeCallableObjectDefaultArg::positional(value)
        })
        .collect::<Vec<_>>();
    let expected_default = NativeCallableDefault::Object {
        class_name: "LargeDefault".to_string(),
        args: args.clone(),
    };
    let spec = native_object_default_record("LargeDefault", &args);

    let constructor_registered = unsafe {
        __elephc_eval_register_native_constructor(&mut ctx, class.as_ptr(), class.len() as u64, 1)
    };
    let default_registered = unsafe {
        __elephc_eval_register_native_constructor_param_default_object(
            &mut ctx,
            class.as_ptr(),
            class.len() as u64,
            0,
            spec.as_ptr(),
            spec.len() as u64,
        )
    };

    assert_eq!(constructor_registered, 1);
    assert_eq!(default_registered, 1);
    assert_eq!(
        ctx.native_constructor_signature("knownclass")
            .expect("constructor metadata")
            .param_default(0),
        Some(&expected_default)
    );
}

/// Verifies native AOT array defaults can carry keyed and nested default values.
#[test]
fn register_native_array_default_decodes_nested_values() {
    let mut ctx = ElephcEvalContext::new();
    let method = b"KnownClass::items";
    let class = b"KnownClass";
    let property = b"KnownClass::items";
    let elements = vec![
        NativeCallableArrayDefaultElement::keyed(
            NativeCallableArrayDefaultKey::String("left".to_string()),
            NativeCallableDefault::String("L".to_string()),
        ),
        NativeCallableArrayDefaultElement::keyed(
            NativeCallableArrayDefaultKey::Int(2),
            NativeCallableDefault::Array(vec![NativeCallableArrayDefaultElement::positional(
                NativeCallableDefault::Int(7),
            )]),
        ),
        NativeCallableArrayDefaultElement::positional(NativeCallableDefault::Object {
            class_name: "ArrayDefaultDep".to_string(),
            args: vec![NativeCallableObjectDefaultArg::named(
                "label",
                NativeCallableDefault::String("dep".to_string()),
            )],
        }),
    ];
    let expected_default = NativeCallableDefault::Array(elements.clone());
    let spec = native_array_default_record(&elements);

    let method_registered = unsafe {
        __elephc_eval_register_native_method(&mut ctx, method.as_ptr(), method.len() as u64, 1)
    };
    let method_default_registered = unsafe {
        __elephc_eval_register_native_method_param_default_array(
            &mut ctx,
            method.as_ptr(),
            method.len() as u64,
            0,
            spec.as_ptr(),
            spec.len() as u64,
        )
    };
    let constructor_registered = unsafe {
        __elephc_eval_register_native_constructor(&mut ctx, class.as_ptr(), class.len() as u64, 1)
    };
    let constructor_default_registered = unsafe {
        __elephc_eval_register_native_constructor_param_default_array(
            &mut ctx,
            class.as_ptr(),
            class.len() as u64,
            0,
            spec.as_ptr(),
            spec.len() as u64,
        )
    };
    let property_default_registered = unsafe {
        __elephc_eval_register_native_property_default_array(
            &mut ctx,
            property.as_ptr(),
            property.len() as u64,
            spec.as_ptr(),
            spec.len() as u64,
        )
    };

    assert_eq!(method_registered, 1);
    assert_eq!(method_default_registered, 1);
    assert_eq!(constructor_registered, 1);
    assert_eq!(constructor_default_registered, 1);
    assert_eq!(property_default_registered, 1);
    assert_eq!(
        ctx.native_method_signature("knownclass", "ITEMS")
            .expect("method metadata")
            .param_default(0),
        Some(&expected_default)
    );
    assert_eq!(
        ctx.native_constructor_signature("knownclass")
            .expect("constructor metadata")
            .param_default(0),
        Some(&expected_default)
    );
    assert_eq!(
        ctx.native_property_default("KnownClass", "items"),
        Some(expected_default)
    );
}
