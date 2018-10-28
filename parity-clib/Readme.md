# Building iOS library

## Get sources

## Patch heapsize

## Patch Rust-Crypto  

# Misc
`cargo lipo`

`cargo lipo --release`

`cargo install cbindgen`

`cbindgen -o test.h`

`parity --light --no-ipc --no-color --no-ws --no-jsonrpc --base-path=$documentsDirectory --chain=$chainspecPath`

`curl --data '{"method":"eth_syncing","params":[],"id":1,"jsonrpc":"2.0"}' -H "Content-Type: application/json" -X POST localhost:8545`

`enode://2b59afa133c69e1fb93e4a88efe56357e1cc073f971951a6576a8d50df0f38a79d244346c49f09d32cbc70d40107c7ba93a6460a70d9b189cd6d0ae88efac072@35.242.227.201:30303`

`#crate-type = ["staticlib"]`

`cargo clean`

remove `~./.cargo/registry/src`

`memory_profiling`

`nm libparity.a | grep " U " | grep usable`

`cargo build --features memory_profiling`

`./target/debug/parity --light db kill`

`parity --light --no-ipc --no-color --no-ws --chain=/Users/jakubtomanik/github/parity/parity-clib/custom_foundation.json`

`https://gist.github.com/Wizermil/1b8144e4f67511c1f7d6`

`rustup update beta`

`rustup override set nightly`

`rustup target add aarch64-apple-ios armv7s-apple-ios armv7-apple-ios x86_64-apple-ios i386-apple-ios`

`cargo build --lib --target x86_64-apple-ios`
`cargo build --lib --target aarch64-apple-ios`

`lipo -create ../target/aarch64-apple-ios/debug/libparity.a ../target/x86_64-apple-ios/debug/libparity.a -output ../target/debug/libparity.a`
