//! Purpose:
//! Defines the array, property, object-call, static-call, and native-result
//! methods of the generated-runtime `RuntimeValueOps` adapter.
//!
//! Called from:
//! - The single `RuntimeValueOps for ElephcRuntimeOps` implementation in `super`.
//!
//! Key details:
//! - Temporary argument arrays are released after bridge calls.

macro_rules! impl_collection_call_ops {
    () => {
    /// Creates a boxed Mixed indexed array through the generated runtime wrapper.
    fn array_new(&mut self, capacity: usize) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_array_new(capacity as u64) })
    }

    /// Creates a boxed Mixed indexed array whose payload uses direct string slots.
    fn string_array_new(&mut self, capacity: usize) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_string_array_new(capacity as u64) })
    }

    /// Appends one string to a boxed direct-string indexed array.
    fn string_array_push(
        &mut self,
        array: RuntimeCellHandle,
        value: &str,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        self.string_array_push_bytes(array, value.as_bytes())
    }

    /// Appends raw bytes: the generated wrapper takes a pointer and a length, not UTF-8.
    fn string_array_push_bytes(
        &mut self,
        array: RuntimeCellHandle,
        value: &[u8],
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe {
            __elephc_eval_value_string_array_push(
                array.as_ptr(),
                value.as_ptr(),
                value.len() as u64,
            )
        })
    }

    /// Reads the RAW boxed slot of an indexed Mixed array and reports an invoker by-reference
    /// marker. The layout read here is the runtime's own: a 24-byte header whose first word is the
    /// length, element slots after it, and the element value type in bits 8..15 of the heap kind
    /// word just before the header. Anything that is not boxed-Mixed indexed storage, and any slot
    /// holding something other than a marker, answers `None`.
    fn array_invoker_marker_at(
        &mut self,
        array: RuntimeCellHandle,
        position: usize,
    ) -> Result<Option<(usize, u64)>, EvalStatus> {
        const BOXED_MIXED_VALUE_TYPE: u64 = 7;
        const ARRAY_HEADER_BYTES: usize = 24;
        const TAG_INDEXED_ARRAY: u64 = 4;
        const TAG_INVOKER_REF_CELL: u64 = 11;
        if self.type_tag(array)? != TAG_INDEXED_ARRAY {
            return Ok(None);
        }
        let base = self.raw_value_word(array)? as usize;
        if base == 0 {
            return Ok(None);
        }
        let kind = unsafe { *((base - 8) as *const u64) };
        if (kind >> 8) & 0x7f != BOXED_MIXED_VALUE_TYPE {
            return Ok(None);
        }
        let len = unsafe { *(base as *const u64) } as usize;
        if position >= len {
            return Ok(None);
        }
        let cell = unsafe { *((base + ARRAY_HEADER_BYTES + position * 8) as *const usize) };
        if cell == 0 {
            return Ok(None);
        }
        let cell = RuntimeCellHandle::from_raw(cell as *mut RuntimeCell);
        if self.type_tag(cell)? != TAG_INVOKER_REF_CELL {
            return Ok(None);
        }
        Ok(Some((
            self.raw_value_word(cell)? as usize,
            self.raw_value_high_word(cell)?,
        )))
    }

    /// Creates a boxed indexed array whose payload uses raw object-pointer slots.
    fn object_array_new(&mut self, capacity: usize) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_object_array_new(capacity as u64) })
    }

    /// Appends one boxed object to a raw object-pointer array, retaining it for the array.
    fn object_array_push(
        &mut self,
        array: RuntimeCellHandle,
        object: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe {
            __elephc_eval_value_object_array_push(array.as_ptr(), object.as_ptr())
        })
    }

    /// Creates a boxed Mixed associative array through the generated runtime wrapper.
    fn assoc_new(&mut self, capacity: usize) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_assoc_new(capacity as u64) })
    }

    /// Reads one element from a boxed Mixed array through the generated runtime wrapper.
    fn array_get(
        &mut self,
        array: RuntimeCellHandle,
        index: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_array_get(array.as_ptr(), index.as_ptr()) })
    }

    /// Checks whether a boxed Mixed array contains a normalized PHP key.
    fn array_key_exists(
        &mut self,
        key: RuntimeCellHandle,
        array: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_array_key_exists(key.as_ptr(), array.as_ptr()) })
    }

    /// Returns one foreach-visible key from a boxed Mixed array by iteration position.
    fn array_iter_key(
        &mut self,
        array: RuntimeCellHandle,
        position: usize,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_array_iter_key(array.as_ptr(), position as u64) })
    }

    /// Writes one element to a boxed Mixed array through the generated runtime wrapper.
    fn array_set(
        &mut self,
        array: RuntimeCellHandle,
        index: RuntimeCellHandle,
        value: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe {
            __elephc_eval_value_array_set(array.as_ptr(), index.as_ptr(), value.as_ptr())
        })
    }

    /// Reads a boxed Mixed object property through the generated user helper.
    fn property_get(
        &mut self,
        object: RuntimeCellHandle,
        property: &str,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        Self::handle(unsafe {
            __elephc_eval_value_property_get(
                object.as_ptr(),
                property.as_ptr(),
                property.len() as u64,
                scope_ptr,
                scope_len,
            )
        })
    }

    /// Checks an AOT instance property initialization marker through the generated helper.
    fn property_is_initialized(
        &mut self,
        object: RuntimeCellHandle,
        property: &str,
    ) -> Result<bool, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let initialized = unsafe {
            __elephc_eval_value_property_is_initialized(
                object.as_ptr(),
                property.as_ptr(),
                property.len() as u64,
                scope_ptr,
                scope_len,
            )
        };
        Ok(initialized != 0)
    }

    /// Writes a boxed Mixed object property through the generated user helper.
    fn property_set(
        &mut self,
        object: RuntimeCellHandle,
        property: &str,
        value: RuntimeCellHandle,
    ) -> Result<(), EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let ok = unsafe {
            __elephc_eval_value_property_set(
                object.as_ptr(),
                property.as_ptr(),
                property.len() as u64,
                value.as_ptr(),
                scope_ptr,
                scope_len,
                self.context.cast(),
            )
        };
        if ok == 0 {
            Err(EvalStatus::RuntimeFatal)
        } else {
            Ok(())
        }
    }

    /// Reads an AOT static property through the generated user helper.
    fn static_property_get(
        &mut self,
        class_name: &str,
        property: &str,
    ) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let ptr = unsafe {
            __elephc_eval_value_static_property_get(
                class_name.as_ptr(),
                class_name.len() as u64,
                property.as_ptr(),
                property.len() as u64,
                scope_ptr,
                scope_len,
            )
        };
        if ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(RuntimeCellHandle::from_raw(ptr)))
        }
    }

    /// Checks an AOT static property initialization marker through the generated helper.
    fn static_property_is_initialized(
        &mut self,
        class_name: &str,
        property: &str,
    ) -> Result<bool, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let initialized = unsafe {
            __elephc_eval_value_static_property_is_initialized(
                class_name.as_ptr(),
                class_name.len() as u64,
                property.as_ptr(),
                property.len() as u64,
                scope_ptr,
                scope_len,
            )
        };
        Ok(initialized != 0)
    }

    /// Writes an AOT static property through the generated user helper.
    fn static_property_set(
        &mut self,
        class_name: &str,
        property: &str,
        value: RuntimeCellHandle,
    ) -> Result<bool, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let ok = unsafe {
            __elephc_eval_value_static_property_set(
                class_name.as_ptr(),
                class_name.len() as u64,
                property.as_ptr(),
                property.len() as u64,
                value.as_ptr(),
                scope_ptr,
                scope_len,
            )
        };
        Ok(ok != 0)
    }

    /// Reads an AOT class-like constant through the generated user helper.
    fn class_constant_get(
        &mut self,
        class_name: &str,
        constant: &str,
    ) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let ptr = unsafe {
            __elephc_eval_value_class_constant_get(
                class_name.as_ptr(),
                class_name.len() as u64,
                constant.as_ptr(),
                constant.len() as u64,
                scope_ptr,
                scope_len,
            )
        };
        if ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(RuntimeCellHandle::from_raw(ptr)))
        }
    }

    /// Creates a shallow clone of a boxed Mixed stdClass/eval object through the generated wrapper.
    fn object_clone_shallow(
        &mut self,
        object: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe { __elephc_eval_value_object_clone_shallow(object.as_ptr()) })
    }

    /// Returns the JSON-visible public property count for a boxed Mixed object.
    fn object_property_len(&mut self, object: RuntimeCellHandle) -> Result<usize, EvalStatus> {
        let len = unsafe { __elephc_eval_value_object_property_len(object.as_ptr()) };
        usize::try_from(len).map_err(|_| EvalStatus::RuntimeFatal)
    }

    /// Returns the compiled `(array)` cast of a boxed object the compiled program laid out.
    fn compiled_object_to_array(
        &mut self,
        object: RuntimeCellHandle,
    ) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
        let result = unsafe { __elephc_eval_value_object_to_array(object.as_ptr()) };
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(RuntimeCellHandle::from_raw(result)))
    }

    /// Returns one JSON-visible public property key for a boxed Mixed object.
    fn object_property_iter_key(
        &mut self,
        object: RuntimeCellHandle,
        position: usize,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        Self::handle(unsafe {
            __elephc_eval_value_object_property_iter_key(object.as_ptr(), position as u64)
        })
    }

    /// Calls a boxed Mixed object method through the generated user helper.
    fn method_call(
        &mut self,
        object: RuntimeCellHandle,
        method: &str,
        args: Vec<RuntimeCellHandle>,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let arg_array = Self::arg_array(args)?;
        let result = unsafe {
            __elephc_eval_value_method_call(
                object.as_ptr(),
                method.as_ptr(),
                method.len() as u64,
                arg_array.as_ptr(),
                scope_ptr,
                scope_len,
                self.context.cast(),
            )
        };
        if crate::eval_trace::enabled() && !result.is_null() {
            let words = unsafe { std::slice::from_raw_parts(result.cast::<u64>(), 3) };
            eprintln!(
                "[elephc-eval-trace] phase=native_method_bridge_raw method={method:?} result={result:p} arg_array={:p} result_words=[{:#x}, {:#x}, {:#x}]",
                arg_array.as_ptr(), words[0], words[1], words[2],
            );
        }
        unsafe {
            __elephc_eval_value_release(arg_array.as_ptr());
        }
        if crate::eval_trace::enabled() && !result.is_null() {
            let words = unsafe { std::slice::from_raw_parts(result.cast::<u64>(), 3) };
            let tag = unsafe { __elephc_eval_value_type_tag(result) };
            eprintln!(
                "[elephc-eval-trace] phase=native_method_bridge_post_args_release method={method:?} result={result:p} result_words=[{:#x}, {:#x}, {:#x}] tag={tag:#x}",
                words[0], words[1], words[2],
            );
        }
        self.handle_native_call_result(result)
    }

    /// Calls a boxed Mixed object method and writes its generated result to caller-owned storage.
    fn method_call_out(
        &mut self,
        object: RuntimeCellHandle,
        method: &str,
        args: Vec<RuntimeCellHandle>,
        result_out: &mut RuntimeCellHandle,
    ) -> Result<(), EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let arg_array = Self::arg_array(args)?;
        let result = unsafe {
            __elephc_eval_value_method_call(
                object.as_ptr(),
                method.as_ptr(),
                method.len() as u64,
                arg_array.as_ptr(),
                scope_ptr,
                scope_len,
                self.context.cast(),
            )
        };
        unsafe {
            __elephc_eval_value_release(arg_array.as_ptr());
        }
        if result.is_null() {
            return self.handle_native_call_result(result).map(|result| {
                *result_out = result;
            });
        }
        *result_out = RuntimeCellHandle::from_raw(result);
        Ok(())
    }

    /// Calls an AOT static method through the generated user helper.
    fn static_method_call(
        &mut self,
        class_name: &str,
        method: &str,
        args: Vec<RuntimeCellHandle>,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        let (scope_ptr, scope_len) = self.current_class_scope_abi();
        let arg_array = Self::arg_array(args)?;
        let result = unsafe {
            __elephc_eval_value_static_method_call(
                class_name.as_ptr(),
                class_name.len() as u64,
                method.as_ptr(),
                method.len() as u64,
                arg_array.as_ptr(),
                scope_ptr,
                scope_len,
                self.context.cast(),
            )
        };
        unsafe {
            __elephc_eval_value_release(arg_array.as_ptr());
        }
        self.handle_native_call_result(result)
    }

    /// Converts a native free-function result into eval status, preserving pending throwables.
    fn native_call_result(
        &mut self,
        result: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        self.handle_native_call_result(result.as_ptr())
    }

    };
}

pub(super) use impl_collection_call_ops;
