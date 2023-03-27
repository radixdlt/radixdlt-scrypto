#[cfg(all(target_family = "unix", feature = "resource_tracker"))]
pub mod qemu_plugin_interface;

#[cfg(all(target_family = "unix", feature = "resource_tracker"))]
pub use qemu_plugin_interface::*;
