use std::borrow::Cow;

use prost::Message;

use crate::upstream::Upstream;

use self::grpc_service::grpc_service::{
    google_grpc::{channel_credentials::CredentialSpecifier, ChannelCredentials, SslCredentials},
    EnvoyGrpc, GoogleGrpc, TargetSpecifier,
};

mod grpc_service {
    include!(concat!(env!("OUT_DIR"), "/envoy.config.core.v3.rs"));
}

impl<'a> Upstream<'a> {
    /// Creates an Envoy-compatible upstream configuration for the given upstream cluster name
    pub fn envoy_upstream(cluster_name: impl ToString, authority: impl ToString) -> Self {
        let cluster_name = cluster_name.to_string();
        let service = grpc_service::GrpcService {
            target_specifier: Some(TargetSpecifier::EnvoyGrpc(EnvoyGrpc {
                authority: authority.to_string(),
                cluster_name,
            })),
            ..Default::default()
        };
        Self(Cow::Owned(service.encode_to_vec()))
    }

    /// like grpc_upstream, but without TLS. used for testing purposes
    pub fn insecure_grpc_upstream(target_uri: impl ToString) -> Self {
        let service = grpc_service::GrpcService {
            target_specifier: Some(TargetSpecifier::GoogleGrpc(GoogleGrpc {
                target_uri: target_uri.to_string(),
                channel_credentials: None,
                call_credentials: vec![],
                channel_args: Default::default(),
                config: Default::default(),
                credentials_factory_name: String::new(),
                per_stream_buffer_limit_bytes: None,
                stat_prefix: "leaksignal_command".to_string(),
            })),
            ..Default::default()
        };
        Self(Cow::Owned(service.encode_to_vec()))
    }

    pub fn grpc_upstream(target_uri: impl ToString) -> Self {
        let target_uri = target_uri.to_string();
        let is_plaintext = target_uri.starts_with("http://");
        let target_uri = target_uri
            .strip_prefix(if is_plaintext { "http://" } else { "https://" })
            .unwrap_or(&*target_uri)
            .to_string();
        let service = grpc_service::GrpcService {
            target_specifier: Some(TargetSpecifier::GoogleGrpc(GoogleGrpc {
                channel_credentials: if is_plaintext {
                    None
                } else {
                    Some(ChannelCredentials {
                        credential_specifier: Some(CredentialSpecifier::SslCredentials(
                            SslCredentials {
                                root_certs: None,
                                private_key: None,
                                cert_chain: None,
                            },
                        )),
                    })
                },
                target_uri,
                call_credentials: vec![],
                channel_args: Default::default(),
                config: Default::default(),
                credentials_factory_name: String::new(),
                per_stream_buffer_limit_bytes: None,
                stat_prefix: "leaksignal_command".to_string(),
            })),
            ..Default::default()
        };
        Self(Cow::Owned(service.encode_to_vec()))
    }
}
