use std::ops::RangeBounds;

use crate::{
    calculate_range,
    context::BaseContext,
    hostcalls::{self, BufferType, MapType},
    log_concern,
    property::envoy::Attributes,
    Status,
};

/// Defines control functions for http data
pub trait HttpControl {
    /// Request or Response
    const TYPE: HttpType;

    /// Retrieve attributes for the http data
    fn attributes(&self) -> &Attributes;

    /// If `true`, this is the last block
    fn end_of_stream(&self) -> bool {
        true
    }

    /// Resume a paused HTTP request/response
    fn resume(&self) {
        log_concern(Self::TYPE.resume(), Self::TYPE.call_resume())
    }

    /// Reset the HTTP request/response
    fn reset(&self) {
        log_concern(Self::TYPE.reset(), Self::TYPE.call_reset())
    }

    /// Send an early HTTP response, terminating the current request/response
    fn send_http_response(
        &self,
        status_code: u32,
        headers: &[(&str, &[u8])],
        body: Option<&[u8]>,
    ) -> Result<(), Status> {
        hostcalls::send_http_response(status_code, headers, body)
    }

    /// Mark this transaction as complete
    fn done(&self) {
        log_concern("trigger-done", hostcalls::done());
    }
}

/// Defines functions to interact with header data
pub trait HttpHeaderControl: HttpControl {
    /// The header type
    const HEADER_TYPE: HeaderType;

    /// Number of headers contained in block
    fn header_count(&self) -> usize;

    /// Get all headers in this block
    fn all(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            Self::HEADER_TYPE.all(),
            hostcalls::get_map(Self::HEADER_TYPE.map()),
        )
        .unwrap_or_default()
    }

    /// Check for a specific header value
    fn get(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            Self::HEADER_TYPE.get(),
            hostcalls::get_map_value(Self::HEADER_TYPE.map(), name.as_ref()),
        )
    }

    /// Set a specific header
    fn set(&self, name: impl AsRef<str>, value: impl AsRef<[u8]>) {
        log_concern(
            Self::HEADER_TYPE.set(),
            hostcalls::set_map_value(Self::HEADER_TYPE.map(), name.as_ref(), Some(value.as_ref())),
        );
    }

    /// Replace all headers in this block
    fn set_all(&self, values: &[(&str, &[u8])]) {
        log_concern(
            Self::HEADER_TYPE.set_all(),
            hostcalls::set_map(Self::HEADER_TYPE.map(), values),
        );
    }

    /// Add a header to this block (append to existing if present)
    fn add(&self, name: impl AsRef<str>, value: impl AsRef<[u8]>) {
        log_concern(
            Self::HEADER_TYPE.add(),
            hostcalls::add_map_value(Self::HEADER_TYPE.map(), name.as_ref(), value.as_ref()),
        );
    }

    /// Remove a header from this block
    fn remove(&self, name: impl AsRef<str>) {
        log_concern(
            Self::HEADER_TYPE.remove(),
            hostcalls::set_map_value(Self::HEADER_TYPE.map(), name.as_ref(), None),
        );
    }
}

/// Defines functions to interact with body data
pub trait HttpBodyControl: HttpControl {
    /// Length of this body fragment
    fn body_size(&self) -> usize;

    /// Get a range of the body block content
    fn get(&self, range: impl RangeBounds<usize>) -> Option<Vec<u8>> {
        let (start, size) = calculate_range(range, self.body_size());
        log_concern(
            Self::TYPE.get(),
            hostcalls::get_buffer(Self::TYPE.buffer(), start, size),
        )
    }

    /// Set a range of the body block content
    fn set(&self, range: impl RangeBounds<usize>, value: &[u8]) {
        let (start, size) = calculate_range(range, self.body_size());
        log_concern(
            Self::TYPE.set(),
            hostcalls::set_buffer(Self::TYPE.buffer(), start, size, value),
        );
    }

    /// Get the entire body block content
    fn all(&self) -> Option<Vec<u8>> {
        self.get(..)
    }

    /// Replace the entire body block with `value`
    fn replace(&self, value: &[u8]) {
        self.set(.., value);
    }

    /// Clear the entire body block
    fn clear(&self) {
        self.replace(&[]);
    }
}

/// Defines which section the header data belongs too
pub enum HeaderType {
    RequestHeaders,
    RequestTrailers,
    ResponseHeaders,
    ResponseTrailers,
}

impl HeaderType {
    const fn all(&self) -> &'static str {
        match self {
            Self::RequestHeaders => "get-all-request-header",
            Self::RequestTrailers => "get-all-request-trailer",
            Self::ResponseHeaders => "get-all-response-header",
            Self::ResponseTrailers => "get-all-response-trailer",
        }
    }

    const fn get(&self) -> &'static str {
        match self {
            Self::RequestHeaders => "get-request-header",
            Self::RequestTrailers => "get-request-trailer",
            Self::ResponseHeaders => "get-response-header",
            Self::ResponseTrailers => "get-response-trailer",
        }
    }

    const fn set(&self) -> &'static str {
        match self {
            Self::RequestHeaders => "set-request-header",
            Self::RequestTrailers => "set-request-trailer",
            Self::ResponseHeaders => "set-response-header",
            Self::ResponseTrailers => "set-response-trailer",
        }
    }

    const fn set_all(&self) -> &'static str {
        match self {
            Self::RequestHeaders => "set-all-request-headers",
            Self::RequestTrailers => "set-all-request-trailers",
            Self::ResponseHeaders => "set-all-response-headers",
            Self::ResponseTrailers => "set-all-response-trailers",
        }
    }

    const fn add(&self) -> &'static str {
        match self {
            Self::RequestHeaders => "add-request-headers",
            Self::RequestTrailers => "add-request-trailers",
            Self::ResponseHeaders => "add-response-headers",
            Self::ResponseTrailers => "add-response-trailers",
        }
    }

    const fn remove(&self) -> &'static str {
        match self {
            Self::RequestHeaders => "remove-request-headers",
            Self::RequestTrailers => "remove-request-trailers",
            Self::ResponseHeaders => "remove-response-headers",
            Self::ResponseTrailers => "remove-response-trailers",
        }
    }

    const fn map(&self) -> MapType {
        match self {
            HeaderType::RequestHeaders => MapType::HttpRequestHeaders,
            HeaderType::RequestTrailers => MapType::HttpRequestTrailers,
            HeaderType::ResponseHeaders => MapType::HttpResponseHeaders,
            HeaderType::ResponseTrailers => MapType::HttpResponseTrailers,
        }
    }
}

/// Defines if data belongs to a request or response
pub enum HttpType {
    Request,
    Response,
}

impl HttpType {
    const fn resume(&self) -> &'static str {
        match self {
            HttpType::Request => "resume-http-request",
            HttpType::Response => "resume-http-response",
        }
    }

    fn call_resume(&self) -> Result<(), Status> {
        match self {
            HttpType::Request => hostcalls::resume_http_request(),
            HttpType::Response => hostcalls::resume_http_response(),
        }
    }

    const fn reset(&self) -> &'static str {
        match self {
            HttpType::Request => "reset-http-request",
            HttpType::Response => "reset-http-response",
        }
    }

    fn call_reset(&self) -> Result<(), Status> {
        match self {
            HttpType::Request => hostcalls::reset_http_request(),
            HttpType::Response => hostcalls::reset_http_response(),
        }
    }

    const fn get(&self) -> &'static str {
        match self {
            HttpType::Request => "get-request-body",
            HttpType::Response => "get-response-body",
        }
    }

    const fn set(&self) -> &'static str {
        match self {
            HttpType::Request => "set-request-body",
            HttpType::Response => "set-response-body",
        }
    }

    const fn buffer(&self) -> BufferType {
        match self {
            HttpType::Request => BufferType::HttpRequestBody,
            HttpType::Response => BufferType::HttpResponseBody,
        }
    }
}

/// Return status for header callbacks
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterHeadersStatus {
    Continue = 0,
    StopIteration = 1,
    ContinueAndEndStream = 2,
    StopAllIterationAndBuffer = 3,
    StopAllIterationAndWatermark = 4,
}

/// Return status for trailer callbacks
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterTrailersStatus {
    Continue = 0,
    StopIteration = 1,
}

/// Return status for body callbacks
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterDataStatus {
    Continue = 0,
    StopAllIterationAndBuffer = 1,
    StopAllIterationAndWatermark = 2,
    StopIterationNoBuffer = 3,
}

/// Request header context
pub struct RequestHeaders {
    pub(crate) header_count: usize,
    pub(crate) end_of_stream: bool,
    pub(crate) attributes: Attributes,
}

impl HttpControl for RequestHeaders {
    const TYPE: HttpType = HttpType::Request;

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn end_of_stream(&self) -> bool {
        self.end_of_stream
    }
}

impl HttpHeaderControl for RequestHeaders {
    const HEADER_TYPE: HeaderType = HeaderType::RequestHeaders;

    fn header_count(&self) -> usize {
        self.header_count
    }
}

pub struct RequestBody {
    pub(crate) body_size: usize,
    pub(crate) end_of_stream: bool,
    pub(crate) attributes: Attributes,
}

impl HttpControl for RequestBody {
    const TYPE: HttpType = HttpType::Request;

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn end_of_stream(&self) -> bool {
        self.end_of_stream
    }
}

impl HttpBodyControl for RequestBody {
    fn body_size(&self) -> usize {
        self.body_size
    }
}

pub struct RequestTrailers {
    pub(crate) trailer_count: usize,
    pub(crate) attributes: Attributes,
}

impl HttpControl for RequestTrailers {
    const TYPE: HttpType = HttpType::Request;

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl HttpHeaderControl for RequestTrailers {
    const HEADER_TYPE: HeaderType = HeaderType::RequestTrailers;

    fn header_count(&self) -> usize {
        self.trailer_count
    }
}

pub struct ResponseHeaders {
    pub(crate) header_count: usize,
    pub(crate) end_of_stream: bool,
    pub(crate) attributes: Attributes,
}

impl HttpControl for ResponseHeaders {
    const TYPE: HttpType = HttpType::Response;

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn end_of_stream(&self) -> bool {
        self.end_of_stream
    }
}

impl HttpHeaderControl for ResponseHeaders {
    const HEADER_TYPE: HeaderType = HeaderType::ResponseHeaders;

    fn header_count(&self) -> usize {
        self.header_count
    }
}

pub struct ResponseBody {
    pub(crate) body_size: usize,
    pub(crate) end_of_stream: bool,
    pub(crate) attributes: Attributes,
}

impl HttpControl for ResponseBody {
    const TYPE: HttpType = HttpType::Response;

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn end_of_stream(&self) -> bool {
        self.end_of_stream
    }
}

impl HttpBodyControl for ResponseBody {
    fn body_size(&self) -> usize {
        self.body_size
    }
}

pub struct ResponseTrailers {
    pub(crate) trailer_count: usize,
    pub(crate) attributes: Attributes,
}

impl HttpControl for ResponseTrailers {
    const TYPE: HttpType = HttpType::Response;

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl HttpHeaderControl for ResponseTrailers {
    const HEADER_TYPE: HeaderType = HeaderType::ResponseTrailers;

    fn header_count(&self) -> usize {
        self.trailer_count
    }
}

/// Context for a HTTP filter plugin.
#[allow(unused_variables)]
pub trait HttpContext: BaseContext {
    /// Called one or more times as the proxy receives request headers. If `headers.end_of_stream()` is true, then they are the last request headers.
    fn on_http_request_headers(&mut self, headers: &RequestHeaders) -> FilterHeadersStatus {
        FilterHeadersStatus::Continue
    }

    /// Called only if and only if there is a request body. Called one or more times as the proxy receives blocks of request body data. If `body.end_of_stream()` is true, it is the last block.
    fn on_http_request_body(&mut self, body: &RequestBody) -> FilterDataStatus {
        FilterDataStatus::Continue
    }

    /// Called once if and only if any trailers are sent at the end of the request. Not called multiple times.
    fn on_http_request_trailers(&mut self, trailers: &RequestTrailers) -> FilterTrailersStatus {
        FilterTrailersStatus::Continue
    }

    /// Called one or more times as the proxy receives response headers. If `headers.end_of_stream()` is true, then they are the last response headers.
    fn on_http_response_headers(&mut self, headers: &ResponseHeaders) -> FilterHeadersStatus {
        FilterHeadersStatus::Continue
    }

    /// Called only if and only if there is a response body. Called one or more times as the proxy receives blocks of response body data. If `body.end_of_stream()` is true, it is the last block.
    fn on_http_response_body(&mut self, body: &ResponseBody) -> FilterDataStatus {
        FilterDataStatus::Continue
    }

    /// Called once if and only if any trailers are sent at the end of the response. Not called multiple times.
    fn on_http_response_trailers(&mut self, trailers: &ResponseTrailers) -> FilterTrailersStatus {
        FilterTrailersStatus::Continue
    }
}
