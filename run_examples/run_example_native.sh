set -e
here=$(realpath $(dirname "$0"))

cd "$here/.."
cargo build --target x86_64-unknown-linux-gnu --release --example mini_proxy
cd "$here"
cp ../target/x86_64-unknown-linux-gnu/release/examples/libmini_proxy.so config/mini_proxy.so
docker-compose -f docker-compose-native.yaml down -v
docker-compose -f docker-compose-native.yaml build --no-cache
docker-compose -f docker-compose-native.yaml up
rm config/mini_proxy.so