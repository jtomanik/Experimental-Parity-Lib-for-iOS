# Experimental Parity Light Client for iOS 

This is repo contains experimental work that aims at creating a build of Parity Library that works on iOS devices. This project is still at an early stage. 

## iOS library

This guide assumes that you already have an Xcode 10.1 installed and configured for development. 
Moreover, you need an iPhone device and a lightning cable in order to run builds on the device.  

## Get sources

### Example test project

Please check out [ParityStaticLibTest](https://github.com/jtomanik/ParityStaticLibTest) repository. 
This is a very basic project and you do not need any additional third-party tools to use it. 

After checking out that repo you won't be able to run that project due to linker error:

```
ld: library not found for -lparity
``` 

You will need to build a Parity library and place `libparity.a` in a `/lib` folder.

### Experimental Parity library

In order to build an experimental build of parity library for iOS, you need to check `feature/iOSlib` from this repo.
Next you will have to patch all the necessary dependencies and finally, you will be able to build the library. 

Note: every time when I use `~/.cargo/registry/src/github.com/` in a path I refer to a folder that contains sources checked out by `cargo`. 
Please adjust it accordingly to match a path where cargo stores sources on your machine.   


### Patching heapsize

Open file `~/.cargo/registry/src/github.com/heapsize-0.4.2/src/lib.rs`

There at a line 44 you should be able to find:

```
#[cfg_attr(any(prefixed_jemalloc, target_os = "macos", target_os = "ios", target_os = "android"), link_name = "je_malloc_usable_size")]
```

This line forces `malloc_usable_size` to be linked externally as `je_malloc_usable_size`. This works only if we're compiling an executable as Rust comes with jemalloc.
However, if we compile into the static lib Rust links against the system allocator and above will not work on systems that do not have jemalloc.
Unfortunately, iOS is one of such systems.

In order to patch heapsize and use on Mac's or iOS's malloc we need to change line 44 to:

```
#[cfg_attr(any(target_os = "macos", target_os = "ios"), link_name = "malloc_size")]
```  

### Patching Rust-Crypto
Open file `~/.cargo/registry/src/github.com/rust-crypto-0.2.36/src/util_helpers.c`  

There at the line 104 insert following snippet:

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

This snippet adds support for ARM8 architecture and comes from [this PR](https://github.com/DaGenix/rust-crypto/pull/384)

### Patching parity-rocksdb-sys
Open file: `~/.cargo/registry/src/github.com/parity-rocksdb-sys-0.5.3/build.rs`

Go to line 25 and insert following snippet:

```
    if target_os.contains("ios") {
        cfg.cxxflag("-fno-rtti");
        cfg.cxxflag("-DIOS_CROSS_COMPILE");
    }
```

This adds necessary flags in order to cross-compile this lib.

Please note that this is not an ideal solution, nor even preferable one just first that worked. RocksDB should be compiled in "LITE" mode on iOS
as described [in INSTALL.md](https://github.com/facebook/rocksdb/blob/master/INSTALL.md) however, this currently would not work
as `C` interface is not included with the LITE build. See "future work" section for more details.

### Patching kvdb-rocksdb
Open file: `~/.cargo/registry/src/github.com/kvdb-rocksdb-0.1.4/src/lib.rs`

Go to the line 226 where `fn col_config(config: &DatabaseConfig, block_opts: &BlockBasedOptions) -> io::Result<Options>`
function starts and replace it with the following: 
```
fn col_config(config: &DatabaseConfig, block_opts: &BlockBasedOptions) -> io::Result<Options> {
    let mut opts = Options::new();
    #[cfg(target_os = "ios")] {
        Ok(opts)
    }

    #[cfg(not(target_os = "ios"))] {
        opts.set_parsed_options("level_compaction_dynamic_level_bytes=true").map_err(other_io_err)?;

        opts.set_block_based_table_factory(block_opts);

        opts.set_parsed_options(
            &format!("block_based_table_factory={{{};{}}}",
                     "cache_index_and_filter_blocks=true",
                     "pin_l0_filter_and_index_blocks_in_cache=true")).map_err(other_io_err)?;

        opts.optimize_level_style_compaction(config.memory_budget_per_col() as i32);
        opts.set_target_file_size_base(config.compaction.initial_file_size);

        opts.set_parsed_options("compression_per_level=").map_err(other_io_err)?;

        Ok(opts)
    }
}
```

Then go to line 284 where you should have `impl Database` and replace the following fragment:
```
impl Database {
    const CORRUPTION_FILE_NAME: &'static str = "CORRUPTED";

    /// Open database with default settings.
    pub fn open_default(path: &str) -> io::Result<Database> {
        Database::open(&DatabaseConfig::default(), path)
    }

    /// Open database file. Creates if it does not exist.
    pub fn open(config: &DatabaseConfig, path: &str) -> io::Result<Database> {
        let mut opts = Options::new();

        if let Some(rate_limit) = config.compaction.write_rate_limit {
            opts.set_parsed_options(&format!("rate_limiter_bytes_per_sec={}", rate_limit)).map_err(other_io_err)?;
        }
        opts.set_use_fsync(false);
        opts.create_if_missing(true);
        opts.set_max_open_files(config.max_open_files);
        opts.set_parsed_options("keep_log_file_num=1").map_err(other_io_err)?;
        opts.set_parsed_options("bytes_per_sync=1048576").map_err(other_io_err)?;
        opts.set_db_write_buffer_size(config.memory_budget_per_col() / 2);
        opts.increase_parallelism(cmp::max(1, ::num_cpus::get() as i32 / 2));

        let mut block_opts = BlockBasedOptions::new();

        {
            block_opts.set_block_size(config.compaction.block_size);
            let cache_size = cmp::max(8, config.memory_budget() / 3);
            let cache = Cache::new(cache_size);
            block_opts.set_cache(cache);
        }

        // attempt database repair if it has been previously marked as corrupted
        let db_corrupted = Path::new(path).join(Database::CORRUPTION_FILE_NAME);
        if db_corrupted.exists() {
            warn!("DB has been previously marked as corrupted, attempting repair");
            DB::repair(&opts, path).map_err(other_io_err)?;
            fs::remove_file(db_corrupted)?;
        }
```

with:

```
impl Database {
    const CORRUPTION_FILE_NAME: &'static str = "CORRUPTED";

    /// Open database with default settings.
    pub fn open_default(path: &str) -> io::Result<Database> {
        Database::open(&DatabaseConfig::default(), path)
    }

    /// Open database file. Creates if it does not exist.
    pub fn open(config: &DatabaseConfig, path: &str) -> io::Result<Database> {
        let mut opts = Options::new();

        #[cfg(not(target_os = "ios"))] {
        if let Some(rate_limit) = config.compaction.write_rate_limit {
                opts.set_parsed_options(&format!("rate_limiter_bytes_per_sec={}", rate_limit)).map_err(other_io_err)?;
        }
        opts.set_use_fsync(false);
        opts.set_max_open_files(config.max_open_files);
        opts.set_parsed_options("keep_log_file_num=1").map_err(other_io_err)?;
        opts.set_parsed_options("bytes_per_sync=1048576").map_err(other_io_err)?;
        opts.set_db_write_buffer_size(config.memory_budget_per_col() / 2);
        opts.increase_parallelism(cmp::max(1, ::num_cpus::get() as i32 / 2));
        }

        opts.create_if_missing(true);

        let mut block_opts = BlockBasedOptions::new();
        #[cfg(not(target_os = "ios"))]
        {
            block_opts.set_block_size(config.compaction.block_size);
            let cache_size = cmp::max(8, config.memory_budget() / 3);
            let cache = Cache::new(cache_size);
            block_opts.set_cache(cache);
        }

        // attempt database repair if it has been previously marked as corrupted
        let db_corrupted = Path::new(path).join(Database::CORRUPTION_FILE_NAME);
        if db_corrupted.exists() {
            warn!("DB has been previously marked as corrupted, attempting repair");
            #[cfg(not(target_os = "ios"))]
            DB::repair(&opts, path).map_err(other_io_err)?;
            fs::remove_file(db_corrupted)?;
        }
```

Lastly go to the line 372 and replace following snippet:
```
Err(ref s) if is_corrupted(s) => {
    warn!("DB corrupted: {}, attempting repair", s);
    DB::repair(&opts, path).map_err(other_io_err)?;
``` 

with:
```
Err(ref s) if is_corrupted(s) => {
    warn!("DB corrupted: {}, attempting repair", s);
    #[cfg(not(target_os = "ios"))]
    DB::repair(&opts, path).map_err(other_io_err)?;
``` 

Because RocksDB is built in a "full" mode it contains features that do not work iOS (or in any LITE build)
So I have disabled all custom configuration and explicitly unsupported functions in order to make a build that works,
Please see "Future work" sections for more information.

## Building iOS library

### Xcode Environment

Before you continue make sure you have Xcode build tools. If you already have the build tools installed and they are up to date, you can skip this step.
Otherwise, execute the following command in the terminal
```
xcode-select --install
```

### Rust environment (for non-Rust developers)
We will be using `rustup`. If you already have rustup installed, you can skip this step. 
Rustup installs Rust from the official release channels and enables you to easily switch between different release versions.

```
curl https://sh.rustup.rs -sSf | sh
```

Add the iOS architectures to `rustup` so we can use them during cross-compilation.
```
rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
```

When you installed Rust, it also installed `cargo`, which is a package. Now we will use cargo to install cargo-lipo. 
This is a cargo subcommand which automatically creates a universal library for use with iOS. 
Without this crate, cross-compiling Rust to work on iOS is infinitely harder.
```
cargo install cargo-lipo
```

Note: This guide was tested with `rustc 1.30.0 (da5f414c2 2018-10-24)`

### Building the library 

From the root folder of this repo ( one that contains this readme ) go to the `\parity-clib` subfolder and execute the following command:

```
cargo lipo --features malloc --release
```

Now go for a walk or just make yourself a coffee and fetch your favorite book. This process can take around an hour on a decent MacBook Pro.

### Copying the library
If everything went well you should be able to find `libparity.a` in the following directory (relative to the root folder of the repo):

`target\`

Please copy that file into the `\lib` directory in the folder where you have checked out ParityStaticLibTest repo.

### Running the ParityStaticLibTest 

Xcode console log:
```

```

# Future work

This section contains currently planned steps on a road to the viable iOS framework:

## Potential improvements

### Performance benchmarking
Before starting further work on improving the performance I need to create a way to automatically log results and do test runs
on different devices. That will allow me to establish a baseline performance profile and properly measure future performance gains. 

### RocksDB tuning
Currently, RocksDB runs on default settings, I'll have to research what settings are supported on mobile devices and what changes may bring performance improvements.

### Jemalloc
In my tests so far builds with Jemalloc got at least 2x performance boost. I'll have to revisit that work and figure out how to enable Jemalloc while building RocksDB 

### RocksDB Lite
RockDB has a special "LITE" compile option that creates a build optimised for use on mobile devices. Unfortunately, currently, that build does not contain a C interface that Rust wrapper depends on.
I'm working on curating normal C interface in order to create a subset that would support features available in a LITE build. Once that done I'll need to update Rust wrapper to use that when compiling for iOS.

### Custom chainspec 
One of the available ways to speed up initial synchronization is to provide a custom chainspec that would contain a hardcoded header that is ~24h old. That will greatly reduce the amount of data needed to process during the initial sync.

# Good to know stuff
For people not familiar with the Rust

## Nightly Rust environment

Available `rustup` options are: `stable`, `beta`, `nightly`

`rustup update nightly`

`rustup override set nightly`

`rustup target add aarch64-apple-ios armv7s-apple-ios armv7-apple-ios x86_64-apple-ios i386-apple-ios`

## Cleaning the environment

Local builds:

`cargo clean`

Downloaded crates:

`rm -rf ~/.cargo/registry/src`

Note: When you discard downloaded crates, you will have to apply patches again.



