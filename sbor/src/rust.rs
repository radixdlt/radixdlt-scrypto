#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub use alloc::borrow;
#[cfg(feature = "alloc")]
pub use alloc::boxed;
#[cfg(feature = "alloc")]
pub use alloc::string;
#[cfg(feature = "alloc")]
pub use alloc::vec;
#[cfg(feature = "alloc")]
pub use core::convert;
#[cfg(feature = "alloc")]
pub use core::hash;
#[cfg(feature = "alloc")]
pub use core::mem;
#[cfg(feature = "alloc")]
pub use core::ptr;

#[cfg(not(feature = "alloc"))]
pub use std::borrow;
#[cfg(not(feature = "alloc"))]
pub use std::boxed;
#[cfg(not(feature = "alloc"))]
pub use std::convert;
#[cfg(not(feature = "alloc"))]
pub use std::hash;
#[cfg(not(feature = "alloc"))]
pub use std::mem;
#[cfg(not(feature = "alloc"))]
pub use std::ptr;
#[cfg(not(feature = "alloc"))]
pub use std::string;
#[cfg(not(feature = "alloc"))]
pub use std::vec;

/// Rust's standard collection library.
pub mod collections {
    #[cfg(feature = "alloc")]
    extern crate alloc;

    #[cfg(feature = "alloc")]
    pub use alloc::collections::BTreeMap;
    #[cfg(feature = "alloc")]
    pub use alloc::collections::BTreeSet;
    #[cfg(feature = "alloc")]
    pub use hashbrown::HashMap;
    #[cfg(feature = "alloc")]
    pub use hashbrown::HashSet;

    #[cfg(not(feature = "alloc"))]
    pub use std::collections::BTreeMap;
    #[cfg(not(feature = "alloc"))]
    pub use std::collections::BTreeSet;
    #[cfg(not(feature = "alloc"))]
    pub use std::collections::HashMap;
    #[cfg(not(feature = "alloc"))]
    pub use std::collections::HashSet;
}
