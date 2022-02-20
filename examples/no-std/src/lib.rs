// Disable linking to std and enable language features.
#![cfg_attr(
    target_arch = "wasm32",
    no_std,
    feature(default_alloc_error_handler, core_intrinsics)
)]

// Abort when panicking.
#[cfg(target_arch = "wasm32")]
#[panic_handler]
pub fn panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

// Use WeeAlloc as our global heap allocator.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//============================
// Scrypto code starts here //
//============================

use scrypto::prelude::*;

blueprint! {
    struct NoStd;

    impl NoStd {
        pub fn say_hello() {
            info!("Hello, I'm running with no_std!");
        }
    }
}
