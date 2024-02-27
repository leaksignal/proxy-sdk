//! <https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/attributes>
//! Can be:
//! * string for UTF-8 strings
//! * bytes for byte buffers
//! * int for 64-bit signed integers
//! * uint for 64-bit unsigned integers
//! * bool for booleans
//! * list for lists of values
//! * map for associative arrays with string keys
//! * timestamp for timestamps as specified by Timestamp
//! * duration for durations as specified by Duration
//! * Protocol buffer message types

use std::{
    fmt,
    net::SocketAddr,
    time::{Duration, SystemTime},
};

use log::warn;

use crate::property::all::AllAttributes;

use super::{get_property_bool, get_property_decode, get_property_int, get_property_string};

mod attributes_proto {
    include!(concat!(env!("OUT_DIR"), "/proxywasm.attributes.rs"));
}
pub use attributes_proto::*;

pub struct Attributes {
    pub request: RequestAttributes,
    pub response: ResponseAttributes,
    pub connection: ConnectionAttributes,
    pub upstream: UpstreamAttributes,
    pub metadata: MetadataAttributes,
    pub configuration: ConfigurationAttributes,
    pub wasm: WasmAttributes,
}

impl Attributes {
    // the internal attribute structs have a private field to ensure a user cant construct them
    pub(crate) fn get() -> Self {
        Self {
            request: RequestAttributes(()),
            response: ResponseAttributes(()),
            connection: ConnectionAttributes(()),
            upstream: UpstreamAttributes(()),
            metadata: MetadataAttributes(()),
            configuration: ConfigurationAttributes(()),
            wasm: WasmAttributes(()),
        }
    }
}

impl fmt::Debug for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", AllAttributes::get(self))
    }
}

/// Available in HTTP filters during a request.
pub struct RequestAttributes(());

impl RequestAttributes {
    /// The path portion of the URL
    pub fn path(&self) -> Option<String> {
        get_property_string("request.path")
    }

    /// The path portion of the URL without the query string
    pub fn url_path(&self) -> Option<String> {
        get_property_string("request.url_path")
    }

    /// The host portion of the URL
    pub fn host(&self) -> Option<String> {
        get_property_string("request.host")
    }

    /// The scheme portion of the URL e.g. “http”
    pub fn scheme(&self) -> Option<String> {
        get_property_string("request.scheme")
    }

    /// Request method e.g. “GET”
    pub fn method(&self) -> Option<String> {
        get_property_string("request.scheme")
    }

    /// All request headers indexed by the lower-cased header name
    /// Header values in request.headers associative array are comma-concatenated in case of multiple values.
    pub fn headers(&self) -> Option<Vec<(String, Vec<u8>)>> {
        let headers = get_property_decode::<attributes_proto::StringMap>("request.headers")?;
        Some(headers.map.into_iter().map(|x| (x.key, x.value)).collect())
    }

    /// Referer request header
    pub fn referer(&self) -> Option<String> {
        get_property_string("request.referer")
    }

    /// User agent request header
    pub fn useragent(&self) -> Option<String> {
        get_property_string("request.useragent")
    }

    /// Time of the first byte received
    pub fn time(&self) -> Option<SystemTime> {
        let raw = get_property_decode::<prost_types::Timestamp>("request.time")?;
        if raw.seconds < 0 || raw.nanos < 0 {
            warn!("request.time returned a negative timestamp, skipped");
            None
        } else {
            Some(SystemTime::UNIX_EPOCH + Duration::new(raw.seconds as u64, raw.nanos as u32))
        }
    }

    /// Request ID corresponding to x-request-id header value
    pub fn id(&self) -> Option<String> {
        get_property_string("request.id")
    }

    /// Request protocol (“HTTP/1.0”, “HTTP/1.1”, “HTTP/2”, or “HTTP/3”)
    pub fn protocol(&self) -> Option<String> {
        get_property_string("request.protocol")
    }

    /// The query portion of the URL in the format of “name1=value1&name2=value2”.
    pub fn query(&self) -> Option<String> {
        get_property_string("request.query")
    }

    /// Total duration of the request
    /// Available in HTTP filters after a request is complete.
    pub fn duration(&self) -> Option<Duration> {
        let raw = get_property_decode::<prost_types::Duration>("request.duration")?;
        if raw.seconds < 0 || raw.nanos < 0 {
            warn!("request.duration returned a negative duration, skipped");
            None
        } else {
            Some(Duration::new(raw.seconds as u64, raw.nanos as u32))
        }
    }

    /// Size of the request body. Content length header is used if available.
    /// Available in HTTP filters after a request is complete.
    pub fn size(&self) -> Option<usize> {
        get_property_int("request.size").map(|x| x as usize)
    }

    /// Total size of the request including the approximate uncompressed size of the headers
    /// Available in HTTP filters after a request is complete.
    pub fn total_size(&self) -> Option<usize> {
        get_property_int("request.total_size").map(|x| x as usize)
    }
}

/// Available in HTTP filters during a response.
pub struct ResponseAttributes(());

impl ResponseAttributes {
    /// Response HTTP status code
    pub fn code(&self) -> Option<u32> {
        get_property_int("response.code").map(|x| x as u32)
    }

    /// Internal response code details (subject to change)
    pub fn code_details(&self) -> Option<String> {
        get_property_string("response.code_details")
    }

    /// Additional details about the response beyond the standard response code encoded as a bit-vector
    pub fn flags(&self) -> Option<u64> {
        get_property_int("response.flags").map(|x| x as u64)
    }

    /// Response gRPC status code
    pub fn grpc_status(&self) -> Option<u32> {
        get_property_int("response.grpc_status").map(|x| x as u32)
    }

    /// All response headers indexed by the lower-cased header name
    /// Header values in response.headers associative array are comma-concatenated in case of multiple values.
    pub fn headers(&self) -> Option<Vec<(String, Vec<u8>)>> {
        let headers = get_property_decode::<attributes_proto::StringMap>("response.headers")?;
        Some(headers.map.into_iter().map(|x| (x.key, x.value)).collect())
    }

    /// All response trailers indexed by the lower-cased trailer name
    /// Header values in response.trailers associative array are comma-concatenated in case of multiple values.
    pub fn trailers(&self) -> Option<Vec<(String, Vec<u8>)>> {
        let headers = get_property_decode::<attributes_proto::StringMap>("response.trailers")?;
        Some(headers.map.into_iter().map(|x| (x.key, x.value)).collect())
    }

    /// The path portion of the URL without the query string
    pub fn size(&self) -> Option<usize> {
        get_property_int("response.size").map(|x| x as usize)
    }

    /// Total size of the response including the approximate uncompressed size of the headers and the trailers
    pub fn total_size(&self) -> Option<usize> {
        get_property_int("response.total_size").map(|x| x as usize)
    }
}

/// The following attributes are available once the downstream connection is established
pub struct ConnectionAttributes(());

impl ConnectionAttributes {
    /// Downstream connection remote address & port
    pub fn source_address(&self) -> Option<SocketAddr> {
        get_property_string("source.address").and_then(|x| x.parse().ok())
    }

    /// Downstream connection remote port
    pub fn source_port(&self) -> Option<u16> {
        get_property_int("source.port").map(|x| x as u16)
    }

    /// Downstream connection local address & port
    pub fn destination_address(&self) -> Option<SocketAddr> {
        get_property_string("destination.address").and_then(|x| x.parse().ok())
    }

    /// Downstream connection local port
    pub fn destination_port(&self) -> Option<u16> {
        get_property_int("destination.port").map(|x| x as u16)
    }

    /// Downstream connection ID
    pub fn id(&self) -> Option<u64> {
        get_property_int("connection.id").map(|x| x as u64)
    }

    /// Indicates whether TLS is applied to the downstream connection and the peer certificate is presented
    pub fn mtls(&self) -> Option<bool> {
        get_property_bool("connection.mtls")
    }

    /// Requested server name in the downstream TLS connection
    pub fn requested_server_name(&self) -> Option<String> {
        get_property_string("connection.requested_server_name")
    }

    /// Requested server name in the downstream TLS connection
    pub fn tls_version(&self) -> Option<String> {
        get_property_string("connection.tls_version")
    }

    /// Requested server name in the downstream TLS connection
    pub fn subject_local_certificate(&self) -> Option<String> {
        get_property_string("connection.subject_local_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn subject_peer_certificate(&self) -> Option<String> {
        get_property_string("connection.subject_peer_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn dns_san_local_certificate(&self) -> Option<String> {
        get_property_string("connection.dns_san_local_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn dns_san_peer_certificate(&self) -> Option<String> {
        get_property_string("connection.dns_san_peer_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn uri_san_local_certificate(&self) -> Option<String> {
        get_property_string("connection.uri_san_local_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn uri_san_peer_certificate(&self) -> Option<String> {
        get_property_string("connection.uri_san_peer_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn sha256_peer_certificate_digest(&self) -> Option<String> {
        get_property_string("connection.sha256_peer_certificate_digest")
    }

    /// The following additional attributes are available upon the downstream connection termination:
    /// Internal termination details of the connection (subject to change)
    pub fn termination_details(&self) -> Option<String> {
        get_property_string("connection.termination_details")
    }
}

/// The following attributes are available once the upstream connection is established
pub struct UpstreamAttributes(());

impl UpstreamAttributes {
    /// Upstream connection remote address & port
    pub fn address(&self) -> Option<SocketAddr> {
        get_property_string("upstream.address").and_then(|x| x.parse().ok())
    }

    /// Upstream connection remote port
    pub fn port(&self) -> Option<u16> {
        get_property_int("upstream.port").map(|x| x as u16)
    }

    /// TLS version of the upstream TLS connection
    pub fn tls_version(&self) -> Option<String> {
        get_property_string("upstream.tls_version")
    }

    /// The subject field of the local certificate in the upstream TLS connection
    pub fn subject_local_certificate(&self) -> Option<String> {
        get_property_string("upstream.subject_local_certificate")
    }

    /// The subject field of the local certificate in the upstream TLS connection
    pub fn subject_peer_certificate(&self) -> Option<String> {
        get_property_string("upstream.subject_peer_certificate")
    }

    /// The first DNS entry in the SAN field of the local certificate in the upstream TLS connection
    pub fn dns_san_local_certificate(&self) -> Option<String> {
        get_property_string("upstream.dns_san_local_certificate")
    }

    /// The first DNS entry in the SAN field of the peer certificate in the upstream TLS connection
    pub fn dns_san_peer_certificate(&self) -> Option<String> {
        get_property_string("upstream.dns_san_peer_certificate")
    }

    /// The first URI entry in the SAN field of the local certificate in the upstream TLS connection
    pub fn uri_san_local_certificate(&self) -> Option<String> {
        get_property_string("upstream.uri_san_local_certificate")
    }

    /// The first URI entry in the SAN field of the peer certificate in the upstream TLS connection
    pub fn uri_san_peer_certificate(&self) -> Option<String> {
        get_property_string("upstream.uri_san_peer_certificate")
    }

    /// Requested server name in the downstream TLS connection
    pub fn sha256_peer_certificate_digest(&self) -> Option<String> {
        get_property_string("upstream.sha256_peer_certificate_digest")
    }

    /// The local address of the upstream connection
    pub fn local_address(&self) -> Option<String> {
        get_property_string("upstream.local_address")
    }

    /// The upstream transport failure reason e.g. certificate validation failed
    pub fn transport_failure_reason(&self) -> Option<String> {
        get_property_string("upstream.transport_failure_reason")
    }
}

/// Data exchanged between filters is available as the following attributes
/// Note that these attributes may change during the life of a request as the data can be updated by filters at any point.
pub struct MetadataAttributes(());

impl MetadataAttributes {
    /// Upstream connection remote address
    pub fn metadata(&self) -> Option<Metadata> {
        get_property_decode("metadata")
    }

    /// Mapping from a filter state name to its serialized string value
    pub fn filter_state(&self) -> Option<Vec<(String, Vec<u8>)>> {
        let headers = get_property_decode::<attributes_proto::StringMap>("filter_state")?;
        Some(headers.map.into_iter().map(|x| (x.key, x.value)).collect())
    }
}

/// Configuration identifiers and metadata related to the handling of the request or the connection is available as the following attributes
pub struct ConfigurationAttributes(());

impl ConfigurationAttributes {
    /// Upstream cluster name
    pub fn cluster_name(&self) -> Option<String> {
        get_property_string("xds.cluster_name")
    }

    /// Upstream cluster metadata
    pub fn cluster_metadata(&self) -> Option<Metadata> {
        get_property_decode("xds.cluster_metadata")
    }

    /// Route name
    pub fn route_name(&self) -> Option<String> {
        get_property_string("xds.route_name")
    }

    /// Route metadata
    pub fn route_metadata(&self) -> Option<Metadata> {
        get_property_decode("xds.route_metadata")
    }

    /// Upstream host metadata
    pub fn upstream_host_metadata(&self) -> Option<Metadata> {
        get_property_decode("xds.upstream_host_metadata")
    }

    /// Listener filter chain name
    pub fn filter_chain_name(&self) -> Option<String> {
        get_property_string("xds.filter_chain_name")
    }
}

#[repr(i64)]
#[derive(Debug)]
pub enum ListenerDirection {
    Unspecified = 0,
    Inbound = 1,
    Outbound = 2,
}

impl ListenerDirection {
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            0 => Some(ListenerDirection::Unspecified),
            1 => Some(ListenerDirection::Inbound),
            2 => Some(ListenerDirection::Outbound),
            _ => None,
        }
    }
}

/// The following extra attributes are available to Wasm extensions
pub struct WasmAttributes(());

impl WasmAttributes {
    pub fn get() -> Self {
        Self(())
    }

    /// Plugin name
    pub fn plugin_name(&self) -> Option<String> {
        get_property_string("plugin_name")
    }

    /// Plugin root ID
    pub fn plugin_root_id(&self) -> Option<String> {
        get_property_string("plugin_root_id")
    }

    /// Plugin VM ID
    pub fn plugin_vm_id(&self) -> Option<String> {
        get_property_string("plugin_vm_id")
    }

    /// Local node description
    pub fn node(&self) -> Option<Node> {
        get_property_decode("node")
    }

    /// Upstream cluster name
    pub fn cluster_name(&self) -> Option<String> {
        get_property_string("cluster_name")
    }

    /// Upstream cluster metadata
    pub fn cluster_metadata(&self) -> Option<Metadata> {
        get_property_decode("cluster_metadata")
    }

    /// Enumeration value of the listener traffic direction
    pub fn listener_direction(&self) -> Option<ListenerDirection> {
        get_property_int("listener_direction").and_then(ListenerDirection::from_i64)
    }

    /// Listener metadata
    pub fn listener_metadata(&self) -> Option<Metadata> {
        get_property_decode("listener_metadata")
    }

    /// Route name
    pub fn route_name(&self) -> Option<String> {
        get_property_string("route_name")
    }

    /// Route metadata
    pub fn route_metadata(&self) -> Option<Metadata> {
        get_property_decode("route_metadata")
    }

    /// Upstream host metadata
    pub fn upstream_host_metadata(&self) -> Option<Metadata> {
        get_property_decode("upstream_host_metadata")
    }
}
