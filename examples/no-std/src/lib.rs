// Disable std and enable language features.
#![cfg_attr(target_arch = "wasm32", no_std)]

// Abort when panicking.
#[cfg(target_arch = "wasm32")]
#[panic_handler]
pub fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Use WeeAlloc as our global heap allocator.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//============================
// Scrypto code starts here //
//============================

use scrypto::prelude::*;

#[blueprint]
mod no_std {
    struct NoStd;

    impl NoStd {
        pub fn say_hello() {
            info!("Hello, I'm running with no_std!");
        }
    }
}
