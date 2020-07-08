# void - A Rust port of [blackhole](https://github.com/adobe/blackhole) - WIP

### Introduction

void is an HTTP sink, to be used for testing & protoyping. Good for testing your outgoing http senders (proxies, http forwarders etc). It is a port of a Golang implementation at the link below.

Please see introduction in the [blackhole](https://github.com/adobe/blackhole#introduction) repository

NOTE: This is done as a learning exercise to evaluate Rust for a different project with similar characteristics.

### Outstanding items (Specific to the task of porting to Rust)

 * CLI interface/options are not implemented yet, but irrelevant for now. The code always saves to current directory with hardcoded filenames.

 * File format and contents have not been verified to be compatible with the Go version or the `replay` tool. There is no plan to make a Rust version of the `replay` tool - currently performance is not critical for replay.

 * The overall design is trying to mimic the design from the Go version. Reasons are below
 ![Design](https://raw.githubusercontent.com/adobe/blackhole/master/design.png)

    * Writing to single file should be done from single writer, ideally, regardless of whether it is Go or Rust. Source of the data is from http handlers, which clearly will be multi-threaded (real or green). The `channel` is serving as the place to have `multiple-producer-single-consumer` (referred to as mpsc in Rust terminology)
    * Using a mutex for i/o would invalidate the `async` advantage provider by `hyper`
    * The code is actually using `mpmc` (multiple consumers) from `crossbeam` crate. Mainly because I wanted to have data split to multiple files. It is unproven multiple parallel writers actually give better performance than a single writer for ssd/nvme type storage. Go version did show improvement with more than one writer. More about these choices later. Just keep in mind that `crossbeam` channel API is not `async`-compatible afaik. `tokio::sync::mpsc` allows only a single writer. Tokio does not have a `mpmc` variant.

 * Performance is only 70% of that of the Go version. It is unclear at this point whether it is
    * fasthttp vs hyper?
    * Use of blocking API (for channel write) from an async handler of hyper. Hoping it is not that bad since write to a buffered channel should be fast?.
    * any performance difference between Rust implementation of [lz4](https://crates.io/crates/lz4)  vs [Go](https://github.com/pierrec/lz4) implementation 
    * general awkwardness of non-idiomatic Rust code written by a Gopher who doesn't know how to properly profile Rust code.
    * Incorrect use of [pool](https://crates.io/crates/object-pool) in a situation that probably doesn't need a pool? Pool usage seems to improve performance, but I have not yet verified it actually reduces allocations.
    * Should channel transfer a `Vec[u8]` of the finished flatbuffer bytes [ No pool, [attempt1.rs](src/attempt1.rs) ] OR should it be a `builder` object taken from a pool, [attempt2.rs](src/attempt2.rs), like the Go implementation.


### State of the code

* `main.rs` is pretty simple - deals just with the creation of a channel, hyper serving endpoint and handlers.
* Two variants `attempt1.rs` and `attempt2.rs` based on which line is uncommented in main.rs at lines 51/52 and lines 83/84.
* Please note that line 44 doesn't define the type of the channel contents. The type changes depending on which `attempt?.rs` is called (via uncommenting). This is because of [fancy type-inference](https://news.ycombinator.com/item?id=15301620) done by Rust based on actual usage several statements or methods later

