#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
pub use alloc::borrow;
#[cfg(feature = "alloc")]
pub use alloc::boxed;
#[cfg(feature = "alloc")]
pub use alloc::fmt;
#[cfg(feature = "alloc")]
pub use alloc::format;
#[cfg(feature = "alloc")]
pub use alloc::rc;
#[cfg(feature = "alloc")]
pub use alloc::str;
#[cfg(feature = "alloc")]
pub use alloc::string;
#[cfg(feature = "alloc")]
pub use alloc::sync;
#[cfg(feature = "alloc")]
pub use alloc::vec;
#[cfg(feature = "alloc")]
pub use core::cell;
#[cfg(feature = "alloc")]
pub use core::cmp;
#[cfg(feature = "alloc")]
pub use core::convert;
#[cfg(feature = "alloc")]
pub use core::hash;
#[cfg(feature = "alloc")]
pub use core::iter;
#[cfg(feature = "alloc")]
pub use core::marker;
#[cfg(feature = "alloc")]
pub use core::mem;
#[cfg(feature = "alloc")]
pub use core::num;
#[cfg(feature = "alloc")]
pub use core::ops;
#[cfg(feature = "alloc")]
pub use core::ptr;
#[cfg(feature = "alloc")]
pub use core::slice;

#[cfg(not(feature = "alloc"))]
pub use std::borrow;
#[cfg(not(feature = "alloc"))]
pub use std::boxed;
#[cfg(not(feature = "alloc"))]
pub use std::cell;
#[cfg(not(feature = "alloc"))]
pub use std::cmp;
#[cfg(not(feature = "alloc"))]
pub use std::convert;
#[cfg(not(feature = "alloc"))]
pub use std::fmt;
#[cfg(not(feature = "alloc"))]
pub use std::format;
#[cfg(not(feature = "alloc"))]
pub use std::hash;
#[cfg(not(feature = "alloc"))]
pub use std::iter;
#[cfg(not(feature = "alloc"))]
pub use std::marker;
#[cfg(not(feature = "alloc"))]
pub use std::mem;
#[cfg(not(feature = "alloc"))]
pub use std::num;
#[cfg(not(feature = "alloc"))]
pub use std::ops;
#[cfg(not(feature = "alloc"))]
pub use std::ptr;
#[cfg(not(feature = "alloc"))]
pub use std::rc;
#[cfg(not(feature = "alloc"))]
pub use std::slice;
#[cfg(not(feature = "alloc"))]
pub use std::str;
#[cfg(not(feature = "alloc"))]
pub use std::string;
#[cfg(not(feature = "alloc"))]
pub use std::sync;
#[cfg(not(feature = "alloc"))]
pub use std::vec;

/// Collection types.
pub mod collections {
    #[cfg(feature = "alloc")]
    extern crate alloc;

    pub mod btree_map {
        #[cfg(feature = "alloc")]
        extern crate alloc;
        #[cfg(feature = "alloc")]
        pub use alloc::collections::btree_map::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::btree_map::*;

        #[macro_export]
        macro_rules! btreemap {
            ( $($key:expr => $value:expr),* ) => ({
                let mut temp = ::sbor::rust::collections::BTreeMap::new();
                $(
                    temp.insert($key, $value);
                )*
                temp
            });
            ( $($key:expr => $value:expr,)* ) => (
                btreemap!{$($key => $value),*}
            );
        }

        #[allow(unused_imports)]
        pub(crate) use btreemap; // Allows btreemap to be used inside this crate
    }

    pub mod btree_set {
        #[cfg(feature = "alloc")]
        extern crate alloc;
        #[cfg(feature = "alloc")]
        pub use alloc::collections::btree_set::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::btree_set::*;

        #[macro_export]
        macro_rules! btreeset {
            ( $($value:expr),* ) => ({
                let mut temp = ::sbor::rust::collections::BTreeSet::new();
                $(
                    temp.insert($value);
                )*
                temp
            });
            ( $($value:expr,)* ) => (
                btreeset!{$($value),*}
            );
        }

        #[allow(unused_imports)]
        pub(crate) use btreeset; // Allows btreeset to be used inside this crate
    }

    pub mod hash_map {
        #[cfg(feature = "alloc")]
        pub use hashbrown::hash_map::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::hash_map::*;

        #[macro_export]
        macro_rules! hashmap {
            ( $($key:expr => $value:expr),* ) => ({
                let mut temp = ::sbor::rust::collections::HashMap::new();
                $(
                    temp.insert($key, $value);
                )*
                temp
            });
            ( $($key:expr => $value:expr,)* ) => (
                hashmap!{$($key => $value),*}
            );
        }

        #[allow(unused_imports)]
        pub(crate) use hashmap; // Allows btreeset to be used inside this crate
    }

    pub mod hash_set {
        #[cfg(feature = "alloc")]
        pub use hashbrown::hash_set::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::hash_set::*;

        #[macro_export]
        macro_rules! hashset {
            ( $($key:expr),* ) => ({
                let mut temp = ::sbor::rust::collections::HashSet::new();
                $(
                    temp.insert($key);
                )*
                temp
            });
            ( $($key:expr,)* ) => (
                hashset!{$($key),*}
            );
        }

        #[allow(unused_imports)]
        pub(crate) use hashset; // Allows hashset to be used inside this crate
    }

    #[cfg(feature = "indexmap")]
    pub use indexmap;

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

    #[cfg(feature = "indexmap")]
    pub use indexmap::IndexMap;
    #[cfg(feature = "indexmap")]
    pub use indexmap::IndexSet;
}
