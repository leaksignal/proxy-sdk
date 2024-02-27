#![allow(clippy::module_inception)]
use std::{
    mem::MaybeUninit,
    ops::{Bound, RangeBounds},
};

#[doc = include_str!("../README.md")]
use log::warn;

mod hostcalls;
pub use hostcalls::call_foreign_function;

mod status;
pub use status::*;

mod dispatcher;
pub use dispatcher::set_root_context_factory;

mod context;
pub use context::*;

mod http_call;
pub use http_call::*;

mod grpc_call;
pub use grpc_call::*;

mod grpc_stream;
pub use grpc_stream::*;

mod http;
pub use http::*;

mod queue;
pub use queue::Queue;

mod shared_data;
pub use shared_data::SharedData;

pub mod property;

mod envoy;

mod stream;
pub use stream::*;

mod upstream;
pub use upstream::Upstream;

mod metrics;
pub use metrics::*;

mod logger;
pub use logger::set_log_level;

#[cfg(target_arch = "wasm32")]
mod rng;

pub mod env;

mod time;
pub use time::*;

mod downcast_box;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[no_mangle]
pub extern "C" fn proxy_abi_version_0_2_1() {}

#[cfg_attr(target_arch = "wasm32", export_name = "malloc")]
#[no_mangle]
pub extern "C" fn proxy_on_memory_allocate(size: usize) -> *mut u8 {
    let mut vec: Vec<MaybeUninit<u8>> = Vec::with_capacity(size);
    unsafe {
        vec.set_len(size);
    }
    let slice = vec.into_boxed_slice();
    Box::into_raw(slice) as *mut u8
}

/// Wipes all thread local state, to be used before any initialization in case of VM reuse in native mode
pub fn reset() {
    dispatcher::reset();
}

pub(crate) fn log_concern<T: Default>(context: &str, result: Result<T, Status>) -> T {
    match result {
        Ok(x) => x,
        Err(e) => {
            warn!("[concern-{context}] {e:?}");
            T::default()
        }
    }
}

pub(crate) fn check_concern<T>(context: &str, result: Result<T, Status>) -> Option<T> {
    match result {
        Ok(x) => Some(x),
        Err(e) => {
            warn!("[concern-{context}] {e:?}");
            None
        }
    }
}

pub(crate) fn calculate_range(range: impl RangeBounds<usize>, limit: usize) -> (usize, usize) {
    let start = match range.start_bound() {
        Bound::Included(x) => *x,
        Bound::Excluded(x) => x.saturating_sub(1),
        Bound::Unbounded => 0,
    };
    let size = match range.end_bound() {
        Bound::Included(x) => *x + 1,
        Bound::Excluded(x) => *x,
        Bound::Unbounded => limit,
    }
    .min(limit)
    .saturating_sub(start);
    (start, size)
}
