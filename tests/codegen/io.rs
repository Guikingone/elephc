//! Purpose:
//! Groups the I/O integration test submodules into the parent suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for printing, files, streams, filesystem, misc, and related suites.

use crate::support::*;

#[path = "io/printing.rs"]
mod printing;
#[path = "io/output_buffering.rs"]
mod output_buffering;
#[path = "io/files.rs"]
mod files;
#[path = "io/streams.rs"]
mod streams;
#[path = "io/compress_wrapper.rs"]
mod compress_wrapper;
#[path = "io/gz_streams.rs"]
mod gz_streams;
#[path = "io/wrapper_read_buffer.rs"]
mod wrapper_read_buffer;
#[path = "io/wrapper_chunk_reads.rs"]
mod wrapper_chunk_reads;
#[path = "io/zlib_string_functions.rs"]
mod zlib_string_functions;
#[path = "io/filesystem.rs"]
mod filesystem;
#[path = "io/misc.rs"]
mod misc;
#[path = "io/stat_ext.rs"]
mod stat_ext;
#[path = "io/paths/mod.rs"]
mod paths;
#[path = "io/modify.rs"]
mod modify;
#[path = "io/streams_ext.rs"]
mod streams_ext;
#[path = "io/stream_context_propagation.rs"]
mod stream_context_propagation;
#[path = "io/symlinks.rs"]
mod symlinks;
