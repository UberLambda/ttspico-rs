# ttspico-rs
Rust bindings for Pico TTS, an open source ([Apache 2.0](LICENSE)) text-to-speech engine.

## Crates in this repo
- [`ttspico_sys`](ttspico-sys/): Low-level (C FFI) Rust bindings to Pico.  
  Compiles Pico (patched for 64-bit compatibility) from source and links to it statically.
- [`ttspico`](ttspico/): High-level, idiomatic Rust bindings to Pico.  
  Built on top of `ttspico_sys`.

## Getting started
See [ttspico/examples/hello.rs](ttspico/examples/hello.rs).

## Platforms
Pico was [originally part of Android](https://android.googlesource.com/platform/external/svox/+/refs/heads/master/pico/),
but it is written in portable C99 and works great on many other operating systems and platforms.
A few small modifications to its source code ([ttspico-sys/build/pico/lib/](ttspico-sys/build/pico/lib/)) were made to make it work on 64-bit platforms.

## License
Both Pico and the Rust bindings are licensed under the [Apache 2.0 license](LICENSE).
