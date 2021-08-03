// There is no main function in Scrypto.
#![no_main]
// Disable linking to std.
#![no_std]
// Use default alloc error handler, i.e. to panic, and enable core intrinsics.
#![feature(default_alloc_error_handler, core_intrinsics)]

// Abort when panicking.
#[panic_handler]
pub fn panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

// Use WeeAlloc as our global heap allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//============================
// Scrypto code starts here //
//============================

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

use scrypto::*;

component! {
    struct Greeting {
        counter: u32
    }

    impl Greeting {
        pub fn new() -> Self {
            Self {
                counter: 0
            }
        }

        pub fn say_hello(&mut self) -> String {
            self.counter += 1;
            "hello".to_string()
        }
    }
}
