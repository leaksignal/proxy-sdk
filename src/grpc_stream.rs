use std::{
    fmt,
    ops::{Bound, RangeBounds},
};

use derive_builder::Builder;

use crate::{
    downcast_box::DowncastBox,
    grpc_call::GrpcCode,
    hostcalls::{self, BufferType},
    log_concern, RootContext, Status, Upstream,
};

#[cfg(feature = "stream-metadata")]
use crate::hostcalls::MapType;

/// Outbound GRPC stream (bidirectional)
#[derive(Builder)]
#[builder(setter(into))]
#[builder(pattern = "owned")]
#[allow(clippy::type_complexity)]
pub struct GrpcStream<'a> {
    /// Upstream cluster to send the request to.
    pub cluster: Upstream<'a>,
    /// The GRPC service to call.
    pub service: &'a str,
    /// The GRPC service method to call.
    pub method: &'a str,
    /// Initial GRPC metadata to send with the request.
    #[builder(setter(each(name = "metadata")), default)]
    pub initial_metadata: Vec<(&'a str, &'a [u8])>,
    /// Callback to call when the server sends initial metadata.
    #[cfg(feature = "stream-metadata")]
    #[builder(setter(custom), default)]
    pub on_initial_metadata: Option<
        Box<
            dyn FnMut(
                &mut DowncastBox<dyn RootContext>,
                GrpcStreamHandle,
                &GrpcStreamInitialMetadata,
            ),
        >,
    >,
    /// Callback to call when the server sends a stream message.
    #[builder(setter(custom), default)]
    pub on_message: Option<
        Box<dyn FnMut(&mut DowncastBox<dyn RootContext>, GrpcStreamHandle, &GrpcStreamMessage)>,
    >,
    /// Callback to call when the server sends trailing metadata.
    #[cfg(feature = "stream-metadata")]
    #[builder(setter(custom), default)]
    pub on_trailing_metadata: Option<
        Box<
            dyn FnMut(
                &mut DowncastBox<dyn RootContext>,
                GrpcStreamHandle,
                &GrpcStreamTrailingMetadata,
            ),
        >,
    >,
    /// Callback to call when the stream closes.
    #[builder(setter(custom), default)]
    pub on_close: Option<Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &GrpcStreamClose)>>,
}

impl<'a> GrpcStreamBuilder<'a> {
    /// Set an initial metadata callback
    #[cfg(feature = "stream-metadata")]
    pub fn on_initial_metadata<R: RootContext + 'static>(
        mut self,
        mut callback: impl FnMut(&mut R, GrpcStreamHandle, &GrpcStreamInitialMetadata) + 'static,
    ) -> Self {
        self.on_initial_metadata = Some(Some(Box::new(move |root, handle, metadata| {
            callback(
                root.as_any_mut().downcast_mut().expect("invalid root type"),
                handle,
                metadata,
            )
        })));
        self
    }

    /// Set a stream message callback
    pub fn on_message<R: RootContext + 'static>(
        mut self,
        mut callback: impl FnMut(&mut R, GrpcStreamHandle, &GrpcStreamMessage) + 'static,
    ) -> Self {
        self.on_message = Some(Some(Box::new(move |root, handle, message| {
            callback(
                root.as_any_mut().downcast_mut().expect("invalid root type"),
                handle,
                message,
            )
        })));
        self
    }

    /// Set a trailing metadata callback
    #[cfg(feature = "stream-metadata")]
    pub fn on_trailing_metadata<R: RootContext + 'static>(
        mut self,
        mut callback: impl FnMut(&mut R, GrpcStreamHandle, &GrpcStreamTrailingMetadata) + 'static,
    ) -> Self {
        self.on_trailing_metadata = Some(Some(Box::new(move |root, handle, metadata| {
            callback(
                root.as_any_mut().downcast_mut().expect("invalid root type"),
                handle,
                metadata,
            )
        })));
        self
    }

    /// Set a stream close callback
    pub fn on_close<R: RootContext + 'static>(
        mut self,
        callback: impl FnOnce(&mut R, &GrpcStreamClose) + 'static,
    ) -> Self {
        self.on_close = Some(Some(Box::new(move |root, close| {
            callback(
                root.as_any_mut().downcast_mut().expect("invalid root type"),
                close,
            )
        })));
        self
    }
}

/// GRPC stream handle to cancel, close, or send a message over a GRPC stream.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GrpcStreamHandle(pub(crate) u32);

impl<'a> GrpcStream<'a> {
    /// Open a new outbound GRPC stream.
    pub fn open(self) -> Result<GrpcStreamHandle, Status> {
        let token = hostcalls::open_grpc_stream(
            &self.cluster.0,
            self.service,
            self.method,
            &self.initial_metadata,
        )?;

        #[cfg(feature = "stream-metadata")]
        if let Some(callback) = self.on_initial_metadata {
            crate::dispatcher::register_grpc_stream_initial_meta(token, callback);
        }
        if let Some(callback) = self.on_message {
            crate::dispatcher::register_grpc_stream_message(token, callback);
        }
        #[cfg(feature = "stream-metadata")]
        if let Some(callback) = self.on_trailing_metadata {
            crate::dispatcher::register_grpc_stream_trailing_metadata(token, callback);
        }
        if let Some(callback) = self.on_close {
            crate::dispatcher::register_grpc_stream_close(token, callback);
        }

        Ok(GrpcStreamHandle(token))
    }
}

impl GrpcStreamHandle {
    /// Attempts to cancel the GRPC stream
    pub fn cancel(&self) {
        hostcalls::cancel_grpc_stream(self.0).ok();
    }

    /// Closes the GRPC stream
    pub fn close(&self) {
        hostcalls::close_grpc_stream(self.0).ok();
    }

    /// Sends a message over the GRPC stream
    pub fn send(&self, message: Option<impl AsRef<[u8]>>, end_stream: bool) -> Result<(), Status> {
        hostcalls::send_grpc_stream_message(
            self.0,
            message.as_ref().map(|x| x.as_ref()),
            end_stream,
        )
    }
}

impl PartialEq<u32> for GrpcStreamHandle {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<GrpcStreamHandle> for u32 {
    fn eq(&self, other: &GrpcStreamHandle) -> bool {
        other == self
    }
}

impl fmt::Display for GrpcStreamHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Response type for [`GrpcStream::on_initial_metadata`]
#[cfg(feature = "stream-metadata")]
pub struct GrpcStreamInitialMetadata {
    num_elements: usize,
}

#[cfg(feature = "stream-metadata")]
impl GrpcStreamInitialMetadata {
    pub(crate) fn new(num_elements: usize) -> Self {
        Self { num_elements }
    }

    /// Number of metadata elements
    pub fn num_elements(&self) -> usize {
        self.num_elements
    }

    /// Get all metadata elements
    pub fn all(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            "grpc-stream-metadata-all",
            hostcalls::get_map(MapType::GrpcReceiveInitialMetadata),
        )
        .unwrap_or_default()
    }

    /// Get a specific metadata element
    pub fn value(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            "grpc-stream-metadata",
            hostcalls::get_map_value(MapType::GrpcReceiveInitialMetadata, name.as_ref()),
        )
    }
}

/// Response type for [`GrpcStream::on_message`]
pub struct GrpcStreamMessage {
    status_code: GrpcCode,
    body_size: usize,
    message: Option<String>,
}

impl GrpcStreamMessage {
    pub(crate) fn new(status_code: GrpcCode, message: Option<String>, body_size: usize) -> Self {
        Self {
            status_code,
            body_size,
            message,
        }
    }

    /// GRPC status code of the message
    pub fn status_code(&self) -> GrpcCode {
        self.status_code
    }

    /// Optional GRPC status message of the message
    pub fn status_message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Total size of the message body
    pub fn body_size(&self) -> usize {
        self.body_size
    }

    /// Get a range of the message body
    pub fn body(&self, range: impl RangeBounds<usize>) -> Option<Vec<u8>> {
        let start = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x.saturating_sub(1),
            Bound::Unbounded => 0,
        };
        let size = match range.end_bound() {
            Bound::Included(x) => *x + 1,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => self.body_size,
        }
        .min(self.body_size)
        .saturating_sub(start);
        log_concern(
            "grpc-stream-message-body",
            hostcalls::get_buffer(BufferType::GrpcReceiveBuffer, start, size),
        )
    }

    /// Get the entire message body
    pub fn full_body(&self) -> Option<Vec<u8>> {
        self.body(..self.body_size)
    }
}

/// Response type for [`GrpcStream::on_trailing_metadata`]
#[cfg(feature = "stream-metadata")]
pub struct GrpcStreamTrailingMetadata {
    num_elements: usize,
}

#[cfg(feature = "stream-metadata")]
impl GrpcStreamTrailingMetadata {
    pub(crate) fn new(num_elements: usize) -> Self {
        Self { num_elements }
    }

    /// Number of metadata elements
    pub fn num_elements(&self) -> usize {
        self.num_elements
    }

    /// Get all metadata elements
    pub fn all(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            "grpc-stream-trailing-metadata-all",
            hostcalls::get_map(MapType::GrpcReceiveTrailingMetadata),
        )
        .unwrap_or_default()
    }

    /// Get a specific metadata element
    pub fn value(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            "grpc-stream-trailing-metadata",
            hostcalls::get_map_value(MapType::GrpcReceiveTrailingMetadata, name.as_ref()),
        )
    }
}

/// Response type for [`GrpcStream::on_close`]
pub struct GrpcStreamClose {
    handle_id: u32,
    status_code: GrpcCode,
    message: Option<String>,
}

impl GrpcStreamClose {
    pub(crate) fn new(token_id: u32, status_code: GrpcCode, message: Option<String>) -> Self {
        Self {
            handle_id: token_id,
            status_code,
            message,
        }
    }

    /// GRPC handle ID of the message
    pub fn handle_id(&self) -> u32 {
        self.handle_id
    }

    /// GRPC status code of the message
    pub fn status_code(&self) -> GrpcCode {
        self.status_code
    }

    /// Optional GRPC status message of the message
    pub fn status_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}
