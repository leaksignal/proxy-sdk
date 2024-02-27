use std::{
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

/// Outbound HTTP call
#[derive(Builder)]
#[builder(setter(into))]
#[builder(pattern = "owned")]
#[allow(clippy::type_complexity)]
pub struct HttpCall<'a> {
    /// Upstream cluster to send the request to.
    pub upstream: Upstream<'a>,
    /// All headers to be sent along with the request. The proxy may add additional headers.
    /// This should include pseudo headers like `:method` and `:path`.
    #[builder(setter(into, each(name = "header")), default)]
    pub headers: Vec<(&'a str, &'a [u8])>,
    /// All trailers to be sent along with the request.
    #[builder(setter(into, each(name = "trailer")), default)]
    pub trailers: Vec<(&'a str, &'a [u8])>,
    /// An optional request body to send with the request.
    #[builder(setter(strip_option, into), default)]
    pub body: Option<&'a [u8]>,
    /// A timeout on waiting for a response. Default is 10 seconds.
    #[builder(setter(strip_option, into), default)]
    pub timeout: Option<Duration>,
    /// Callback to call when a response has arrived.
    #[builder(setter(custom), default)]
    pub callback: Option<Box<dyn FnOnce(&mut DowncastBox<dyn RootContext>, &HttpCallResponse)>>,
}

impl<'a> HttpCallBuilder<'a> {
    /// Set a response callback
    pub fn callback<R: RootContext + 'static>(
        mut self,
        callback: impl FnOnce(&mut R, &HttpCallResponse) + 'static,
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

impl<'a> HttpCall<'a> {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Sends this `HttpCall` over the network.
    pub fn dispatch(self) -> Result<(), Status> {
        let token = hostcalls::dispatch_http_call(
            &self.upstream.0,
            &self.headers,
            self.body,
            &self.trailers,
            self.timeout.unwrap_or(Self::DEFAULT_TIMEOUT),
        )?;
        if let Some(callback) = self.callback {
            crate::dispatcher::register_http_callback(token, callback);
        }
        Ok(())
    }
}

/// Response type for [`HttpCall::callback`]
pub struct HttpCallResponse {
    num_headers: usize,
    body_size: usize,
    num_trailers: usize,
}

impl HttpCallResponse {
    pub(crate) fn new(num_headers: usize, body_size: usize, num_trailers: usize) -> Self {
        Self {
            num_headers,
            body_size,
            num_trailers,
        }
    }

    /// Number of headers contained
    pub fn num_headers(&self) -> usize {
        self.num_headers
    }

    /// Number of trailers contained
    pub fn num_trailers(&self) -> usize {
        self.num_trailers
    }

    /// Total size of the response body
    pub fn body_size(&self) -> usize {
        self.body_size
    }

    /// Get all response headers
    pub fn headers(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            "http-call-headers",
            hostcalls::get_map(MapType::HttpCallResponseHeaders),
        )
        .unwrap_or_default()
    }

    /// Get a specific response header
    pub fn header(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            "http-call-header",
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
            "http-call-body",
            hostcalls::get_buffer(BufferType::HttpCallResponseBody, start, size),
        )
    }

    /// Get the entire response body
    pub fn full_body(&self) -> Option<Vec<u8>> {
        self.body(..)
    }

    /// Get all response trailers
    pub fn trailers(&self) -> Vec<(String, Vec<u8>)> {
        log_concern(
            "http-call-trailers",
            hostcalls::get_map(MapType::HttpCallResponseTrailers),
        )
        .unwrap_or_default()
    }

    /// Get a specific response trailer
    pub fn trailer(&self, name: impl AsRef<str>) -> Option<Vec<u8>> {
        log_concern(
            "http-call-trailer",
            hostcalls::get_map_value(MapType::HttpCallResponseTrailers, name.as_ref()),
        )
    }
}
