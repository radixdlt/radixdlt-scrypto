#[cfg(not(feature = "alloc"))]
mod cargo;
mod panic;
mod slice;

#[cfg(not(feature = "alloc"))]
pub use cargo::compile_package;
pub use panic::set_up_panic_hook;
pub use slice::{combine, copy_u8_array};
