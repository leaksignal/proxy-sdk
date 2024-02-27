use std::ops::RangeBounds;

use crate::{
    calculate_range,
    context::BaseContext,
    hostcalls::{self, BufferType},
    log_concern,
    property::envoy::Attributes,
};

/// Defines control functions for streams
pub trait StreamControl {
    /// Retrieve attributes for the stream data
    fn attributes(&self) -> &Attributes;

    /// TODO: UNKNOWN PURPOSE
    fn resume_downstream(&self) {
        log_concern("resume-downstream", hostcalls::resume_downstream());
    }

    /// TODO: UNKNOWN PURPOSE
    fn close_downstream(&self) {
        log_concern("close-downstream", hostcalls::close_downstream());
    }

    /// TODO: UNKNOWN PURPOSE
    fn resume_upstream(&self) {
        log_concern("resume-upstream", hostcalls::resume_upstream());
    }

    /// TODO: UNKNOWN PURPOSE
    fn close_upstream(&self) {
        log_concern("close-upstream", hostcalls::close_upstream());
    }
}

/// Defines functions to interact with stream data
pub trait StreamDataControl {
    /// Upstream or Downstream
    const TYPE: StreamType;

    /// Length of this chunk of data
    fn data_size(&self) -> usize;

    /// If true, this will be the last downstream data for this context.
    fn end_of_stream(&self) -> bool;

    /// Get all data
    fn all(&self) -> Option<Vec<u8>> {
        self.get(..)
    }

    /// Get a range of data
    fn get(&self, range: impl RangeBounds<usize>) -> Option<Vec<u8>> {
        let (start, size) = calculate_range(range, self.data_size());
        log_concern(
            Self::TYPE.get(),
            hostcalls::get_buffer(Self::TYPE.buffer(), start, size),
        )
    }

    /// Replace a range of data with `value`.
    fn set(&self, range: impl RangeBounds<usize>, value: &[u8]) {
        let (start, size) = calculate_range(range, self.data_size());
        log_concern(
            Self::TYPE.set(),
            hostcalls::set_buffer(Self::TYPE.buffer(), start, size, value),
        );
    }

    /// Replace the entire data with `value`
    fn replace(&self, value: &[u8]) {
        self.set(.., value);
    }

    /// Clear the data
    fn clear(&self) {
        self.replace(&[]);
    }
}

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterStreamStatus {
    Continue = 0,
    StopIteration = 1,
}

#[derive(Debug)]
pub enum StreamType {
    Upstream,
    Downstream,
}

impl StreamType {
    const fn get(&self) -> &'static str {
        match self {
            Self::Upstream => "get-upstream-data",
            Self::Downstream => "get-downstream-data",
        }
    }

    const fn set(&self) -> &'static str {
        match self {
            Self::Upstream => "set-upstream-data",
            Self::Downstream => "set-downstream-data",
        }
    }

    const fn buffer(&self) -> BufferType {
        match self {
            Self::Upstream => BufferType::UpstreamData,
            Self::Downstream => BufferType::DownstreamData,
        }
    }
}

/// Upstream data reference for a Stream filter
pub struct UpstreamData {
    pub(crate) data_size: usize,
    pub(crate) end_of_stream: bool,
    pub(crate) attributes: Attributes,
}

impl StreamControl for UpstreamData {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl StreamDataControl for UpstreamData {
    const TYPE: StreamType = StreamType::Upstream;

    fn data_size(&self) -> usize {
        self.data_size
    }

    fn end_of_stream(&self) -> bool {
        self.end_of_stream
    }
}

/// Downstream data reference for a Stream filter
pub struct DownstreamData {
    pub(crate) data_size: usize,
    pub(crate) end_of_stream: bool,
    pub(crate) attributes: Attributes,
}

impl StreamControl for DownstreamData {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl StreamDataControl for DownstreamData {
    const TYPE: StreamType = StreamType::Downstream;

    fn data_size(&self) -> usize {
        self.data_size
    }

    fn end_of_stream(&self) -> bool {
        self.end_of_stream
    }
}

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum CloseType {
    Unknown = 0,
    /// Close initiated by the proxy
    Local = 1,
    /// Close initiated by the peer
    Remote = 2,
}

/// Context for a stream closing event
pub struct StreamClose {
    pub(crate) close_type: CloseType,
    pub(crate) attributes: Attributes,
}

impl StreamClose {
    /// Get close type of closed peer
    pub fn close_type(&self) -> CloseType {
        self.close_type
    }
}

impl StreamControl for StreamClose {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

/// Trait to implement stream filters (L4 filters).
#[allow(unused_variables)]
pub trait StreamContext: BaseContext {
    /// Called on a new connection.
    /// TODO: FilterStreamStatus effect unknown.
    fn on_new_connection(&mut self) -> FilterStreamStatus {
        FilterStreamStatus::Continue
    }

    /// Called when a chunk of downstream data is available.
    /// `FilterStreamStatus::Pause` will delay flushing of data until `FilterStreamStatus::Continue` is returned.
    /// TODO: `resume_downstream` might be able to trigger this from another context?
    fn on_downstream_data(&mut self, data: &DownstreamData) -> FilterStreamStatus {
        FilterStreamStatus::Continue
    }

    /// Called when a downstream connection closes.
    fn on_downstream_close(&mut self, data: &StreamClose) {}

    /// Called when a chunk of upstream data is available.
    /// `FilterStreamStatus::Pause` will delay flushing of data until `FilterStreamStatus::Continue` is returned.
    /// TODO: `resume_downstream` might be able to trigger this from another context?
    fn on_upstream_data(&mut self, data: &UpstreamData) -> FilterStreamStatus {
        FilterStreamStatus::Continue
    }

    /// Called when an upstream connection closes.
    fn on_upstream_close(&mut self, data: &StreamClose) {}
}
