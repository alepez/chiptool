rm -r pac_generated
mkdir -p pac_generated/src
cargo run --release -- generate --svd TC37X.svd
mv lib.rs pac_generated/src
cp Cargo.toml.template pac_generated/Cargo.toml
cd pac_generated && cargo fmt && cargo build