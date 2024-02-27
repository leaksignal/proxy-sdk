use std::{
    fmt,
    ops::{Bound, RangeBounds},
    time::Duration,
};

use derive_builder::Builder;

use crate::{
    downcast_box::DowncastBox,
    hostcalls::{self, BufferType, MapType},
    log_concern,
    upstream::Upstream,
    RootContext, Status,
};

/// Outbound GRPC call
#[derive(Builder)]
#[builder(setter(into))]
#[builder(pattern = "owned")]
#[allow(clippy::type_complexity)]
pub struct GrpcCall<'a> {
    /// Upstream cluster to send the request to.
    pub upstream: Upstream<'a>,
    /// The GRPC service to call.
    pub service: &'a str,
    /// The GRPC service method to call.
    pub method: &'a str,
    /// Initial GRPC metadata to send with the request.
    #[builder(setter(each(name = "metadata")), default)]
    pub initial_metadata: Vec<(&'a str, &'a [u8])>,
    /// An optional request body to send with the request.
    #[builder(setter(strip_option, into), default)]
    pub message: Option<&'a [u8]>,
    /// A timeout on waiting for a response. Default is 10 seconds.
    #[builder(setter(strip_option, into), default)]
    pub timeout: Option<Duration>,
    /// Callback to call when a response has arrived.
    #[builder(setter(custom), default)]
    pub callback: Option<Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &GrpcCallResponse)>>,
}

impl<'a> GrpcCallBuilder<'a> {
    /// Set a response callback
    pub fn callback<R: RootContext + 'static>(
        mut self,
        callback: impl FnOnce(&mut R, &GrpcCallResponse) + 'static,
    ) -> Self {
        self.callback = Some(Some(Box::new(move |root, resp| {
            callback(
                root.as_any_mut().downcast_mut().expect("invalid root type"),
                resp,
            )
        })));
        self
    }
}

impl<'a> GrpcCall<'a> {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Sends this `GrpcCall` over the network.
    pub fn dispatch(self) -> Result<GrpcCancelHandle, Status> {
        let token = hostcalls::dispatch_grpc_call(
            &self.upstream.0,
            self.service,
            self.method,
            &self.initial_metadata,
            self.message,
            self.timeout.unwrap_or(Self::DEFAULT_TIMEOUT),
        )?;
        if let Some(callback) = self.callback {
            crate::dispatcher::register_grpc_callback(token, callback);
        }
        Ok(GrpcCancelHandle(token))
    }
}

/// GRPC Call Handle to cancel a request
#[derive(Debug)]
pub struct GrpcCancelHandle(u32);

impl GrpcCancelHandle {
    /// Attempts to cancel the GRPC call
    pub fn cancel(&self) {
        hostcalls::cancel_grpc_call(self.0).ok();
    }
}

impl fmt::Display for GrpcCancelHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<u32> for GrpcCancelHandle {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<GrpcCancelHandle> for u32 {
    fn eq(&self, other: &GrpcCancelHandle) -> bool {
        other == self
    }
}

/// Copied from `tonic` crate, GRPC status codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GrpcCode {
    /// The operation completed successfully.
    Ok = 0,

    /// The operation was cancelled.
    Cancelled = 1,

    /// Unknown error.
    Unknown = 2,

    /// Client specified an invalid argument.
    InvalidArgument = 3,

    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,

    /// Some requested entity was not found.
    NotFound = 5,

    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,

    /// Some resource has been exhausted.
    ResourceExhausted = 8,

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 9,

    /// The operation was aborted.
    Aborted = 10,

    /// Operation was attempted past the valid range.
    OutOfRange = 11,

    /// Operation is not implemented or not supported.
    Unimplemented = 12,

    /// Internal error.
    Internal = 13,

    /// The service is currently unavailable.
    Unavailable = 14,

    /// Unrecoverable data loss or corruption.
    DataLoss = 15,

    /// The request does not have valid authentication credentials
    Unauthenticated = 16,

    /// Unknown code
    Other(u32),
}

impl From<u32> for GrpcCode {
    fn from(value: u32) -> GrpcCode {
        match value {
            0 => GrpcCode::Ok,
            1 => GrpcCode::Cancelled,
            2 => GrpcCode::Unknown,
            3 => GrpcCode::InvalidArgument,
            4 => GrpcCode::DeadlineExceeded,
            5 => GrpcCode::NotFound,
            6 => GrpcCode::AlreadyExists,
            7 => GrpcCode::PermissionDenied,
            8 => GrpcCode::ResourceExhausted,
            9 => GrpcCode::FailedPrecondition,
            10 => GrpcCode::Aborted,
            11 => GrpcCode::OutOfRange,
            12 => GrpcCode::Unimplemented,
            13 => GrpcCode::Internal,
            14 => GrpcCode::Unavailable,
            15 => GrpcCode::DataLoss,
            16 => GrpcCode::Unauthenticated,
            x => GrpcCode::Other(x),
        }
    }
}

impl PartialEq<u32> for GrpcCode {
    fn eq(&self, other: &u32) -> bool {
        *self == Self::from(*other)
    }
}

impl PartialEq<GrpcCode> for u32 {
    fn eq(&self, other: &GrpcCode) -> bool {
        other == self
    }
}

/// Response type for [`GrpcCall::callback`]
pub struct GrpcCallResponse {
    handle_id: u32,
    status_code: GrpcCode,
    body_size: usize,
    message: Option<String>,
}

impl GrpcCallResponse {
    pub(crate) fn new(
        token_id: u32,
        status_code: GrpcCode,
        message: Option<String>,
        body_size: usize,
    ) -> Self {
        Self {
            handle_id: token_id,
            status_code,
            body_size,
            message,
        }
    }

    /// GRPC handle ID of the response
    pub fn handle_id(&self) -> u32 {
        self.handle_id
    }

    /// GRPC status code of the response
    pub fn status_code(&self) -> GrpcCode {
        self.status_code
    }

    /// Optional GRPC status message of the response
    pub fn status_message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Total size of the response body
    pub fn body_size(&self) -> usize {
        self.body_size
    }

    /// Get all response headers
    pub fn headers(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            "grpc-call-headers",
            hostcalls::get_map(MapType::HttpCallResponseHeaders),
        )
        .unwrap_or_default()
    }

    /// Get a specific response header
    pub fn header(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            "grpc-call-header",
            hostcalls::get_map_value(MapType::HttpCallResponseHeaders, name.as_ref()),
        )
    }

    /// Get a range of the response body
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
            "grpc-call-body",
            hostcalls::get_buffer(BufferType::GrpcReceiveBuffer, start, size),
        )
    }

    /// Get the entire response body
    pub fn full_body(&self) -> Option<Vec<u8>> {
        self.body(..)
    }

    /// Get all response trailers
    pub fn trailers(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            "grpc-call-trailers",
            hostcalls::get_map(MapType::HttpCallResponseTrailers),
        )
        .unwrap_or_default()
    }

    /// Get a specific response trailer
    pub fn trailer(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            "grpc-call-trailer",
            hostcalls::get_map_value(MapType::HttpCallResponseTrailers, name.as_ref()),
        )
    }
}
