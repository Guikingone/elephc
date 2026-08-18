//! Purpose:
//! Coordinates focused unit-test modules for the PHAR bridge.
//!
//! Called from:
//! - `cargo test -p elephc-phar` through Rust's test harness.
//!
//! Key details:
//! - Common archive fixture builders are shared through this parent module.

use super::*;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Write;

struct PharTestTrackingAllocator;

static PHAR_TEST_TRACK_ALLOCATIONS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

static PHAR_TEST_LARGEST_ALLOCATION: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

unsafe impl std::alloc::GlobalAlloc for PharTestTrackingAllocator {
    /// Allocates through the system allocator while recording the largest
    /// single request made inside an explicitly enabled PHAR probe.
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        if PHAR_TEST_TRACK_ALLOCATIONS.load(std::sync::atomic::Ordering::Relaxed) {
            PHAR_TEST_LARGEST_ALLOCATION
                .fetch_max(layout.size(), std::sync::atomic::Ordering::Relaxed);
        }
        unsafe { std::alloc::System.alloc(layout) }
    }

    /// Releases a system allocation created by the tracking allocator.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }

    /// Resizes a system allocation while recording the requested new size.
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: std::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        if PHAR_TEST_TRACK_ALLOCATIONS.load(std::sync::atomic::Ordering::Relaxed) {
            PHAR_TEST_LARGEST_ALLOCATION
                .fetch_max(new_size, std::sync::atomic::Ordering::Relaxed);
        }
        unsafe { std::alloc::System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static PHAR_TEST_ALLOCATOR: PharTestTrackingAllocator = PharTestTrackingAllocator;

mod extraction;
mod fixtures;
mod metadata;
mod mutation;
mod security;
mod signatures;
mod zip_features;

#[allow(unused_imports)]
use fixtures::*;
