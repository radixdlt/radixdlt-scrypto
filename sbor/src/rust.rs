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

        #[cfg(feature = "alloc")]
        pub use alloc::collections::BTreeMap;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::BTreeMap;

        #[macro_export]
        macro_rules! btreemap {
            ( $($key:expr => $value:expr),* ) => ({
                let mut temp = $crate::rust::collections::btree_map::BTreeMap::new();
                $(
                    temp.insert($key, $value);
                )*
                temp
            });
            ( $($key:expr => $value:expr,)* ) => (
                $crate::rust::collections::btree_map::btreemap!{$($key => $value),*}
            );
        }

        pub use btreemap;
    }

    pub mod btree_set {
        #[cfg(feature = "alloc")]
        extern crate alloc;
        #[cfg(feature = "alloc")]
        pub use alloc::collections::btree_set::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::btree_set::*;

        #[cfg(feature = "alloc")]
        pub use alloc::collections::BTreeSet;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::BTreeSet;

        #[macro_export]
        macro_rules! btreeset {
            ( $($value:expr),* ) => ({
                let mut temp = $crate::rust::collections::btree_set::BTreeSet::new();
                $(
                    temp.insert($value);
                )*
                temp
            });
            ( $($value:expr,)* ) => (
                $crate::rust::collections::btree_set::btreeset!{$($value),*}
            );
        }

        pub use btreeset;
    }

    pub mod hash_map {
        #[cfg(feature = "alloc")]
        pub use hashbrown::hash_map::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::hash_map::*;

        #[cfg(feature = "alloc")]
        pub use hashbrown::HashMap;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::HashMap;

        #[macro_export]
        macro_rules! hashmap {
            ( $($key:expr => $value:expr),* ) => ({
                // Note: `stringify!($key)` is just here to consume the repetition,
                // but we throw away that string literal during constant evaluation.
                const CAP: usize = <[()]>::len(&[$({ stringify!($key); }),*]);
                let mut temp = $crate::rust::collections::hash_map::HashMap::with_capacity(CAP);
                $(
                    temp.insert($key, $value);
                )*
                temp
            });
            ( $($key:expr => $value:expr,)* ) => (
                $crate::rust::collections::hash_map::hashmap!{$($key => $value),*}
            );
        }

        pub use hashmap;
    }

    pub mod hash_set {
        #[cfg(feature = "alloc")]
        pub use hashbrown::hash_set::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::hash_set::*;

        #[cfg(feature = "alloc")]
        pub use hashbrown::HashSet;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::HashSet;

        #[macro_export]
        macro_rules! hashset {
            ( $($key:expr),* ) => ({
                // Note: `stringify!($key)` is just here to consume the repetition,
                // but we throw away that string literal during constant evaluation.
                const CAP: usize = <[()]>::len(&[$({ stringify!($key); }),*]);
                let mut temp = $crate::rust::collections::hash_set::HashSet::with_capacity(CAP);
                $(
                    temp.insert($key);
                )*
                temp
            });
            ( $($key:expr,)* ) => (
                $crate::rust::collections::hash_set::hashset!{$($key),*}
            );
        }

        pub use hashset;
    }

    #[cfg(feature = "indexmap")]
    /// The methods and macros provided directly in this `index_map` module (`new`, `with_capacity`) work in both std and no-std modes - unlike the
    /// corresponding methods on `IndexMap` itself.
    ///
    /// Unfortunately `IndexMap` is very hard to use from no-std (see [docs](https://docs.rs/indexmap/latest/indexmap/#no-standard-library-targets)
    /// and [relevant github issue](https://github.com/bluss/indexmap/issues/184)). It uses a weird build flag to detect if no-std is present, which
    /// is hard to force unless you explicitly do eg a WASM build and see that it's missing.
    ///
    /// The recommended way to use IndexMap is to add `use sbor::rust::collections::*` and then reference the type inline as `index_map::IndexMap`
    /// and create new sets using `index_map::new`, `index_map::with_capacity`, or the `index_map::indexmap!` macro. Always putting the `index_map`
    /// mod will help enforce the use of these methods instead of the methods on `IndexMap` itself.
    ///
    /// You can use these exports as follows:
    /// ```
    /// use sbor::rust::collections::*;
    ///
    /// # type K = u32;
    /// # type V = u32;
    /// # let n: usize = 1;
    /// let index_map: IndexMap<K, V> = index_map_new();
    /// let index_map: IndexMap<K, V> = index_map_with_capacity(n);
    /// let index_map = indexmap!(1u32 => "entry_one", 5u32 => "entry_two");
    /// ```
    pub mod index_map {
        #[cfg(feature = "alloc")]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(not(feature = "alloc"))]
        pub type DefaultHashBuilder = std::collections::hash_map::RandomState;

        // See https://github.com/bluss/indexmap/pull/207
        // By defining an alias with a default `DefaultHashBuilder`, we ensure that this type works as `IndexMap<K, V>` and that the `FromIter` impl works in no-std.
        pub type IndexMap<K, V, S = DefaultHashBuilder> = indexmap::IndexMap<K, V, S>;

        /// This is safe for std and no-std use cases (unlike `IndexMap::new` which disappears when std is not in the toolchain - see
        /// [this article](https://faultlore.com/blah/defaults-affect-inference/) for deep technical reasons)
        pub fn new<K, V>() -> IndexMap<K, V> {
            IndexMap::with_capacity_and_hasher(0, DefaultHashBuilder::default())
        }

        /// This is safe for std and no-std use cases (unlike `IndexMap::with_capacity` which disappears when std is not in the toolchain - see
        /// [this article](https://faultlore.com/blah/defaults-affect-inference/) for deep technical reasons)
        pub fn with_capacity<K, V>(n: usize) -> IndexMap<K, V> {
            IndexMap::with_capacity_and_hasher(n, DefaultHashBuilder::default())
        }

        #[macro_export]
        macro_rules! indexmap {
            ($($key:expr => $value:expr,)+) => ( $crate::rust::collections::index_map::indexmap!{$($key => $value),*} );
            ($($key:expr => $value:expr),*) => ({
                // Note: `stringify!($key)` is just here to consume the repetition,
                // but we throw away that string literal during constant evaluation.
                const CAP: usize = <[()]>::len(&[$({ stringify!($key); }),*]);
                let mut temp = $crate::rust::collections::index_map::with_capacity(CAP);
                $(
                    temp.insert($key, $value);
                )*
                temp
            });
        }

        pub use indexmap;
    }

    #[cfg(feature = "indexmap")]
    /// The methods and macros provided directly in this `index_set` module (`new`, `with_capacity`) work in both std and no-std modes - unlike the
    /// corresponding methods on `IndexSet` itself.
    ///
    /// Unfortunately `IndexSet` is very hard to use from no-std (see [docs](https://docs.rs/indexmap/latest/indexmap/#no-standard-library-targets)
    /// and [relevant github issue](https://github.com/bluss/indexmap/issues/184)). It uses a weird build.rs script to detect if no-std is present, which
    /// is hard to force unless you explicitly do eg a WASM build and see that it's missing.
    ///
    /// You can use these methods as follows:
    /// ```
    /// use sbor::rust::collections::*;
    ///
    /// # type K = u32;
    /// # let n: usize = 1;
    /// let index_set: IndexSet<K> = index_set_new();
    /// let index_set: IndexSet<K> = index_set_with_capacity(n);
    /// let index_set = indexset!(1u32, 2u32);
    /// ```
    pub mod index_set {
        #[cfg(feature = "alloc")]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(not(feature = "alloc"))]
        pub type DefaultHashBuilder = std::collections::hash_map::RandomState;

        // See https://github.com/bluss/indexmap/pull/207
        // By defining an alias with a default `DefaultHashBuilder`, we ensure that this type works as `IndexSet<K>` and that the `FromIter` impl works in no-std.
        pub type IndexSet<K, S = DefaultHashBuilder> = indexmap::IndexSet<K, S>;

        /// This is safe for std and no-std use cases (unlike `IndexSet::new` which disappears when std is not in the toolchain - see
        /// [this article](https://faultlore.com/blah/defaults-affect-inference/) for deep technical reasons)
        pub fn new<K>() -> IndexSet<K, DefaultHashBuilder> {
            IndexSet::with_capacity_and_hasher(0, DefaultHashBuilder::default())
        }

        /// This is safe for std and no-std use cases (unlike `IndexSet::with_capacity` which disappears when std is not in the toolchain - see
        /// [this article](https://faultlore.com/blah/defaults-affect-inference/) for deep technical reasons)
        pub fn with_capacity<K>(n: usize) -> IndexSet<K, DefaultHashBuilder> {
            IndexSet::with_capacity_and_hasher(n, DefaultHashBuilder::default())
        }

        #[macro_export]
        macro_rules! indexset {
            ($($key:expr,)+) => ( $crate::rust::collections::index_set::indexset!{$($key),*} );
            ($($key:expr),*) => ({
                // Note: `stringify!($key)` is just here to consume the repetition,
                // but we throw away that string literal during constant evaluation.
                const CAP: usize = <[()]>::len(&[$({ stringify!($key); }),*]);
                let mut temp = $crate::rust::collections::index_set::with_capacity(CAP);
                $(
                    temp.insert($key);
                )*
                temp
            });
        }

        pub use indexset;
    }

    pub use btree_map::btreemap;
    pub use btree_map::BTreeMap;
    pub use btree_set::btreeset;
    pub use btree_set::BTreeSet;
    pub use hash_map::hashmap;
    pub use hash_map::HashMap;
    pub use hash_set::hashset;
    pub use hash_set::HashSet;
    #[cfg(feature = "indexmap")]
    pub use index_map::indexmap;
    #[cfg(feature = "indexmap")]
    pub use index_map::IndexMap;
    #[cfg(feature = "indexmap")]
    pub use index_map::{new as index_map_new, with_capacity as index_map_with_capacity};
    #[cfg(feature = "indexmap")]
    pub use index_set::indexset;
    #[cfg(feature = "indexmap")]
    pub use index_set::IndexSet;
    #[cfg(feature = "indexmap")]
    pub use index_set::{new as index_set_new, with_capacity as index_set_with_capacity};
}
