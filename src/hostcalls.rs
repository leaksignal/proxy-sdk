#![allow(clippy::type_complexity)]

use std::ptr::{null, null_mut, NonNull};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::Status;

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Critical = 5,
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum StreamType {
    HttpRequest = 0,
    HttpResponse = 1,
    Downstream = 2,
    Upstream = 3,
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum BufferType {
    HttpRequestBody = 0,
    HttpResponseBody = 1,
    DownstreamData = 2,
    UpstreamData = 3,
    HttpCallResponseBody = 4,
    GrpcReceiveBuffer = 5,
    VmConfiguration = 6,
    PluginConfiguration = 7,
    #[allow(dead_code)]
    CallData = 8,
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum MapType {
    HttpRequestHeaders = 0,
    HttpRequestTrailers = 1,
    HttpResponseHeaders = 2,
    HttpResponseTrailers = 3,
    GrpcReceiveInitialMetadata = 4,
    GrpcReceiveTrailingMetadata = 5,
    HttpCallResponseHeaders = 6,
    HttpCallResponseTrailers = 7,
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum MetricType {
    Counter = 0,
    Gauge = 1,
    Histogram = 2,
}

extern "C" {
    pub fn proxy_log(level: LogLevel, message_data: *const u8, message_size: usize) -> Status;
    pub fn proxy_get_log_level(return_level: *mut LogLevel) -> Status;
    pub fn proxy_get_current_time_nanoseconds(return_time: *mut u64) -> Status;
    pub fn proxy_set_tick_period_milliseconds(period: u32) -> Status;
    pub fn proxy_get_buffer_bytes(
        buffer_type: BufferType,
        start: usize,
        max_size: usize,
        return_buffer_data: *mut *mut u8,
        return_buffer_size: *mut usize,
    ) -> Status;
    pub fn proxy_set_buffer_bytes(
        buffer_type: BufferType,
        start: usize,
        size: usize,
        buffer_data: *const u8,
        buffer_size: usize,
    ) -> Status;
    pub fn proxy_get_header_map_pairs(
        map_type: MapType,
        return_map_data: *mut *mut u8,
        return_map_size: *mut usize,
    ) -> Status;
    pub fn proxy_set_header_map_pairs(
        map_type: MapType,
        map_data: *const u8,
        map_size: usize,
    ) -> Status;
    pub fn proxy_get_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
    pub fn proxy_replace_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
    pub fn proxy_remove_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
    ) -> Status;
    pub fn proxy_add_header_map_value(
        map_type: MapType,
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
    pub fn proxy_get_property(
        path_data: *const u8,
        path_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
    pub fn proxy_set_property(
        path_data: *const u8,
        path_size: usize,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
    pub fn proxy_get_shared_data(
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
        return_cas: *mut u32,
    ) -> Status;
    pub fn proxy_set_shared_data(
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
        cas: u32,
    ) -> Status;
    pub fn proxy_register_shared_queue(
        name_data: *const u8,
        name_size: usize,
        return_id: *mut u32,
    ) -> Status;
    pub fn proxy_resolve_shared_queue(
        vm_id_data: *const u8,
        vm_id_size: usize,
        name_data: *const u8,
        name_size: usize,
        return_id: *mut u32,
    ) -> Status;
    pub fn proxy_dequeue_shared_queue(
        queue_id: u32,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> Status;
    pub fn proxy_enqueue_shared_queue(
        queue_id: u32,
        value_data: *const u8,
        value_size: usize,
    ) -> Status;
    pub fn proxy_continue_stream(stream_type: StreamType) -> Status;
    pub fn proxy_close_stream(stream_type: StreamType) -> Status;
    pub fn proxy_send_local_response(
        status_code: u32,
        status_code_details_data: *const u8,
        status_code_details_size: usize,
        body_data: *const u8,
        body_size: usize,
        headers_data: *const u8,
        headers_size: usize,
        grpc_status: i32,
    ) -> Status;
    pub fn proxy_http_call(
        upstream_data: *const u8,
        upstream_size: usize,
        headers_data: *const u8,
        headers_size: usize,
        body_data: *const u8,
        body_size: usize,
        trailers_data: *const u8,
        trailers_size: usize,
        timeout: u32,
        return_token: *mut u32,
    ) -> Status;
    pub fn proxy_grpc_call(
        upstream_data: *const u8,
        upstream_size: usize,
        service_name_data: *const u8,
        service_name_size: usize,
        method_name_data: *const u8,
        method_name_size: usize,
        initial_metadata_data: *const u8,
        initial_metadata_size: usize,
        message_data_data: *const u8,
        message_data_size: usize,
        timeout: u32,
        return_callout_id: *mut u32,
    ) -> Status;
    pub fn proxy_grpc_stream(
        upstream_data: *const u8,
        upstream_size: usize,
        service_name_data: *const u8,
        service_name_size: usize,
        method_name_data: *const u8,
        method_name_size: usize,
        initial_metadata_data: *const u8,
        initial_metadata_size: usize,
        return_stream_id: *mut u32,
    ) -> Status;
    pub fn proxy_grpc_send(
        token: u32,
        message_ptr: *const u8,
        message_len: usize,
        end_stream: bool,
    ) -> Status;
    pub fn proxy_grpc_cancel(token_id: u32) -> Status;
    pub fn proxy_grpc_close(token_id: u32) -> Status;
    pub fn proxy_get_status(
        return_code: *mut u32,
        return_message_data: *mut *mut u8,
        return_message_size: *mut usize,
    ) -> Status;
    pub fn proxy_set_effective_context(context_id: u32) -> Status;
    pub fn proxy_call_foreign_function(
        function_name_data: *const u8,
        function_name_size: usize,
        arguments_data: *const u8,
        arguments_size: usize,
        results_data: *mut *mut u8,
        results_size: *mut usize,
    ) -> Status;
    pub fn proxy_done() -> Status;
    pub fn proxy_define_metric(
        metric_type: MetricType,
        name_data: *const u8,
        name_size: usize,
        return_id: *mut u32,
    ) -> Status;
    pub fn proxy_get_metric(metric_id: u32, return_value: *mut u64) -> Status;
    pub fn proxy_record_metric(metric_id: u32, value: u64) -> Status;
    pub fn proxy_increment_metric(metric_id: u32, offset: i64) -> Status;
}

pub fn log(level: LogLevel, message: &str) -> Result<(), Status> {
    unsafe {
        match proxy_log(level, message.as_ptr(), message.len()) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

#[allow(dead_code)]
pub fn get_log_level() -> Result<LogLevel, Status> {
    let mut return_level = LogLevel::Trace;
    unsafe {
        match proxy_get_log_level(&mut return_level) {
            Status::Ok => Ok(return_level),
            e => Err(e),
        }
    }
}

pub fn get_current_time() -> Result<SystemTime, Status> {
    let mut return_time = 0;
    unsafe {
        match proxy_get_current_time_nanoseconds(&mut return_time) {
            Status::Ok => Ok(UNIX_EPOCH + Duration::from_nanos(return_time)),
            e => Err(e),
        }
    }
}

pub fn set_tick_period(period: Duration) -> Result<(), Status> {
    unsafe {
        match proxy_set_tick_period_milliseconds(period.as_millis() as u32) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn get_buffer(
    buffer_type: BufferType,
    start: usize,
    max_size: usize,
) -> Result<Option<Vec<u8>>, Status> {
    let mut return_data = null_mut();
    let mut return_size = 0;
    unsafe {
        match proxy_get_buffer_bytes(
            buffer_type,
            start,
            max_size,
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => Ok(NonNull::new(return_data).map(|return_data| {
                Vec::from_raw_parts(return_data.as_ptr(), return_size, return_size)
            })),
            Status::NotFound => Ok(None),
            e => Err(e),
        }
    }
}

pub fn set_buffer(
    buffer_type: BufferType,
    start: usize,
    size: usize,
    value: &[u8],
) -> Result<(), Status> {
    unsafe {
        match proxy_set_buffer_bytes(buffer_type, start, size, value.as_ptr(), value.len()) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn get_map(map_type: MapType) -> Result<Option<Vec<(String, Vec<u8>)>>, Status> {
    unsafe {
        let mut return_data = null_mut();
        let mut return_size = 0;
        match proxy_get_header_map_pairs(map_type, &mut return_data, &mut return_size) {
            Status::Ok => NonNull::new(return_data)
                .map(|return_data| {
                    let serialized_map =
                        Vec::from_raw_parts(return_data.as_ptr(), return_size, return_size);
                    utils::deserialize_map_bytes(&serialized_map)
                })
                .transpose(),
            Status::NotFound => Ok(None),
            e => Err(e),
        }
    }
}

pub fn set_map(map_type: MapType, map: &[(&str, &[u8])]) -> Result<(), Status> {
    let serialized_map = utils::serialize_map(map);
    unsafe {
        match proxy_set_header_map_pairs(map_type, serialized_map.as_ptr(), serialized_map.len()) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn get_map_value(map_type: MapType, key: &str) -> Result<Option<Vec<u8>>, Status> {
    let mut return_data = null_mut();
    let mut return_size = 0;
    unsafe {
        match proxy_get_header_map_value(
            map_type,
            key.as_ptr(),
            key.len(),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => Ok(NonNull::new(return_data).map(|return_data| {
                Vec::from_raw_parts(return_data.as_ptr(), return_size, return_size)
            })),
            Status::NotFound => Ok(None),
            e => Err(e),
        }
    }
}

pub fn set_map_value(map_type: MapType, key: &str, value: Option<&[u8]>) -> Result<(), Status> {
    unsafe {
        if let Some(value) = value {
            match proxy_replace_header_map_value(
                map_type,
                key.as_ptr(),
                key.len(),
                value.as_ptr(),
                value.len(),
            ) {
                Status::Ok => Ok(()),
                e => Err(e),
            }
        } else {
            match proxy_remove_header_map_value(map_type, key.as_ptr(), key.len()) {
                Status::Ok => Ok(()),
                e => Err(e),
            }
        }
    }
}

pub fn add_map_value(map_type: MapType, key: &str, value: &[u8]) -> Result<(), Status> {
    unsafe {
        match proxy_add_header_map_value(
            map_type,
            key.as_ptr(),
            key.len(),
            value.as_ptr(),
            value.len(),
        ) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn get_property<S: AsRef<str>>(
    path: impl IntoIterator<Item = S>,
) -> Result<Option<Vec<u8>>, Status> {
    let serialized_path = utils::serialize_property_path(path);
    let mut return_data = null_mut();
    let mut return_size = 0;
    unsafe {
        match proxy_get_property(
            serialized_path.as_ptr(),
            serialized_path.len(),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => Ok(NonNull::new(return_data).map(|return_data| {
                Vec::from_raw_parts(return_data.as_ptr(), return_size, return_size)
            })),
            Status::NotFound => Ok(None),
            e => Err(e),
        }
    }
}

pub fn set_property<S: AsRef<str>>(
    path: impl IntoIterator<Item = S>,
    value: Option<impl AsRef<[u8]>>,
) -> Result<(), Status> {
    let serialized_path = utils::serialize_property_path(path);
    let value = value.as_ref().map(|x| x.as_ref());
    unsafe {
        match proxy_set_property(
            serialized_path.as_ptr(),
            serialized_path.len(),
            value.map_or(null(), |value| value.as_ptr()),
            value.map_or(0, |value| value.len()),
        ) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn get_shared_data(key: impl AsRef<str>) -> Result<(Option<Vec<u8>>, Option<u32>), Status> {
    let mut return_data = null_mut();
    let mut return_size = 0;
    let mut return_cas = 0;
    let key = key.as_ref();
    unsafe {
        match proxy_get_shared_data(
            key.as_ptr(),
            key.len(),
            &mut return_data,
            &mut return_size,
            &mut return_cas,
        ) {
            Status::Ok => {
                let cas = match return_cas {
                    0 => None,
                    cas => Some(cas),
                };
                Ok((
                    NonNull::new(return_data).map(|return_data| {
                        Vec::from_raw_parts(return_data.as_ptr(), return_size, return_size)
                    }),
                    cas,
                ))
            }
            Status::NotFound => Ok((None, None)),
            e => Err(e),
        }
    }
}

pub fn set_shared_data(
    key: impl AsRef<str>,
    value: Option<impl AsRef<[u8]>>,
    cas: Option<u32>,
) -> Result<(), Status> {
    let key = key.as_ref();
    let value = value.as_ref().map(|x| x.as_ref());
    unsafe {
        match proxy_set_shared_data(
            key.as_ptr(),
            key.len(),
            value.map_or(null(), |value| value.as_ptr()),
            value.map_or(0, |value| value.len()),
            cas.unwrap_or(0),
        ) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn register_shared_queue(name: impl AsRef<str>) -> Result<u32, Status> {
    let name = name.as_ref();
    unsafe {
        let mut return_id = 0;
        match proxy_register_shared_queue(name.as_ptr(), name.len(), &mut return_id) {
            Status::Ok => Ok(return_id),
            e => Err(e),
        }
    }
}

pub fn resolve_shared_queue(
    vm_id: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Option<u32>, Status> {
    let vm_id = vm_id.as_ref();
    let name = name.as_ref();
    let mut return_id = 0;
    unsafe {
        match proxy_resolve_shared_queue(
            vm_id.as_ptr(),
            vm_id.len(),
            name.as_ptr(),
            name.len(),
            &mut return_id,
        ) {
            Status::Ok => Ok(Some(return_id)),
            Status::NotFound => Ok(None),
            e => Err(e),
        }
    }
}

pub fn dequeue_shared_queue(queue_id: u32) -> Result<Option<Vec<u8>>, Status> {
    let mut return_data = null_mut();
    let mut return_size = 0;
    unsafe {
        match proxy_dequeue_shared_queue(queue_id, &mut return_data, &mut return_size) {
            Status::Ok => Ok(Some(Vec::from_raw_parts(
                return_data,
                return_size,
                return_size,
            ))),
            Status::Empty => Ok(None),
            e => Err(e),
        }
    }
}

pub fn enqueue_shared_queue(queue_id: u32, value: impl AsRef<[u8]>) -> Result<(), Status> {
    let value = value.as_ref();
    unsafe {
        match proxy_enqueue_shared_queue(queue_id, value.as_ptr(), value.len()) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn resume_downstream() -> Result<(), Status> {
    unsafe {
        match proxy_continue_stream(StreamType::Downstream) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn resume_upstream() -> Result<(), Status> {
    unsafe {
        match proxy_continue_stream(StreamType::Upstream) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn resume_http_request() -> Result<(), Status> {
    unsafe {
        match proxy_continue_stream(StreamType::HttpRequest) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn resume_http_response() -> Result<(), Status> {
    unsafe {
        match proxy_continue_stream(StreamType::HttpResponse) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn close_downstream() -> Result<(), Status> {
    unsafe {
        match proxy_close_stream(StreamType::Downstream) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}
pub fn close_upstream() -> Result<(), Status> {
    unsafe {
        match proxy_close_stream(StreamType::Upstream) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn reset_http_request() -> Result<(), Status> {
    unsafe {
        match proxy_close_stream(StreamType::HttpRequest) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn reset_http_response() -> Result<(), Status> {
    unsafe {
        match proxy_close_stream(StreamType::HttpResponse) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn send_http_response(
    status_code: u32,
    headers: &[(&str, &[u8])],
    body: Option<&[u8]>,
) -> Result<(), Status> {
    let serialized_headers = utils::serialize_map(headers);
    unsafe {
        match proxy_send_local_response(
            status_code,
            null(),
            0,
            body.map_or(null(), |body| body.as_ptr()),
            body.map_or(0, |body| body.len()),
            serialized_headers.as_ptr(),
            serialized_headers.len(),
            -1,
        ) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn dispatch_http_call(
    upstream: &[u8],
    headers: &[(&str, &[u8])],
    body: Option<&[u8]>,
    trailers: &[(&str, &[u8])],
    timeout: Duration,
) -> Result<u32, Status> {
    let serialized_headers = utils::serialize_map(headers);
    let serialized_trailers = utils::serialize_map(trailers);
    let mut return_token = 0;
    unsafe {
        match proxy_http_call(
            upstream.as_ptr(),
            upstream.len(),
            serialized_headers.as_ptr(),
            serialized_headers.len(),
            body.map_or(null(), |body| body.as_ptr()),
            body.map_or(0, |body| body.len()),
            serialized_trailers.as_ptr(),
            serialized_trailers.len(),
            timeout.as_millis() as u32,
            &mut return_token,
        ) {
            Status::Ok => Ok(return_token),
            e => Err(e),
        }
    }
}

pub fn dispatch_grpc_call(
    upstream_name: &[u8],
    service_name: &str,
    method_name: &str,
    initial_metadata: &[(&str, &[u8])],
    message: Option<&[u8]>,
    timeout: Duration,
) -> Result<u32, Status> {
    let mut return_callout_id = 0;
    let serialized_initial_metadata = utils::serialize_map(initial_metadata);
    unsafe {
        match proxy_grpc_call(
            upstream_name.as_ptr(),
            upstream_name.len(),
            service_name.as_ptr(),
            service_name.len(),
            method_name.as_ptr(),
            method_name.len(),
            serialized_initial_metadata.as_ptr(),
            serialized_initial_metadata.len(),
            message.map_or(null(), |message| message.as_ptr()),
            message.map_or(0, |message| message.len()),
            timeout.as_millis() as u32,
            &mut return_callout_id,
        ) {
            Status::Ok => Ok(return_callout_id),
            e => Err(e),
        }
    }
}

pub fn open_grpc_stream(
    upstream_name: &[u8],
    service_name: &str,
    method_name: &str,
    initial_metadata: &[(&str, &[u8])],
) -> Result<u32, Status> {
    let mut return_stream_id = 0;
    let serialized_initial_metadata = utils::serialize_map(initial_metadata);
    unsafe {
        match proxy_grpc_stream(
            upstream_name.as_ptr(),
            upstream_name.len(),
            service_name.as_ptr(),
            service_name.len(),
            method_name.as_ptr(),
            method_name.len(),
            serialized_initial_metadata.as_ptr(),
            serialized_initial_metadata.len(),
            &mut return_stream_id,
        ) {
            Status::Ok => Ok(return_stream_id),
            e => Err(e),
        }
    }
}

pub fn send_grpc_stream_message(
    token: u32,
    message: Option<&[u8]>,
    end_stream: bool,
) -> Result<(), Status> {
    unsafe {
        match proxy_grpc_send(
            token,
            message.map_or(null(), |message| message.as_ptr()),
            message.map_or(0, |message| message.len()),
            end_stream,
        ) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn cancel_grpc_call(token_id: u32) -> Result<(), Status> {
    unsafe {
        match proxy_grpc_cancel(token_id) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn cancel_grpc_stream(token_id: u32) -> Result<(), Status> {
    unsafe {
        match proxy_grpc_cancel(token_id) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn close_grpc_stream(token_id: u32) -> Result<(), Status> {
    unsafe {
        match proxy_grpc_close(token_id) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn get_grpc_status() -> Result<(u32, Option<String>), Status> {
    let mut return_code = 0;
    let mut return_data = null_mut();
    let mut return_size = 0;
    unsafe {
        match proxy_get_status(&mut return_code, &mut return_data, &mut return_size) {
            Status::Ok => Ok((
                return_code,
                NonNull::new(return_data).and_then(|return_data| {
                    String::from_utf8(Vec::from_raw_parts(
                        return_data.as_ptr(),
                        return_size,
                        return_size,
                    ))
                    .ok()
                }),
            )),
            e => Err(e),
        }
    }
}

pub fn set_effective_context(context_id: u32) -> Result<(), Status> {
    unsafe {
        match proxy_set_effective_context(context_id) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

/// Calls a foreign function as defined by the proxy.
pub fn call_foreign_function(
    function_name: impl AsRef<str>,
    arguments: Option<impl AsRef<[u8]>>,
) -> Result<Option<Vec<u8>>, Status> {
    let mut return_data = null_mut();
    let mut return_size = 0;
    let function_name = function_name.as_ref();
    let arguments = arguments.as_ref().map(|x| x.as_ref());
    unsafe {
        match proxy_call_foreign_function(
            function_name.as_ptr(),
            function_name.len(),
            arguments.map_or(null(), |arguments| arguments.as_ptr()),
            arguments.map_or(0, |arguments| arguments.len()),
            &mut return_data,
            &mut return_size,
        ) {
            Status::Ok => Ok(NonNull::new(return_data).map(|return_data| {
                Vec::from_raw_parts(return_data.as_ptr(), return_size, return_size)
            })),
            e => Err(e),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
lazy_static::lazy_static! {
    static ref LIBRARY: libloading::os::unix::Library = libloading::os::unix::Library::this();
    static ref PROXY_WRITE_UPSTREAM: Option<libloading::os::unix::Symbol<unsafe extern "C" fn(*const u8, usize) -> Status>> = unsafe { LIBRARY.get(b"proxy_write_upstream").ok() };
    static ref PROXY_WRITE_DOWNSTREAM: Option<libloading::os::unix::Symbol<unsafe extern "C" fn(*const u8, usize) -> Status>> = unsafe { LIBRARY.get(b"proxy_write_downstream").ok() };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_upstream(buffer: &[u8]) -> Result<(), Status> {
    let Some(proxy_write_upstream) = &*PROXY_WRITE_UPSTREAM else {
        return Err(Status::InternalFailure);
    };
    match unsafe { proxy_write_upstream(buffer.as_ptr(), buffer.len()) } {
        Status::Ok => Ok(()),
        e => Err(e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_downstream(buffer: &[u8]) -> Result<(), Status> {
    let Some(proxy_write_downstream) = &*PROXY_WRITE_DOWNSTREAM else {
        return Err(Status::InternalFailure);
    };
    match unsafe { proxy_write_downstream(buffer.as_ptr(), buffer.len()) } {
        Status::Ok => Ok(()),
        e => Err(e),
    }
}

pub fn done() -> Result<(), Status> {
    unsafe {
        match proxy_done() {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn define_metric(metric_type: MetricType, name: &str) -> Result<u32, Status> {
    let mut return_id = 0;
    unsafe {
        match proxy_define_metric(metric_type, name.as_ptr(), name.len(), &mut return_id) {
            Status::Ok => Ok(return_id),
            e => Err(e),
        }
    }
}

pub fn get_metric(metric_id: u32) -> Result<u64, Status> {
    let mut return_value = 0;
    unsafe {
        match proxy_get_metric(metric_id, &mut return_value) {
            Status::Ok => Ok(return_value),
            e => Err(e),
        }
    }
}

pub fn record_metric(metric_id: u32, value: u64) -> Result<(), Status> {
    unsafe {
        match proxy_record_metric(metric_id, value) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

pub fn increment_metric(metric_id: u32, offset: i64) -> Result<(), Status> {
    unsafe {
        match proxy_increment_metric(metric_id, offset) {
            Status::Ok => Ok(()),
            e => Err(e),
        }
    }
}

mod utils {
    use super::Status;
    use std::ops::Range;

    pub(super) fn serialize_property_path<S: AsRef<str>>(
        path: impl IntoIterator<Item = S>,
    ) -> Vec<u8> {
        let mut out = Vec::new();
        for part in path {
            out.extend_from_slice(part.as_ref().as_bytes());
            out.push(0);
        }
        if !out.is_empty() {
            out.pop();
        }
        out
    }

    pub(super) fn serialize_map(map: &[(&str, &[u8])]) -> Vec<u8> {
        let mut size: usize = 4;
        for (name, value) in map {
            size += name.len() + value.len() + 10;
        }
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&(map.len() as u32).to_le_bytes());
        for (name, value) in map {
            bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        }
        for (name, value) in map {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
        bytes
    }

    pub(super) fn deserialize_map_bytes(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, Status> {
        let mut map = Vec::new();
        if bytes.is_empty() {
            return Ok(map);
        }
        let get = |r: Range<usize>| bytes.get(r).ok_or(Status::ParseFailure);

        let size = u32::from_le_bytes(get(0..4)?.try_into().unwrap()) as usize;
        let mut p = 4 + size * 8;
        for n in 0..size {
            let s = 4 + n * 8;
            let size = u32::from_le_bytes(get(s..s + 4)?.try_into().unwrap()) as usize;
            let key = get(p..p + size)?;
            p += size + 1;
            let size = u32::from_le_bytes(get(s + 4..s + 8)?.try_into().unwrap()) as usize;
            let value = get(p..p + size)?;
            p += size + 1;
            map.push((String::from_utf8(key.to_vec()).unwrap(), value.to_vec()));
        }
        Ok(map)
    }
}
