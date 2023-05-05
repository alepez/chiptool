rm -r pac-generated
mkdir -p pac-generated/src
cargo run --release -- generate --svd TC37X.svd
mv lib.rs pac-generated/src
cp Cargo.toml.template pac-generated/Cargo.toml
cd pac-generated && cargo fmt && cargo build