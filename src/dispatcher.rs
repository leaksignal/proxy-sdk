#![allow(clippy::type_complexity)]

use log::{debug, error, warn};

use crate::{
    check_concern,
    context::{Context, RootContext},
    downcast_box::DowncastBox,
    grpc_call::GrpcCallResponse,
    grpc_stream::{GrpcStreamClose, GrpcStreamHandle, GrpcStreamMessage},
    hostcalls::{self, BufferType},
    http::{
        HttpContext, RequestBody, RequestHeaders, RequestTrailers, ResponseBody, ResponseHeaders,
        ResponseTrailers,
    },
    http_call::HttpCallResponse,
    property::envoy::Attributes,
    queue::Queue,
    stream::{DownstreamData, StreamClose, StreamContext, UpstreamData},
    CloseType, FilterDataStatus, FilterHeadersStatus, FilterStreamStatus, FilterTrailersStatus,
    GrpcCode,
};
use std::{
    cell::{Cell, RefCell, RefMut},
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

#[cfg(feature = "stream-metadata")]
pub use crate::grpc_stream::{GrpcStreamInitialMetadata, GrpcStreamTrailingMetadata};

thread_local! {
    static DISPATCHER: Dispatcher = Dispatcher::new();
}
static DISPATCHER_GEN: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn reset() {
    DISPATCHER_GEN.fetch_add(1, Ordering::Relaxed);
    *ROOT_INIT.lock().unwrap() = None;
}

pub(crate) fn root_id() -> u32 {
    DISPATCHER.with(|x| x.active_root_id.get())
}

fn dispatch<F, R>(f: F) -> R
where
    F: FnOnce(&Dispatcher) -> R,
{
    DISPATCHER.with(|d| {
        let current_gen = DISPATCHER_GEN.load(Ordering::Relaxed);
        if d.generation.get() != current_gen {
            d.generation.set(current_gen);
            d.reset();
        }
        f(d)
    })
}

static ROOT_INIT: Mutex<Option<Box<dyn Fn() -> DowncastBox<dyn RootContext> + Send + Sync>>> =
    Mutex::new(None);

struct HttpCallback {
    context_id: u32,
    root_context_id: u32,
    callback: Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &HttpCallResponse)>,
}

struct GrpcCallback {
    context_id: u32,
    root_context_id: u32,
    callback: Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &GrpcCallResponse)>,
}

#[derive(Default)]
struct GrpcStreamCallback {
    context_id: u32,
    root_context_id: u32,
    close: Option<Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &GrpcStreamClose)>>,
    message: Option<
        Box<dyn FnMut(&mut DowncastBox<dyn RootContext>, GrpcStreamHandle, &GrpcStreamMessage)>,
    >,
    #[cfg(feature = "stream-metadata")]
    initial_meta: Option<
        Box<
            dyn FnMut(
                &mut DowncastBox<dyn RootContext>,
                GrpcStreamHandle,
                &GrpcStreamInitialMetadata,
            ),
        >,
    >,
    #[cfg(feature = "stream-metadata")]
    trailer_meta: Option<
        Box<
            dyn FnMut(
                &mut DowncastBox<dyn RootContext>,
                GrpcStreamHandle,
                &GrpcStreamTrailingMetadata,
            ),
        >,
    >,
}

struct StreamInfo {
    parent_context_id: u32,
    data: Box<dyn StreamContext>,
}

struct HttpStreamInfo {
    parent_context_id: u32,
    data: Box<dyn HttpContext>,
}

struct RootInfo {
    data: DowncastBox<dyn RootContext>,
}

#[derive(Default)]
struct Dispatcher {
    roots: RefCell<HashMap<u32, RootInfo>>,
    streams: RefCell<HashMap<u32, StreamInfo>>,
    http_streams: RefCell<HashMap<u32, HttpStreamInfo>>,
    http_callbacks: RefCell<HashMap<u32, HttpCallback>>,
    grpc_callbacks: RefCell<HashMap<u32, GrpcCallback>>,
    grpc_streams: RefCell<HashMap<u32, GrpcStreamCallback>>,
    queue_callbacks:
        RefCell<HashMap<u32, Box<dyn FnMut(&mut DowncastBox<dyn RootContext>, Queue)>>>,
    active_id: Cell<u32>,
    active_root_id: Cell<u32>,
    generation: Cell<usize>,
}

impl Dispatcher {
    fn reset(&self) {
        self.roots.borrow_mut().clear();
        self.streams.borrow_mut().clear();
        self.http_streams.borrow_mut().clear();
        self.http_callbacks.borrow_mut().clear();
        self.grpc_callbacks.borrow_mut().clear();
        self.grpc_streams.borrow_mut().clear();
        self.queue_callbacks.borrow_mut().clear();
        self.roots.borrow_mut().clear();
        self.active_id.set(0);
        self.active_root_id.set(0);
    }

    fn root<'a>(
        roots: &'a mut RefMut<'_, HashMap<u32, RootInfo>>,
        root_context_id: u32,
    ) -> &'a mut DowncastBox<dyn RootContext> {
        roots.entry(root_context_id).or_insert_with(|| RootInfo {
            data: ROOT_INIT
                .lock()
                .unwrap()
                .as_ref()
                .expect("missing root_context_factory")(),
        });
        &mut roots.get_mut(&root_context_id).unwrap().data
    }
}

/// Sets root context factory. Should be called from _init. Can only be called once.
pub fn set_root_context_factory<R: RootContext + 'static>(root: fn() -> R) {
    *ROOT_INIT.lock().unwrap() = Some(Box::new(move || DowncastBox::new(Box::new(root()))));
}

pub(crate) fn register_http_callback(
    token: u32,
    callback: Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &HttpCallResponse)>,
) {
    dispatch(|d| {
        d.http_callbacks.borrow_mut().insert(
            token,
            HttpCallback {
                context_id: d.active_id.get(),
                root_context_id: d.active_root_id.get(),
                callback,
            },
        )
    });
}

pub(crate) fn register_grpc_callback(
    token: u32,
    callback: Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &GrpcCallResponse)>,
) {
    dispatch(|d| {
        d.grpc_callbacks.borrow_mut().insert(
            token,
            GrpcCallback {
                context_id: d.active_id.get(),
                root_context_id: d.active_root_id.get(),
                callback,
            },
        )
    });
}

#[cfg(feature = "stream-metadata")]
pub(crate) fn register_grpc_stream_initial_meta(
    token: u32,
    callback: Box<
        dyn FnMut(&mut DowncastBox<dyn RootContext>, GrpcStreamHandle, &GrpcStreamInitialMetadata),
    >,
) {
    dispatch(|d| {
        let context_id = d.d.active_id.get();
        let root_context_id = d.active_root_id.get();
        match d.grpc_streams.borrow_mut().entry(token) {
            Entry::Occupied(entry) if entry.get().context_id != context_id => {
                error!(
                    "mismatch in context for register_grpc_stream_initial_meta! {} != {}",
                    entry.get().context_id,
                    context_id
                );
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().initial_meta = Some(callback);
            }
            Entry::Vacant(entry) => {
                entry.insert(GrpcStreamCallback {
                    context_id,
                    root_context_id,
                    initial_meta: Some(callback),
                    ..Default::default()
                });
            }
        }
    });
}

pub(crate) fn register_grpc_stream_message(
    token: u32,
    callback: Box<
        dyn FnMut(&mut DowncastBox<dyn RootContext>, GrpcStreamHandle, &GrpcStreamMessage),
    >,
) {
    dispatch(|d| {
        let context_id = d.active_id.get();
        let root_context_id = d.active_root_id.get();
        match d.grpc_streams.borrow_mut().entry(token) {
            Entry::Occupied(entry) if entry.get().context_id != context_id => {
                error!(
                    "mismatch in context for register_grpc_stream_message! {} != {}",
                    entry.get().context_id,
                    context_id
                );
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().message = Some(callback);
            }
            Entry::Vacant(entry) => {
                entry.insert(GrpcStreamCallback {
                    context_id,
                    root_context_id,
                    message: Some(callback),
                    ..Default::default()
                });
            }
        }
    });
}

#[cfg(feature = "stream-metadata")]
pub(crate) fn register_grpc_stream_trailing_metadata(
    token: u32,
    callback: Box<
        dyn FnMut(&mut DowncastBox<dyn RootContext>, GrpcStreamHandle, &GrpcStreamTrailingMetadata),
    >,
) {
    dispatch(|d| {
        let context_id = d.active_id.get();
        let root_context_id = d.active_root_id.get();
        match d.grpc_streams.borrow_mut().entry(token) {
            Entry::Occupied(entry) if entry.get().context_id != context_id => {
                error!(
                    "mismatch in context for register_grpc_stream_trailing_metadata! {} != {}",
                    entry.get().context_id,
                    context_id
                );
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().trailer_meta = Some(callback);
            }
            Entry::Vacant(entry) => {
                entry.insert(GrpcStreamCallback {
                    context_id,
                    root_context_id,
                    trailer_meta: Some(callback),
                    ..Default::default()
                });
            }
        }
    });
}

pub(crate) fn register_grpc_stream_close(
    token: u32,
    callback: Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &GrpcStreamClose)>,
) {
    dispatch(|d| {
        let context_id = d.active_id.get();
        let root_context_id = d.active_root_id.get();
        match d.grpc_streams.borrow_mut().entry(token) {
            Entry::Occupied(entry) if entry.get().context_id != context_id => {
                error!(
                    "mismatch in context for register_grpc_stream_close! {} != {}",
                    entry.get().context_id,
                    context_id
                );
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().close = Some(callback);
            }
            Entry::Vacant(entry) => {
                entry.insert(GrpcStreamCallback {
                    context_id,
                    root_context_id,
                    close: Some(callback),
                    ..Default::default()
                });
            }
        }
    })
}

pub(crate) fn register_queue_callback<R: RootContext + 'static>(
    token: u32,
    mut callback: impl FnMut(&mut R, Queue) + 'static,
) {
    dispatch(|d| {
        d.queue_callbacks.borrow_mut().insert(
            token,
            Box::new(move |root, queue| {
                callback(
                    root.as_any_mut().downcast_mut().expect("invalid root type"),
                    queue,
                )
            }),
        );
    })
}

struct EffectiveContext {
    name: &'static str,
    prior: u32,
    prior_root: u32,
}

impl EffectiveContext {
    pub fn enter(id: u32, root_id: u32, name: &'static str) -> Option<Self> {
        if let Err(e) = hostcalls::set_effective_context(id) {
            debug!("failed to assume context {root_id}/{id} for {name}: {e:?}");
            return None;
        };
        let (prior, prior_root) = dispatch(|d| {
            let prior = d.active_id.get();
            d.active_id.set(id);
            let prior_root = d.active_root_id.get();
            d.active_root_id.set(root_id);
            (prior, prior_root)
        });
        Some(Self {
            name,
            prior,
            prior_root,
        })
    }
}

impl Drop for EffectiveContext {
    fn drop(&mut self) {
        if let Err(e) = hostcalls::set_effective_context(self.prior) {
            debug!("failed to reset context for {}: {e:?}", self.name);
        };
        dispatch(|d| {
            d.active_id.set(self.prior);
            d.active_root_id.set(self.prior_root);
        });
    }
}

impl Dispatcher {
    fn new() -> Dispatcher {
        Self::default()
    }

    fn do_create_subcontext(&self, root_context_id: u32, context_id: u32) {
        let mut roots = self.roots.borrow_mut();
        let root = Self::root(&mut roots, root_context_id);
        match root.create_context() {
            Context::Http(context) => {
                if self
                    .http_streams
                    .borrow_mut()
                    .insert(
                        context_id,
                        HttpStreamInfo {
                            parent_context_id: root_context_id,
                            data: context,
                        },
                    )
                    .is_some()
                {
                    warn!("reused context_id without proper cleanup");
                }
            }
            Context::Stream(context) => {
                if self
                    .streams
                    .borrow_mut()
                    .insert(
                        context_id,
                        StreamInfo {
                            parent_context_id: root_context_id,
                            data: context,
                        },
                    )
                    .is_some()
                {
                    warn!("reused context_id without proper cleanup");
                }
            }
        }
    }

    fn on_create_context(&self, context_id: u32, parent_context_id: u32) {
        if parent_context_id == 0 {
            let mut roots = self.roots.borrow_mut();
            Self::root(&mut roots, context_id);
        } else if self.roots.borrow().contains_key(&parent_context_id) {
            self.do_create_subcontext(parent_context_id, context_id);
        } else {
            warn!("attempted to create context {context_id} under unknown context {parent_context_id}");
        }
    }

    fn on_done(&self, context_id: u32) -> bool {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            self.active_root_id.set(http_stream.parent_context_id);
            http_stream.data.on_done()
        } else if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            self.active_root_id.set(stream.parent_context_id);
            stream.data.on_done()
        } else if self.roots.borrow().contains_key(&context_id) {
            self.active_id.set(context_id);
            self.active_root_id.set(context_id);
            let mut roots = self.roots.borrow_mut();
            Self::root(&mut roots, context_id).on_done()
        } else {
            warn!("on_done called on unknown context: {context_id}");
            true
        }
    }

    fn on_log(&self, context_id: u32) {
        if let Some(http_stream) = self.http_streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            self.active_root_id.set(http_stream.parent_context_id);
            http_stream.data.on_log();
        } else if let Some(stream) = self.streams.borrow_mut().get_mut(&context_id) {
            self.active_id.set(context_id);
            self.active_root_id.set(stream.parent_context_id);
            stream.data.on_log();
        } else if self.roots.borrow().contains_key(&context_id) {
            self.active_id.set(context_id);
            self.active_root_id.set(context_id);
            let mut roots = self.roots.borrow_mut();
            Self::root(&mut roots, context_id).on_log();
        } else {
            warn!("on_log called on unknown context: {context_id}");
        }
    }

    fn on_delete(&self, context_id: u32) {
        if self.http_streams.borrow_mut().remove(&context_id).is_some() {
            return;
        }
        if self.streams.borrow_mut().remove(&context_id).is_some() {
            return;
        }
        if self.roots.borrow_mut().remove(&context_id).is_some() {
            return;
        }
        warn!("deleting unknown context_id {context_id}");
    }

    fn on_vm_start(&self, context_id: u32, vm_configuration_size: usize) -> bool {
        if !self.roots.borrow().contains_key(&context_id) {
            warn!("received on_vm_start for non-root-context: {context_id}");
            return true;
        }
        let Some(configuration) = check_concern(
            "vm-start-config",
            hostcalls::get_buffer(BufferType::VmConfiguration, 0, vm_configuration_size),
        ) else {
            return false;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context_id);
        let mut roots = self.roots.borrow_mut();
        Self::root(&mut roots, context_id).on_vm_start(configuration)
    }

    fn on_configure(&self, context_id: u32, plugin_configuration_size: usize) -> bool {
        if !self.roots.borrow().contains_key(&context_id) {
            warn!("received on_configure for non-root-context: {context_id}");
            return true;
        }
        let Some(configuration) = check_concern(
            "configure-fetch",
            hostcalls::get_buffer(
                BufferType::PluginConfiguration,
                0,
                plugin_configuration_size,
            ),
        ) else {
            return false;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context_id);
        let mut roots = self.roots.borrow_mut();
        Self::root(&mut roots, context_id).on_configure(configuration)
    }

    fn on_tick(&self, context_id: u32) {
        if !self.roots.borrow().contains_key(&context_id) {
            warn!("received on_tick for non-root-context: {context_id}");
            return;
        }
        self.active_id.set(context_id);
        self.active_root_id.set(context_id);
        let mut roots = self.roots.borrow_mut();
        Self::root(&mut roots, context_id).on_tick();
    }

    fn on_queue_ready(&self, context_id: u32, queue_id: u32) {
        if !self.roots.borrow().contains_key(&context_id) {
            warn!("received on_queue_ready for non-root-context: {context_id}");
            return;
        }
        if let Some(callback) = self.queue_callbacks.borrow_mut().get_mut(&queue_id) {
            let mut roots = self.roots.borrow_mut();
            callback(
                &mut roots.get_mut(&context_id).unwrap().data,
                Queue(queue_id),
            );
        }
    }

    fn on_new_connection(&self, context_id: u32) -> FilterStreamStatus {
        let mut streams = self.streams.borrow_mut();
        let stream = if let Some(context) = streams.get_mut(&context_id) {
            context
        } else {
            // self.do_create_subcontext(context_id);
            // let Some(context) = self.streams.get_mut(&context_id) else {
            warn!(
                "no http context found for context (and was not implicitly created): {context_id}"
            );
            return FilterStreamStatus::Continue;
            // };
            // context
        };
        self.active_id.set(context_id);
        self.active_root_id.set(stream.parent_context_id);
        stream.data.on_new_connection()
    }

    fn on_downstream_data(
        &self,
        context_id: u32,
        data_size: usize,
        end_of_stream: bool,
    ) -> FilterStreamStatus {
        let mut streams = self.streams.borrow_mut();
        let Some(stream) = streams.get_mut(&context_id) else {
            return FilterStreamStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(stream.parent_context_id);
        stream.data.on_downstream_data(&DownstreamData {
            data_size,
            end_of_stream,
            attributes: Attributes::get(),
        })
    }

    fn on_downstream_close(&self, context_id: u32, close_type: CloseType) {
        let mut streams = self.streams.borrow_mut();
        let Some(stream) = streams.get_mut(&context_id) else {
            return;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(stream.parent_context_id);
        stream.data.on_downstream_close(&StreamClose {
            close_type,
            attributes: Attributes::get(),
        })
    }

    fn on_upstream_data(
        &self,
        context_id: u32,
        data_size: usize,
        end_of_stream: bool,
    ) -> FilterStreamStatus {
        let mut streams = self.streams.borrow_mut();
        let Some(stream) = streams.get_mut(&context_id) else {
            return FilterStreamStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(stream.parent_context_id);
        stream.data.on_upstream_data(&UpstreamData {
            data_size,
            end_of_stream,
            attributes: Attributes::get(),
        })
    }

    fn on_upstream_close(&self, context_id: u32, close_type: CloseType) {
        let mut streams = self.streams.borrow_mut();
        let Some(stream) = streams.get_mut(&context_id) else {
            return;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(stream.parent_context_id);
        stream.data.on_upstream_close(&StreamClose {
            close_type,
            attributes: Attributes::get(),
        })
    }

    fn on_http_request_headers(
        &self,
        context_id: u32,
        header_count: usize,
        end_of_stream: bool,
    ) -> FilterHeadersStatus {
        let mut http_streams = self.http_streams.borrow_mut();
        let context = if let Some(context) = http_streams.get_mut(&context_id) {
            context
        } else {
            // self.do_create_subcontext(context_id);
            // let Some(context) = self.http_streams.get_mut(&context_id) else {
            warn!("no http context found for on_http_request_headers (and was not implicitly created): {context_id}");
            return FilterHeadersStatus::Continue;
            // };
            // context
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context.parent_context_id);
        context.data.on_http_request_headers(&RequestHeaders {
            header_count,
            end_of_stream,
            attributes: Attributes::get(),
        })
    }

    fn on_http_request_body(
        &self,
        context_id: u32,
        body_size: usize,
        end_of_stream: bool,
    ) -> FilterDataStatus {
        let mut http_streams = self.http_streams.borrow_mut();
        let Some(context) = http_streams.get_mut(&context_id) else {
            warn!("no http context found for on_http_request_body: {context_id}");
            return FilterDataStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context.parent_context_id);
        context.data.on_http_request_body(&RequestBody {
            body_size,
            end_of_stream,
            attributes: Attributes::get(),
        })
    }

    fn on_http_request_trailers(
        &self,
        context_id: u32,
        trailer_count: usize,
    ) -> FilterTrailersStatus {
        let mut http_streams = self.http_streams.borrow_mut();
        let Some(context) = http_streams.get_mut(&context_id) else {
            warn!("no http context found for on_http_request_trailers: {context_id}");
            return FilterTrailersStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context.parent_context_id);
        context.data.on_http_request_trailers(&RequestTrailers {
            trailer_count,
            attributes: Attributes::get(),
        })
    }

    fn on_http_response_headers(
        &self,
        context_id: u32,
        header_count: usize,
        end_of_stream: bool,
    ) -> FilterHeadersStatus {
        let mut http_streams = self.http_streams.borrow_mut();
        let Some(context) = http_streams.get_mut(&context_id) else {
            warn!("no http context found for on_http_response_headers: {context_id}");
            return FilterHeadersStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context.parent_context_id);
        context.data.on_http_response_headers(&ResponseHeaders {
            header_count,
            end_of_stream,
            attributes: Attributes::get(),
        })
    }

    fn on_http_response_body(
        &self,
        context_id: u32,
        body_size: usize,
        end_of_stream: bool,
    ) -> FilterDataStatus {
        let mut http_streams = self.http_streams.borrow_mut();
        let Some(context) = http_streams.get_mut(&context_id) else {
            warn!("no http context found for on_http_response_body: {context_id}");
            return FilterDataStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context.parent_context_id);
        context.data.on_http_response_body(&ResponseBody {
            body_size,
            end_of_stream,
            attributes: Attributes::get(),
        })
    }

    fn on_http_response_trailers(
        &self,
        context_id: u32,
        trailer_count: usize,
    ) -> FilterTrailersStatus {
        let mut http_streams = self.http_streams.borrow_mut();
        let Some(context) = http_streams.get_mut(&context_id) else {
            warn!("no http context found for on_http_response_trailers: {context_id}");
            return FilterTrailersStatus::Continue;
        };
        self.active_id.set(context_id);
        self.active_root_id.set(context.parent_context_id);
        context.data.on_http_response_trailers(&ResponseTrailers {
            trailer_count,
            attributes: Attributes::get(),
        })
    }

    fn on_http_call_response(
        &self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
    ) {
        let Some(callback) = self.http_callbacks.borrow_mut().remove(&token_id) else {
            debug!(
                "received http_call_response for token {token_id}, but no callback was registered"
            );
            return;
        };
        let mut roots = self.roots.borrow_mut();
        let Some(root) = roots.get_mut(&callback.root_context_id) else {
            debug!("referenced non-existing root context");
            return;
        };
        let Some(_ctx) = EffectiveContext::enter(
            callback.context_id,
            callback.root_context_id,
            "http callback",
        ) else {
            return;
        };
        (callback.callback)(
            &mut root.data,
            &HttpCallResponse::new(num_headers, body_size, num_trailers),
        );
    }

    #[cfg(feature = "stream-metadata")]
    fn on_grpc_receive_initial_metadata(&self, token_id: u32, num_headers: u32) {
        let mut grpc_streams = self.grpc_streams;
        let Some(callback) = grpc_streams.get_mut(&token_id) else {
            debug!("received grpc message for unknown token {token_id}");
            return;
        };
        let Some(function) = &mut callback.initial_meta else {
            return;
        };
        let mut roots = self.roots.borrow_mut();
        let Some(root) = roots.get_mut(&callback.root_context_id) else {
            debug!("referenced non-existing root context");
            return;
        };

        let Some(_ctx) =
            EffectiveContext::enter(callback.context_id, callback.root_context_id, "grpc stream")
        else {
            return;
        };

        function(
            &mut root.data,
            GrpcStreamHandle(token_id),
            &GrpcStreamInitialMetadata::new(num_headers as usize),
        );
    }

    fn on_grpc_receive(&self, token_id: u32, response_size: usize) {
        if let Some(callback) = self.grpc_callbacks.borrow_mut().remove(&token_id) {
            let mut roots = self.roots.borrow_mut();
            let Some(root) = roots.get_mut(&callback.root_context_id) else {
                debug!("referenced non-existing root context");
                return;
            };
            let Some(_ctx) = EffectiveContext::enter(
                callback.context_id,
                callback.root_context_id,
                "grpc callback",
            ) else {
                return;
            };

            (callback.callback)(
                &mut root.data,
                &GrpcCallResponse::new(token_id, GrpcCode::Ok, None, response_size),
            );
        } else if let Some(callback) = self.grpc_streams.borrow_mut().get_mut(&token_id) {
            let Some(function) = &mut callback.message else {
                return;
            };
            let mut roots = self.roots.borrow_mut();
            let Some(root) = roots.get_mut(&callback.root_context_id) else {
                debug!("referenced non-existing root context");
                return;
            };

            let Some(_ctx) = EffectiveContext::enter(
                callback.context_id,
                callback.root_context_id,
                "grpc stream",
            ) else {
                return;
            };

            function(
                &mut root.data,
                GrpcStreamHandle(token_id),
                &GrpcStreamMessage::new(GrpcCode::Ok, None, response_size),
            );
        } else {
            debug!("received grpc message for unknown token {token_id}");
        }
    }

    #[cfg(feature = "stream-metadata")]
    fn on_grpc_receive_trailing_metadata(&self, token_id: u32, num_headers: u32) {
        let mut grpc_streams = self.grpc_streams.borrow_mut();
        let Some(callback) = grpc_streams.get_mut(&token_id) else {
            debug!("received grpc message for unknown token {token_id}");
            return;
        };
        let Some(function) = &mut callback.trailer_meta else {
            return;
        };
        let mut roots = self.roots.borrow_mut();
        let Some(root) = roots.get_mut(&callback.root_context_id) else {
            debug!("referenced non-existing root context");
            return;
        };
        let Some(_ctx) =
            EffectiveContext::enter(callback.context_id, callback.root_context_id, "grpc stream")
        else {
            return;
        };

        function(
            &mut root.data,
            GrpcStreamHandle(token_id),
            &GrpcStreamTrailingMetadata::new(num_headers as usize),
        );
    }

    fn on_grpc_close(&self, token_id: u32, status_code: u32) {
        if let Some(callback) = self.grpc_callbacks.borrow_mut().remove(&token_id) {
            let mut roots = self.roots.borrow_mut();
            let Some(root) = roots.get_mut(&callback.root_context_id) else {
                debug!("referenced non-existing root context");
                return;
            };
            let Some(_ctx) = EffectiveContext::enter(
                callback.context_id,
                callback.root_context_id,
                "grpc callback",
            ) else {
                return;
            };
            let Some((status, message)) =
                check_concern("grpc-call-close-status", hostcalls::get_grpc_status())
            else {
                return;
            };
            if status != status_code {
                warn!("status code mismatch for on_grpc_close");
            }

            (callback.callback)(
                &mut root.data,
                &GrpcCallResponse::new(token_id, status.into(), message, 0),
            );
        } else if let Some(callback) = self.grpc_streams.borrow_mut().remove(&token_id) {
            let Some(function) = callback.close else {
                return;
            };
            let mut roots = self.roots.borrow_mut();
            let Some(root) = roots.get_mut(&callback.root_context_id) else {
                debug!("referenced non-existing root context");
                return;
            };
            let Some(_ctx) = EffectiveContext::enter(
                callback.context_id,
                callback.root_context_id,
                "grpc stream",
            ) else {
                return;
            };
            let Some((status, message)) =
                check_concern("grpc-stream-close-status", hostcalls::get_grpc_status())
            else {
                return;
            };

            function(
                &mut root.data,
                &GrpcStreamClose::new(token_id, status.into(), message),
            );
        } else {
            debug!("received grpc close for unknown token {token_id}");
        }
    }
}

#[no_mangle]
pub extern "C" fn proxy_on_context_create(context_id: usize, root_context_id: usize) {
    dispatch(|d| d.on_create_context(context_id as u32, root_context_id as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_done(context_id: usize) -> usize {
    dispatch(|d| d.on_done(context_id as u32)) as usize
}

#[no_mangle]
pub extern "C" fn proxy_on_log(context_id: usize) {
    dispatch(|d| d.on_log(context_id as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_delete(context_id: usize) {
    dispatch(|d| d.on_delete(context_id as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_vm_start(context_id: usize, vm_configuration_size: usize) -> usize {
    dispatch(|d| d.on_vm_start(context_id as u32, vm_configuration_size)) as usize
}

#[no_mangle]
pub extern "C" fn proxy_on_configure(context_id: usize, plugin_configuration_size: usize) -> usize {
    dispatch(|d| d.on_configure(context_id as u32, plugin_configuration_size)) as usize
}

#[no_mangle]
pub extern "C" fn proxy_on_tick(context_id: usize) {
    dispatch(|d| d.on_tick(context_id as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_queue_ready(context_id: usize, queue_id: usize) {
    dispatch(|d| d.on_queue_ready(context_id as u32, queue_id as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_new_connection(context_id: usize) -> FilterStreamStatus {
    dispatch(|d| d.on_new_connection(context_id as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_downstream_data(
    context_id: usize,
    data_size: usize,
    end_of_stream: usize,
) -> FilterStreamStatus {
    dispatch(|d| d.on_downstream_data(context_id as u32, data_size, end_of_stream != 0))
}

#[no_mangle]
pub extern "C" fn proxy_on_downstream_connection_close(context_id: usize, close_type: CloseType) {
    dispatch(|d| d.on_downstream_close(context_id as u32, close_type))
}

#[no_mangle]
pub extern "C" fn proxy_on_upstream_data(
    context_id: usize,
    data_size: usize,
    end_of_stream: usize,
) -> FilterStreamStatus {
    dispatch(|d| d.on_upstream_data(context_id as u32, data_size, end_of_stream != 0))
}

#[no_mangle]
pub extern "C" fn proxy_on_upstream_connection_close(context_id: usize, close_type: CloseType) {
    dispatch(|d| d.on_upstream_close(context_id as u32, close_type))
}

#[no_mangle]
pub extern "C" fn proxy_on_request_headers(
    context_id: usize,
    num_headers: usize,
    end_of_stream: usize,
) -> FilterHeadersStatus {
    dispatch(|d| d.on_http_request_headers(context_id as u32, num_headers, end_of_stream != 0))
}

#[no_mangle]
pub extern "C" fn proxy_on_request_body(
    context_id: usize,
    body_size: usize,
    end_of_stream: usize,
) -> FilterDataStatus {
    dispatch(|d| d.on_http_request_body(context_id as u32, body_size, end_of_stream != 0))
}

#[no_mangle]
pub extern "C" fn proxy_on_request_trailers(
    context_id: usize,
    num_trailers: usize,
) -> FilterTrailersStatus {
    dispatch(|d| d.on_http_request_trailers(context_id as u32, num_trailers))
}

#[no_mangle]
pub extern "C" fn proxy_on_response_headers(
    context_id: usize,
    num_headers: usize,
    end_of_stream: usize,
) -> FilterHeadersStatus {
    dispatch(|d| d.on_http_response_headers(context_id as u32, num_headers, end_of_stream != 0))
}

#[no_mangle]
pub extern "C" fn proxy_on_response_body(
    context_id: usize,
    body_size: usize,
    end_of_stream: usize,
) -> FilterDataStatus {
    dispatch(|d| d.on_http_response_body(context_id as u32, body_size, end_of_stream != 0))
}

#[no_mangle]
pub extern "C" fn proxy_on_response_trailers(
    context_id: usize,
    num_trailers: usize,
) -> FilterTrailersStatus {
    dispatch(|d| d.on_http_response_trailers(context_id as u32, num_trailers))
}

#[no_mangle]
pub extern "C" fn proxy_on_http_call_response(
    _context_id: usize,
    token_id: usize,
    num_headers: usize,
    body_size: usize,
    num_trailers: usize,
) {
    dispatch(|d| d.on_http_call_response(token_id as u32, num_headers, body_size, num_trailers))
}

#[cfg(feature = "stream-metadata")]
#[no_mangle]
pub extern "C" fn proxy_on_grpc_receive_initial_metadata(
    _context_id: usize,
    token_id: usize,
    headers: usize,
) {
    DISPATCHER
        .with_borrow_mut(|d| d.on_grpc_receive_initial_metadata(token_id as u32, headers as u32))
}

#[no_mangle]
pub extern "C" fn proxy_on_grpc_receive(_context_id: usize, token_id: usize, response_size: usize) {
    dispatch(|d| d.on_grpc_receive(token_id as u32, response_size))
}

#[cfg(feature = "stream-metadata")]
#[no_mangle]
pub extern "C" fn proxy_on_grpc_receive_trailing_metadata(
    _context_id: usize,
    token_id: usize,
    trailers: usize,
) {
    dispatch(|d| d.on_grpc_receive_trailing_metadata(token_id as usize, trailers as usize))
}

#[no_mangle]
pub extern "C" fn proxy_on_grpc_close(_context_id: usize, token_id: usize, status_code: usize) {
    dispatch(|d| d.on_grpc_close(token_id as u32, status_code as u32))
}
