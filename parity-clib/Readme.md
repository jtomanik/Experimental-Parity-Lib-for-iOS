# Building iOS library

## Get sources

## Patch heapsize

## Patch Rust-Crypto  

# Misc
`cargo lipo`

`cargo lipo --release`

`cbindgen -o test.h`

`parity --light --no-ipc --no-color --no-ws --no-jsonrpc --base-path=$documentsDirectory --chain=$chainspecPath`

`curl --data '{"method":"eth_syncing","params":[],"id":1,"jsonrpc":"2.0"}' -H "Content-Type: application/json" -X POST localhost:8545`

`enode://2b59afa133c69e1fb93e4a88efe56357e1cc073f971951a6576a8d50df0f38a79d244346c49f09d32cbc70d40107c7ba93a6460a70d9b189cd6d0ae88efac072@35.242.227.201:30303`

`#crate-type = ["staticlib"]`