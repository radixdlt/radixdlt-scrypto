# No Standard Library

## Why `no_std`? 

Disabling link to Rust standard library forces developers to write code against bare metal environment, which happen to be the case for most blockchain platform. By doing so, no extra code from std is imported and many optimization can be applied to make the code smaller and more efficient.

## How?

Most of Scrypto libraries support `no_std`, using `core + alloc` instead.

To add `no_std` support, you'll need to disable the default feature set and turn on `alloc` in the cargo configuration file `Cargo.toml`, like

```
[dependencies]
sbor = { path = "../../../sbor", default-features = false, features = ["alloc"] }
scrypto = { path = "../../../scrypto", default-features = false, features = ["alloc"] }
```

In addition, we need to provide a memory allocator. You can use the WebAssembly-optimized allocator like `WeeAlloc`.
```
wee_alloc = { version = "0.4", default-features = false }
```

Finally, implement the language dependencies required by Rust.

```rust
// Disable linking to std.
#![cfg_attr(not(test), no_std)]
// Use default alloc error handler, i.e. to panic, and enable core intrinsics.
#![cfg_attr(not(test), feature(default_alloc_error_handler, core_intrinsics))]

// Abort when panicking.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

// Use WeeAlloc as our global heap allocator.
#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```