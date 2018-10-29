# Building iOS library

## Get sources

## Patching heapsize

Open file `~/.cargo/registry/src/github.com/heapsize-0.4.2/src/lib.rs`

There you should find a line 44 44 that looks like that:

`#[cfg_attr(any(prefixed_jemalloc, target_os = "macos", target_os = "ios", target_os = "android"), link_name = "je_malloc_usable_size")]`

This line forces `malloc_usable_size` to be linked externally as `je_malloc_usable_size`. This works only if we're compiling an executable as Rust comes with jemalloc.
However if we compile into the static lib Rust links against the system allocator and above will not work on systems that do not have jemalloc.
Unfortunately iOS is one of such systems.

In order to patch it for use with Mac's or iOS's malloc we need to change line 44 to:

```
#[cfg_attr(any(target_os = "macos", target_os = "ios"), link_name = "malloc_size")]
```  

## Patching Rust-Crypto
Open file `~/.cargo/registry/src/github.com/rust-crypto-0.2.36/src/util_helpers.c`  

add following snippet at the line 104

```
#ifdef __aarch64__
uint32_t rust_crypto_util_fixed_time_eq_asm(uint8_t* lhsp, uint8_t* rhsp, size_t count) {
    if (count == 0) {
        return 1;
    }
    uint8_t result = 0;
    asm(
        " \
            1: \
            \
            ldrb w4, [%1]; \
            ldrb w5, [%2]; \
            eor w4, w4, w5; \
            orr %w0, %w0, w4; \
            \
            add %w1, %w1, #1; \
            add %w2, %w2, #1; \
            subs %w3, %w3, #1; \
            bne 1b; \
        "
        : "+&r" (result), "+&r" (lhsp), "+&r" (rhsp), "+&r" (count) // all input and output
        : // input
        : "w4", "w5", "cc" // clobbers
   );

    return result;
}
#endif
```

## Nightly Rust environment

`rustup update beta`

`rustup override set nightly`

`rustup target add aarch64-apple-ios armv7s-apple-ios armv7-apple-ios x86_64-apple-ios i386-apple-ios`


## Create 64bit library
Due to different page sizes on 64 bit and 32 bit iPhones and jemalloc configuration when compiling library with jemalloc support we can only target 64 bit platforms.

Before proceeding make sure that yopu are using `nightly` environment.

In order to create universal 64 bit library you need to run following commands from `parity-clib` directory:

`cargo build --lib --target x86_64-apple-ios`

`cargo build --lib --target aarch64-apple-ios`

`lipo -create ../target/aarch64-apple-ios/debug/libparity.a ../target/x86_64-apple-ios/debug/libparity.a -output ../target/debug/libparity.a`


 

# Different memmory allocators for libparity

## 1. system's Malloc

1. Make sure that `heapsize` is patched.
2. Make sure `rust-crypto` is patched
2. create universal library with following command: `cargo lipo --features malloc --release
`

## 2. Rust's jemalloc

1. Make sure that `heapsize` is **not** patched.  
2. Uncomment following lines from `parity-clib/src.lib.rs` 
```
#![feature(alloc_jemalloc)]
#![crate_type = "staticlib"]
```
3. create universal library with following command: `cargo lipo --features jemalloc --release`

## 3. external jemalloc

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

remove `rm -rf ~/.cargo/registry/src`

`memory_profiling`

`nm libparity.a | grep " U " | grep usable`

`cargo build --features memory_profiling`

`./target/debug/parity --light db kill`

`parity --light --no-ipc --no-color --no-ws --chain=/Users/jakubtomanik/github/parity/parity-clib/custom_foundation.json`

`https://gist.github.com/Wizermil/1b8144e4f67511c1f7d6`

`rustup update beta`

`rustup override set nightly`

`rustup override set stable`

`rustup target add aarch64-apple-ios armv7s-apple-ios armv7-apple-ios x86_64-apple-ios i386-apple-ios`

`cargo build --lib --target x86_64-apple-ios`
`cargo build --lib --target aarch64-apple-ios`

`lipo -create ../target/aarch64-apple-ios/debug/libparity.a ../target/x86_64-apple-ios/debug/libparity.a -output ../target/debug/libparity.a`
`lipo -replace arm64  aarch64-apple-ios/release/libparity.a universal/release/libparity.a -output universal/release/libparity2.a`
