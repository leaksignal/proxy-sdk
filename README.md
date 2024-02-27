
# Proxy-SDK

A rewrite of the `proxy-wasm` rust crate. Has improved ergonomics, more type safety, and [documentation](https://docs.rs/proxy-sdk).

## Proxy->WASM Calls Not Implemented
* `validate_configuration`: Seems unused
* `on_request_metadata`: Unused
* `on_response_metadata`: Unused
* `on_grpc_create_initial_metadata`: Never called/incompletely defined in proxy-wasm-cpp-host
* `on_foreign_function`: Never called from Envoy outside of tests

## Feature Flags

* `stream-metadata`, if enabled, enables GRPC metadata callbacks. Known to cause crashes in some versions of Envoy.
