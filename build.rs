fn main() {
    prost_build::Config::default()
        .compile_protos(
            &["proto/grpc_service.proto", "proto/attributes.proto"],
            &["proto"],
        )
        .unwrap();
}
