use std::{
    net::SocketAddr,
    time::{Duration, SystemTime},
};

use super::envoy::{Attributes, ListenerDirection, Metadata, Node};

#[derive(Debug)]
pub struct AllAttributes {
    pub request: AllRequestAttributes,
    pub response: AllResponseAttributes,
    pub connection: AllConnectionAttributes,
    pub upstream: AllUpstreamAttributes,
    pub metadata: AllMetadataAttributes,
    pub configuration: AllConfigurationAttributes,
    pub wasm: AllWasmAttributes,
}

impl AllAttributes {
    pub fn get(a: &Attributes) -> Self {
        Self {
            request: AllRequestAttributes::get(a),
            response: AllResponseAttributes::get(a),
            connection: AllConnectionAttributes::get(a),
            upstream: AllUpstreamAttributes::get(a),
            metadata: AllMetadataAttributes::get(a),
            configuration: AllConfigurationAttributes::get(a),
            wasm: AllWasmAttributes::get(a),
        }
    }
}

#[derive(Debug)]
pub struct AllRequestAttributes {
    pub path: Option<String>,
    pub url_path: Option<String>,
    pub host: Option<String>,
    pub scheme: Option<String>,
    pub method: Option<String>,
    pub headers: Option<Vec<(String, Vec<u8>)>>,
    pub referer: Option<String>,
    pub useragent: Option<String>,
    pub time: Option<SystemTime>,
    pub id: Option<String>,
    pub protocol: Option<String>,
    pub query: Option<String>,
    pub duration: Option<Duration>,
    pub size: Option<usize>,
    pub total_size: Option<usize>,
}

impl AllRequestAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            path: a.request.path(),
            url_path: a.request.url_path(),
            host: a.request.host(),
            scheme: a.request.scheme(),
            method: a.request.method(),
            headers: a.request.headers(),
            referer: a.request.referer(),
            useragent: a.request.useragent(),
            time: a.request.time(),
            id: a.request.id(),
            protocol: a.request.protocol(),
            query: a.request.query(),
            duration: a.request.duration(),
            size: a.request.size(),
            total_size: a.request.total_size(),
        }
    }
}

#[derive(Debug)]
pub struct AllResponseAttributes {
    pub code: Option<u32>,
    pub code_details: Option<String>,
    pub flags: Option<u64>,
    pub grpc_status: Option<u32>,
    pub headers: Option<Vec<(String, Vec<u8>)>>,
    pub trailers: Option<Vec<(String, Vec<u8>)>>,
    pub size: Option<usize>,
    pub total_size: Option<usize>,
}

impl AllResponseAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            code: a.response.code(),
            code_details: a.response.code_details(),
            flags: a.response.flags(),
            grpc_status: a.response.grpc_status(),
            headers: a.response.headers(),
            trailers: a.response.trailers(),
            size: a.response.size(),
            total_size: a.response.total_size(),
        }
    }
}

#[derive(Debug)]
pub struct AllConnectionAttributes {
    pub source_address: Option<SocketAddr>,
    pub source_port: Option<u16>,
    pub destination_address: Option<SocketAddr>,
    pub destination_port: Option<u16>,
    pub id: Option<u64>,
    pub mtls: Option<bool>,
    pub requested_server_name: Option<String>,
    pub tls_version: Option<String>,
    pub subject_local_certificate: Option<String>,
    pub subject_peer_certificate: Option<String>,
    pub dns_san_local_certificate: Option<String>,
    pub dns_san_peer_certificate: Option<String>,
    pub uri_san_local_certificate: Option<String>,
    pub uri_san_peer_certificate: Option<String>,
    pub sha256_peer_certificate_digest: Option<String>,
    pub termination_details: Option<String>,
}

impl AllConnectionAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            source_address: a.connection.source_address(),
            source_port: a.connection.source_port(),
            destination_address: a.connection.destination_address(),
            destination_port: a.connection.destination_port(),
            id: a.connection.id(),
            mtls: a.connection.mtls(),
            requested_server_name: a.connection.requested_server_name(),
            tls_version: a.connection.tls_version(),
            subject_local_certificate: a.connection.subject_local_certificate(),
            subject_peer_certificate: a.connection.subject_peer_certificate(),
            dns_san_local_certificate: a.connection.dns_san_local_certificate(),
            dns_san_peer_certificate: a.connection.dns_san_peer_certificate(),
            uri_san_local_certificate: a.connection.uri_san_local_certificate(),
            uri_san_peer_certificate: a.connection.uri_san_peer_certificate(),
            sha256_peer_certificate_digest: a.connection.sha256_peer_certificate_digest(),
            termination_details: a.connection.termination_details(),
        }
    }
}

#[derive(Debug)]
pub struct AllUpstreamAttributes {
    pub address: Option<SocketAddr>,
    pub port: Option<u16>,
    pub tls_version: Option<String>,
    pub subject_local_certificate: Option<String>,
    pub subject_peer_certificate: Option<String>,
    pub dns_san_local_certificate: Option<String>,
    pub dns_san_peer_certificate: Option<String>,
    pub uri_san_local_certificate: Option<String>,
    pub uri_san_peer_certificate: Option<String>,
    pub sha256_peer_certificate_digest: Option<String>,
    pub local_address: Option<String>,
    pub transport_failure_reason: Option<String>,
}

impl AllUpstreamAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            address: a.upstream.address(),
            port: a.upstream.port(),
            tls_version: a.upstream.tls_version(),
            subject_local_certificate: a.upstream.subject_local_certificate(),
            subject_peer_certificate: a.upstream.subject_peer_certificate(),
            dns_san_local_certificate: a.upstream.dns_san_local_certificate(),
            dns_san_peer_certificate: a.upstream.dns_san_peer_certificate(),
            uri_san_local_certificate: a.upstream.uri_san_local_certificate(),
            uri_san_peer_certificate: a.upstream.uri_san_peer_certificate(),
            sha256_peer_certificate_digest: a.upstream.sha256_peer_certificate_digest(),
            local_address: a.upstream.local_address(),
            transport_failure_reason: a.upstream.transport_failure_reason(),
        }
    }
}

#[derive(Debug)]
pub struct AllMetadataAttributes {
    pub metadata: Option<Metadata>,
    pub filter_state: Option<Vec<(String, Vec<u8>)>>,
}

impl AllMetadataAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            metadata: a.metadata.metadata(),
            filter_state: a.metadata.filter_state(),
        }
    }
}

#[derive(Debug)]
pub struct AllConfigurationAttributes {
    pub cluster_name: Option<String>,
    pub cluster_metadata: Option<Metadata>,
    pub route_name: Option<String>,
    pub route_metadata: Option<Metadata>,
    pub upstream_host_metadata: Option<Metadata>,
    pub filter_chain_name: Option<String>,
}

impl AllConfigurationAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            cluster_name: a.configuration.cluster_name(),
            cluster_metadata: a.configuration.cluster_metadata(),
            route_name: a.configuration.route_name(),
            route_metadata: a.configuration.route_metadata(),
            upstream_host_metadata: a.configuration.upstream_host_metadata(),
            filter_chain_name: a.configuration.filter_chain_name(),
        }
    }
}

#[derive(Debug)]
pub struct AllWasmAttributes {
    pub plugin_name: Option<String>,
    pub plugin_root_id: Option<String>,
    pub plugin_vm_id: Option<String>,
    pub node: Option<Node>,
    pub cluster_name: Option<String>,
    pub cluster_metadata: Option<Metadata>,
    pub listener_direction: Option<ListenerDirection>,
    pub listener_metadata: Option<Metadata>,
    pub route_name: Option<String>,
    pub route_metadata: Option<Metadata>,
    pub upstream_host_metadata: Option<Metadata>,
}

impl AllWasmAttributes {
    fn get(a: &Attributes) -> Self {
        Self {
            plugin_name: a.wasm.plugin_name(),
            plugin_root_id: a.wasm.plugin_root_id(),
            plugin_vm_id: a.wasm.plugin_vm_id(),
            node: a.wasm.node(),
            cluster_name: a.wasm.cluster_name(),
            cluster_metadata: a.wasm.cluster_metadata(),
            listener_direction: a.wasm.listener_direction(),
            listener_metadata: a.wasm.listener_metadata(),
            route_name: a.wasm.route_name(),
            route_metadata: a.wasm.route_metadata(),
            upstream_host_metadata: a.wasm.upstream_host_metadata(),
        }
    }
}
