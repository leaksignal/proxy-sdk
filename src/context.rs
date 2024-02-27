use std::any::Any;

use crate::{http::HttpContext, stream::StreamContext};

pub enum Context {
    Http(Box<dyn HttpContext>),
    Stream(Box<dyn StreamContext>),
}

pub trait BaseContext {
    /// Called for access log WASM plugins. Not well supported in this crate. Unclear what context this gets called on.
    fn on_log(&mut self) {}

    /// Called when all processing is complete in the proxy for this context.
    /// If returns true, the context is deleted immediately (i.e. dropped).
    /// If returns false, then the drop is deferred
    fn on_done(&mut self) -> bool {
        true
    }
}

#[allow(unused_variables)]
pub trait RootContext: BaseContext + Any {
    /// If returns true, VM startup is successful (and shall continue)
    /// If returns false, VM startup is a failure and will be aborted.
    fn on_vm_start(&mut self, configuration: Option<Vec<u8>>) -> bool {
        true
    }

    /// If returns true, VM startup is successful (and shall continue)
    /// If returns false, VM startup is a failure and will be aborted.
    fn on_configure(&mut self, configuration: Option<Vec<u8>>) -> bool {
        true
    }

    /// Called every tick period as set by [`crate::time::set_tick_period`]
    fn on_tick(&mut self) {}

    /// Called to initiate a new HTTP or Stream context.
    fn create_context(&mut self) -> Context;
}

impl<R: RootContext> From<Box<R>> for Box<dyn RootContext> {
    fn from(value: Box<R>) -> Self {
        value
    }
}
