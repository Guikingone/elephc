//! Purpose:
//! Emits the opaque, dynamically growing PHP resource registry and initial stream-state helpers.
//! This is the Gate 1 runtime boundary shared by every supported target.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()`.
//!
//! Key details:
//! - Registry slots contain no raw PHP-visible descriptor identity.
//! - Target-specific leaf emitters share the layouts declared in `layout`.

mod context;
pub(crate) mod layout;
mod registry;
mod stream;

use crate::codegen_support::emit::Emitter;

/// Emits the resource-registry lifecycle and stream-state helper entry points.
pub(crate) fn emit_resource_runtime(emitter: &mut Emitter) {
    registry::emit_resource_registry(emitter);
    stream::emit_stream_resources(emitter);
    context::emit_context_resources(emitter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{Arch, Platform, Target};

    /// Verifies every supported architecture exposes the complete Gate 1 helper ABI.
    #[test]
    fn emits_gate_one_resource_entry_points() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_resource_runtime(&mut emitter);
            let asm = emitter.output();
            for label in [
                "__rt_resource_registry_init:",
                "__rt_resource_alloc:",
                "__rt_resource_lookup_any:",
                "__rt_resource_retain:",
                "__rt_resource_release:",
                "__rt_resource_registry_request_reset:",
                "__rt_resource_mark_closing:",
                "__rt_resource_mark_closed:",
                "__rt_resource_id_of_registry:",
                "__rt_resource_is_open:",
                "__rt_resource_kind_if_open:",
                "__rt_stream_adopt_fd:",
                "__rt_stream_state:",
                "__rt_stream_fd:",
                "__rt_stream_chunk_size:",
                "__rt_stream_set_chunk_size:",
                "__rt_stream_close_backend:",
                "__rt_context_state:",
                "__rt_context_destroy_state:",
            ] {
                assert!(asm.contains(label), "{target:?} omitted {label}");
            }
        }
    }

    /// Verifies the public layouts remain compatible with handle and stream ABI contracts.
    #[test]
    fn layouts_match_gate_one_contract() {
        assert_eq!(layout::RESOURCE_SLOT_SIZE, 64);
        assert_eq!(layout::STREAM_STATE_SIZE, 320);
        assert_eq!(layout::STREAM_FD_OFFSET, 16);
        assert_eq!(layout::STREAM_CONTEXT_HANDLE_OFFSET, 80);
        assert_eq!(layout::CONTEXT_STATE_SIZE, 32);
        assert_eq!(layout::CONTEXT_OPTIONS_OFFSET, 0);
        assert_eq!(layout::CONTEXT_PARAMS_OFFSET, 8);
        assert_eq!(layout::CONTEXT_NOTIFIER_OFFSET, 16);
        assert_eq!(layout::CONTEXT_FLAGS_OFFSET, 24);
        assert_eq!(layout::STANDARD_STREAM_COUNT, 3);
    }

    /// Verifies ContextState teardown releases both retained children before parent storage.
    #[test]
    fn context_destructor_releases_children_before_state() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_resource_runtime(&mut emitter);
            let asm = emitter.output();
            let destructor = &asm[asm
                .find("__rt_context_destroy_state:")
                .expect("context destructor label")..];
            let options = destructor
                .find("__rt_decref_any")
                .expect("context options release");
            let notifier = destructor
                .find("__rt_callable_descriptor_release")
                .expect("context notifier release");
            let state = destructor
                .find("__rt_heap_free")
                .expect("context state release");
            assert!(
                options < notifier && notifier < state,
                "{target:?} emitted ContextState teardown out of order"
            );
        }
    }

    /// Verifies request reset clears borrowed bridges, destroys streams first, then drops the default context owner.
    #[test]
    fn request_reset_preserves_contexts_until_stream_teardown_finishes() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_resource_runtime(&mut emitter);
            let asm = emitter.output();
            let reset = &asm[asm
                .find("__rt_resource_registry_request_reset:")
                .expect("request reset label")..asm
                .find("__rt_resource_mark_closing:")
                .expect("next resource helper label")];
            let options = reset
                .find("_stream_context_options")
                .expect("borrowed options bridge clear");
            let notifier = reset
                .find("_stream_notification_callback")
                .expect("borrowed notifier bridge clear");
            let current = reset
                .find("_stream_current_context_handle")
                .expect("borrowed wrapper context clear");
            let stream_phase = reset
                .find("__rt_resource_registry_request_reset_kind_ready")
                .expect("stream-kind phase guard");
            let default_context = reset
                .find("_stream_default_context_handle")
                .expect("default context owner detach");
            let epoch = reset
                .find("_resource_registry_epoch")
                .expect("request epoch advance");
            assert!(
                options < notifier
                    && notifier < current
                    && current < stream_phase
                    && stream_phase < default_context
                    && default_context < epoch,
                "{target:?} emitted request reset phases out of order"
            );
        }
    }

    /// Verifies typed state destructors re-resolve their slot after callbacks may grow the registry.
    #[test]
    fn resource_release_refreshes_slot_after_reentrant_state_teardown() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_resource_runtime(&mut emitter);
            let asm = emitter.output();
            let release = &asm[asm
                .find("__rt_resource_release:")
                .expect("resource release label")..asm
                .find("__rt_resource_registry_request_reset:")
                .expect("request reset label")];
            let refresh = release
                .find("__rt_resource_release_after_state_destroy:")
                .expect("post-destructor slot refresh label");
            let recycle = release
                .find("__rt_resource_release_recycle:")
                .expect("resource recycle label");
            assert!(refresh < recycle, "{target:?} refreshes after recycling");
            let refresh_body = &release[refresh..recycle];
            assert!(
                refresh_body.contains("__rt_resource_lookup_any"),
                "{target:?} does not re-resolve the slot after typed teardown"
            );
            assert!(
                release
                    .matches("__rt_resource_release_after_state_destroy")
                    .count()
                    >= 3,
                "{target:?} does not route both typed destructors through slot refresh"
            );
        }
    }

    /// Verifies x86_64 calls the custom heap allocator with its size in `rax`.
    #[test]
    fn x86_64_resource_allocations_use_heap_runtime_abi() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_resource_runtime(&mut emitter);
        let asm = emitter.output();
        assert!(asm.contains("mov eax, 512"));
        assert!(asm.contains("mov eax, 320"));
        assert!(!asm.contains("mov edi, 512"));
        assert!(!asm.contains("mov edi, 320"));
    }
}
