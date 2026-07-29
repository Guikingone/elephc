//! Purpose:
//! Defines the target-independent memory layout and discriminants for opaque PHP resources.
//! The emitted AArch64 and x86_64 helpers share these exact offsets.
//!
//! Called from:
//! - `crate::codegen_support::runtime::resources::registry`.
//! - `crate::codegen_support::runtime::resources::stream`.
//!
//! Key details:
//! - Resource handles pack `generation << 32 | (slot_index + 1)`.
//! - Every registry slot is 64 bytes and every stream state is 320 bytes.
//! - Context states are 32-byte owned aggregates whose child references must
//!   be released before the state allocation itself.

/// Number of bits occupied by the one-based registry slot in an opaque handle.
pub(crate) const HANDLE_INDEX_BITS: u32 = 32;

/// Initial number of registry slots, including the three persistent standard streams.
pub(crate) const INITIAL_REGISTRY_CAPACITY: u64 = 8;

/// Size in bytes of one resource-registry slot.
pub(crate) const RESOURCE_SLOT_SIZE: u64 = 64;

/// Byte offset of the generation word in a registry slot.
pub(crate) const SLOT_GENERATION_OFFSET: i64 = 0;

/// Byte offset of the resource-kind word in a registry slot.
pub(crate) const SLOT_KIND_OFFSET: i64 = 8;

/// Byte offset of the lifecycle-status word in a registry slot.
pub(crate) const SLOT_STATUS_OFFSET: i64 = 16;

/// Byte offset of the strong-reference count in a registry slot.
pub(crate) const SLOT_REFS_OFFSET: i64 = 24;

/// Byte offset of the PHP-visible resource id in a registry slot.
pub(crate) const SLOT_PHP_ID_OFFSET: i64 = 32;

/// Byte offset of the stable resource-state pointer in a registry slot.
pub(crate) const SLOT_STATE_PTR_OFFSET: i64 = 40;

/// Byte offset of the one-based next-free index in a registry slot.
///
/// Live slots reuse this word as their request epoch because a slot cannot be
/// both live and linked into the free list.
pub(crate) const SLOT_NEXT_FREE_OFFSET: i64 = 48;

/// Byte offset of the request epoch that owns a live registry slot.
pub(crate) const SLOT_REQUEST_EPOCH_OFFSET: i64 = SLOT_NEXT_FREE_OFFSET;

/// Byte offset of the resource ownership and persistence flags in a registry slot.
pub(crate) const SLOT_FLAGS_OFFSET: i64 = 56;

/// Empty/free registry-slot kind.
pub(crate) const RESOURCE_KIND_FREE: u64 = 0;

/// Stream registry-slot kind.
pub(crate) const RESOURCE_KIND_STREAM: u64 = 1;

/// Stream-context registry-slot kind.
pub(crate) const RESOURCE_KIND_CONTEXT: u64 = 2;

/// Live resource lifecycle state.
pub(crate) const RESOURCE_STATUS_LIVE: u64 = 1;

/// Closing resource lifecycle state, published before re-entrant cleanup.
pub(crate) const RESOURCE_STATUS_CLOSING: u64 = 2;

/// Closed resource lifecycle state.
pub(crate) const RESOURCE_STATUS_CLOSED: u64 = 3;

/// Marks registry slots whose state storage is owned by the registry.
pub(crate) const RESOURCE_FLAG_OWNS_STATE: u64 = 1;

/// Marks process-persistent registry slots such as STDIN, STDOUT, and STDERR.
pub(crate) const RESOURCE_FLAG_PERSISTENT: u64 = 2;

/// Reference-count sentinel used by persistent standard-stream slots.
pub(crate) const RESOURCE_REFS_IMMORTAL: i64 = -1;

/// Number of persistent standard-stream registry slots.
pub(crate) const STANDARD_STREAM_COUNT: u64 = 3;

/// Size in bytes of one stable stream-state allocation.
pub(crate) const STREAM_STATE_SIZE: u64 = 320;

/// Byte offset of the stream backend kind.
pub(crate) const STREAM_BACKEND_KIND_OFFSET: i64 = 0;

/// Byte offset of the PHP stream-wrapper identity.
pub(crate) const STREAM_WRAPPER_ID_OFFSET: i64 = 8;

/// Byte offset of the optional OS descriptor.
pub(crate) const STREAM_FD_OFFSET: i64 = 16;

/// Byte offset of the owned PHP-visible stream URI pointer.
pub(crate) const STREAM_URI_PTR_OFFSET: i64 = 24;

/// Byte offset of the PHP-visible stream URI byte length.
pub(crate) const STREAM_URI_LEN_OFFSET: i64 = 32;

/// Byte offset of the owned transport-host pointer used by TLS defaults.
pub(crate) const STREAM_CONNECT_HOST_PTR_OFFSET: i64 = 40;

/// Byte offset of the transport-host byte length used by TLS defaults.
pub(crate) const STREAM_CONNECT_HOST_LEN_OFFSET: i64 = 48;

/// Byte offset of the PHP-visible end-of-file state.
pub(crate) const STREAM_EOF_OFFSET: i64 = 56;

/// Byte offset reserved for backend-specific auxiliary state.
pub(crate) const STREAM_BACKEND_AUX_OFFSET: i64 = 64;

/// Byte offset of the PHP-visible stream chunk size, or zero for the 8192-byte default.
pub(crate) const STREAM_CHUNK_SIZE_OFFSET: i64 = 144;

/// Byte offset of stream ownership flags.
pub(crate) const STREAM_OWNERSHIP_FLAGS_OFFSET: i64 = 296;

/// Marks a stream instance whose wrapper definition declared `STREAM_IS_URL`.
pub(crate) const STREAM_STATE_FLAG_IS_URL: u64 = 1 << 8;

/// Backend kind for a directly owned OS descriptor.
pub(crate) const STREAM_BACKEND_FD: u64 = 1;

/// Backend kind for a userspace stream-wrapper instance.
pub(crate) const STREAM_BACKEND_USER_WRAPPER: u64 = 2;

/// Backend kind for a process pipe owned by `popen`.
pub(crate) const STREAM_BACKEND_POPEN: u64 = 3;

/// Backend kind for a native libc `DIR*` iterator.
pub(crate) const STREAM_BACKEND_DIRECTORY: u64 = 4;

/// Backend kind for a buffered Phar write stream.
pub(crate) const STREAM_BACKEND_PHAR_WRITE: u64 = 5;

/// Backend kind for a userspace directory-wrapper instance.
pub(crate) const STREAM_BACKEND_USER_DIRECTORY: u64 = 6;

/// Backend kind for an owned `glob://` iterator state.
pub(crate) const STREAM_BACKEND_GLOB_DIRECTORY: u64 = 7;

/// Size in bytes of one stream-context state allocation.
pub(crate) const CONTEXT_STATE_SIZE: u64 = 32;

/// Byte offset of the retained options hash in a stream-context state.
pub(crate) const CONTEXT_OPTIONS_OFFSET: i64 = 0;

/// Byte offset of the retained params value in a stream-context state.
pub(crate) const CONTEXT_PARAMS_OFFSET: i64 = 8;

/// Byte offset of the retained notification callable in a stream-context state.
pub(crate) const CONTEXT_NOTIFIER_OFFSET: i64 = 16;

/// Byte offset of stream-context state flags.
pub(crate) const CONTEXT_FLAGS_OFFSET: i64 = 24;

const _: () = {
    assert!(RESOURCE_KIND_FREE == 0);
    assert!(SLOT_REQUEST_EPOCH_OFFSET == SLOT_NEXT_FREE_OFFSET);
    assert!(STREAM_WRAPPER_ID_OFFSET == STREAM_BACKEND_KIND_OFFSET + 8);
    assert!(STREAM_URI_PTR_OFFSET == STREAM_FD_OFFSET + 8);
    assert!(STREAM_URI_LEN_OFFSET == STREAM_URI_PTR_OFFSET + 8);
    assert!(STREAM_CONNECT_HOST_PTR_OFFSET == STREAM_URI_LEN_OFFSET + 8);
    assert!(STREAM_CONNECT_HOST_LEN_OFFSET == STREAM_CONNECT_HOST_PTR_OFFSET + 8);
    assert!(STREAM_EOF_OFFSET > STREAM_FD_OFFSET);
    assert!(STREAM_BACKEND_AUX_OFFSET == STREAM_EOF_OFFSET + 8);
    assert!(STREAM_EOF_OFFSET < STREAM_CHUNK_SIZE_OFFSET);
    assert!(STREAM_CHUNK_SIZE_OFFSET < STREAM_OWNERSHIP_FLAGS_OFFSET);
    assert!(CONTEXT_STATE_SIZE == 32);
    assert!(CONTEXT_PARAMS_OFFSET == CONTEXT_OPTIONS_OFFSET + 8);
    assert!(CONTEXT_NOTIFIER_OFFSET == CONTEXT_PARAMS_OFFSET + 8);
    assert!(CONTEXT_FLAGS_OFFSET == CONTEXT_NOTIFIER_OFFSET + 8);
};
