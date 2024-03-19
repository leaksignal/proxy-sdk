set -e
here=$(realpath $(dirname "$0"))

cd "$here/.."
cargo build --target wasm32-unknown-unknown --release --example mini_proxy
cd "$here"
cp ../target/wasm32-unknown-unknown/release/examples/mini_proxy.wasm config/mini_proxy.wasm
docker-compose -f docker-compose-wasm.yaml down -v
docker-compose -f docker-compose-wasm.yaml build --no-cache
docker-compose -f docker-compose-wasm.yaml up
rm config/mini_proxy.wasm