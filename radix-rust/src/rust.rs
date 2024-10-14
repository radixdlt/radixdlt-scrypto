pub mod prelude {
    // See eg https://doc.rust-lang.org/std/prelude/index.html

    // std::prelude::v1
    pub use super::borrow::ToOwned;
    pub use super::boxed::Box;
    pub use super::cell::RefCell;
    pub use super::clone::Clone;
    pub use super::cmp::{Eq, Ord, PartialEq, PartialOrd};
    pub use super::convert::{AsMut, AsRef, From, Into};
    pub use super::default::Default;
    pub use super::iter::{DoubleEndedIterator, ExactSizeIterator, Extend, IntoIterator, Iterator};
    pub use super::marker::{Copy, Send, Sized, Sync, Unpin};
    pub use super::mem::drop;
    pub use super::ops::{Drop, Fn, FnMut, FnOnce};
    pub use super::option::Option::{self, None, Some};
    pub use super::result::Result::{self, Err, Ok};
    pub use super::string::{String, ToString};
    pub use super::vec::Vec;

    // std::prelude::rust_2021
    pub use super::convert::{TryFrom, TryInto};
    pub use super::iter::FromIterator;

    // And some extra useful additions we use a lot:
    pub use super::borrow;
    pub use super::borrow::Cow;
    pub use super::cell::*;
    pub use super::collections::*;
    pub use super::fmt;
    pub use super::fmt::{Debug, Display};
    pub use super::format;
    pub use super::marker::PhantomData;
    pub use super::mem;
    pub use super::rc::Rc;
    pub use super::str::FromStr;
    pub use super::vec;
}

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
pub use core::clone;
#[cfg(feature = "alloc")]
pub use core::cmp;
#[cfg(feature = "alloc")]
pub use core::convert;
#[cfg(feature = "alloc")]
pub use core::default;
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
pub use core::option;
#[cfg(feature = "alloc")]
pub use core::ptr;
#[cfg(feature = "alloc")]
pub use core::result;
#[cfg(feature = "alloc")]
pub use core::slice;

#[cfg(not(feature = "alloc"))]
pub use core::hash;
#[cfg(not(feature = "alloc"))]
pub use std::alloc;
#[cfg(not(feature = "alloc"))]
pub use std::borrow;
#[cfg(not(feature = "alloc"))]
pub use std::boxed;
#[cfg(not(feature = "alloc"))]
pub use std::cell;
#[cfg(not(feature = "alloc"))]
pub use std::clone;
#[cfg(not(feature = "alloc"))]
pub use std::cmp;
#[cfg(not(feature = "alloc"))]
pub use std::convert;
#[cfg(not(feature = "alloc"))]
pub use std::default;
#[cfg(not(feature = "alloc"))]
pub use std::fmt;
#[cfg(not(feature = "alloc"))]
pub use std::format;
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
pub use std::option;
#[cfg(not(feature = "alloc"))]
pub use std::ptr;
#[cfg(not(feature = "alloc"))]
pub use std::rc;
#[cfg(not(feature = "alloc"))]
pub use std::result;
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

    #[cfg(feature = "alloc")]
    pub use alloc::collections::LinkedList;
    #[cfg(not(feature = "alloc"))]
    pub use std::collections::LinkedList;

    #[cfg(feature = "alloc")]
    pub use alloc::collections::VecDeque;
    #[cfg(not(feature = "alloc"))]
    pub use std::collections::VecDeque;

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
            ( ) => ({
                $crate::rust::collections::btree_map::BTreeMap::new()
            });
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
            ( ) => ({
                $crate::rust::collections::btree_set::BTreeSet::new()
            });
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

    /// This is a stub implementation for Hasher (used by `IndexMap`, `IndexSet`) to get rid of non-deterministic output (caused by random seeding the hashes).
    /// This is useful when fuzz testing, where exactly the same output is expected for the same input data across different runs.
    #[cfg(feature = "fuzzing")]
    pub mod stub_hasher {
        use core::hash::{BuildHasher, Hasher};

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct StubHasher {
            seed: u64,
        }

        impl Hasher for StubHasher {
            fn write(&mut self, _bytes: &[u8]) {}

            fn finish(&self) -> u64 {
                self.seed
            }
        }

        impl BuildHasher for StubHasher {
            type Hasher = StubHasher;

            fn build_hasher(&self) -> Self::Hasher {
                StubHasher { seed: self.seed }
            }
        }

        impl StubHasher {
            fn new() -> Self {
                StubHasher { seed: 0 }
            }
        }

        impl Default for StubHasher {
            fn default() -> Self {
                StubHasher::new()
            }
        }
    }

    pub mod hash_map {
        #[cfg(feature = "fuzzing")]
        pub type DefaultHashBuilder = crate::rust::collections::stub_hasher::StubHasher;
        #[cfg(all(not(feature = "fuzzing"), feature = "alloc"))]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(all(not(feature = "fuzzing"), not(feature = "alloc")))]
        pub type DefaultHashBuilder = fxhash::FxBuildHasher;

        #[cfg(feature = "alloc")]
        pub use hashbrown::hash_map::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::hash_map::*;

        #[cfg(not(feature = "alloc"))]
        pub use fxhash::FxHashMap as ext_HashMap;
        #[cfg(feature = "alloc")]
        pub use hashbrown::HashMap as ext_HashMap;

        pub type HashMap<K, V> = ext_HashMap<K, V>;

        /// Creates an empty map with capacity 0 and default Hasher
        pub fn new<K, V>() -> HashMap<K, V> {
            HashMap::with_capacity_and_hasher(0, DefaultHashBuilder::default())
        }

        /// Creates an empty map with given capacity and default Hasher
        pub fn with_capacity<K, V>(n: usize) -> HashMap<K, V> {
            HashMap::with_capacity_and_hasher(n, DefaultHashBuilder::default())
        }

        #[macro_export]
        macro_rules! hashmap {
            ( ) => ({
                $crate::rust::collections::hash_map::HashMap::default()
            });
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
        #[cfg(feature = "fuzzing")]
        pub type DefaultHashBuilder = crate::rust::collections::stub_hasher::StubHasher;
        #[cfg(all(not(feature = "fuzzing"), feature = "alloc"))]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(all(not(feature = "fuzzing"), not(feature = "alloc")))]
        pub type DefaultHashBuilder = fxhash::FxBuildHasher;

        #[cfg(feature = "alloc")]
        pub use hashbrown::hash_set::*;
        #[cfg(not(feature = "alloc"))]
        pub use std::collections::hash_set::*;

        #[cfg(not(feature = "alloc"))]
        pub use fxhash::FxHashSet as ext_HashSet;
        #[cfg(feature = "alloc")]
        pub use hashbrown::HashSet as ext_HashSet;

        pub type HashSet<V> = ext_HashSet<V>;

        /// Creates an empty set with capacity 0 and default Hasher
        pub fn new<K>() -> HashSet<K> {
            HashSet::with_capacity_and_hasher(0, DefaultHashBuilder::default())
        }

        /// Creates an empty set with given capacity and default Hasher
        pub fn with_capacity<K>(n: usize) -> HashSet<K> {
            HashSet::with_capacity_and_hasher(n, DefaultHashBuilder::default())
        }

        #[macro_export]
        macro_rules! hashset {
            ( ) => ({
                $crate::rust::collections::hash_set::HashSet::new()
            });
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

    /// The methods and macros provided directly in this `index_map` module (`new`, `with_capacity`) work in both std and no-std modes - unlike the
    /// corresponding methods on `IndexMap` itself.
    ///
    /// Unfortunately `IndexMap` is very hard to use from no-std (see [docs](https://docs.rs/indexmap/latest/indexmap/#no-standard-library-targets)
    /// and [relevant github issue](https://github.com/bluss/indexmap/issues/184)). It uses a weird build flag to detect if no-std is present, which
    /// is hard to force unless you explicitly do eg a WASM build and see that it's missing.
    ///
    /// The recommended way to use IndexMap is to add `use radix_rust::prelude::*` and then reference the type inline as `index_map::IndexMap`
    /// and create new sets using `index_map::new`, `index_map::with_capacity`, or the `index_map::indexmap!` macro. Always putting the `index_map`
    /// mod will help enforce the use of these methods instead of the methods on `IndexMap` itself.
    ///
    /// You can use these exports as follows:
    /// ```
    /// use radix_rust::rust::collections::*;
    ///
    /// # type K = u32;
    /// # type V = u32;
    /// # let n: usize = 1;
    /// let index_map: IndexMap<K, V> = index_map_new();
    /// let index_map: IndexMap<K, V> = index_map_with_capacity(n);
    /// let index_map = indexmap!(1u32 => "entry_one", 5u32 => "entry_two");
    /// ```
    pub mod index_map {
        #[cfg(feature = "fuzzing")]
        pub type DefaultHashBuilder = crate::rust::collections::stub_hasher::StubHasher;
        #[cfg(all(not(feature = "fuzzing"), feature = "alloc"))]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(all(not(feature = "fuzzing"), not(feature = "alloc")))]
        pub type DefaultHashBuilder = fxhash::FxBuildHasher;

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
            ( ) => ({
                $crate::rust::collections::index_map_new()
            });
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

    /// The methods and macros provided directly in this `index_set` module (`new`, `with_capacity`) work in both std and no-std modes - unlike the
    /// corresponding methods on `IndexSet` itself.
    ///
    /// Unfortunately `IndexSet` is very hard to use from no-std (see [docs](https://docs.rs/indexmap/latest/indexmap/#no-standard-library-targets)
    /// and [relevant github issue](https://github.com/bluss/indexmap/issues/184)). It uses a weird build.rs script to detect if no-std is present, which
    /// is hard to force unless you explicitly do eg a WASM build and see that it's missing.
    ///
    /// You can use these methods as follows:
    /// ```
    /// use radix_rust::rust::collections::*;
    ///
    /// # type K = u32;
    /// # let n: usize = 1;
    /// let index_set: IndexSet<K> = index_set_new();
    /// let index_set: IndexSet<K> = index_set_with_capacity(n);
    /// let index_set = indexset!(1u32, 2u32);
    /// ```
    pub mod index_set {
        #[cfg(feature = "fuzzing")]
        pub type DefaultHashBuilder = crate::rust::collections::stub_hasher::StubHasher;
        #[cfg(all(not(feature = "fuzzing"), feature = "alloc"))]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(all(not(feature = "fuzzing"), not(feature = "alloc")))]
        pub type DefaultHashBuilder = fxhash::FxBuildHasher;

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
            () => ({
                $crate::rust::collections::index_set_new()
            });
            ($($key:expr),+$(,)?) => ({
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

    pub mod non_iter_map {

        #[cfg(feature = "alloc")]
        use hashbrown::HashMap;
        #[cfg(not(feature = "alloc"))]
        use std::collections::HashMap;

        #[cfg(feature = "alloc")]
        use core::hash::Hash;
        #[cfg(not(feature = "alloc"))]
        use core::hash::Hash;

        #[cfg(feature = "alloc")]
        use core::borrow::Borrow;
        #[cfg(not(feature = "alloc"))]
        use std::borrow::Borrow;

        #[cfg(feature = "fuzzing")]
        pub type DefaultHashBuilder = crate::rust::collections::stub_hasher::StubHasher;
        #[cfg(all(not(feature = "fuzzing"), feature = "alloc"))]
        pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;
        #[cfg(all(not(feature = "fuzzing"), not(feature = "alloc")))]
        pub type DefaultHashBuilder = fxhash::FxBuildHasher;

        /// A thin wrapper around a `HashMap`, which guarantees that a `HashMap` usage will not
        /// result in a non-deterministic execution (simply by disallowing the iteration over its
        /// elements).
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct NonIterMap<K: Eq + Hash, V, S: core::hash::BuildHasher = DefaultHashBuilder>(
            HashMap<K, V, S>,
        );

        #[cfg(feature = "alloc")]
        pub type Entry<'a, K, V> = hashbrown::hash_map::Entry<'a, K, V, DefaultHashBuilder>;
        #[cfg(not(feature = "alloc"))]
        pub type Entry<'a, K, V> = std::collections::hash_map::Entry<'a, K, V>;

        impl<K: Hash + Eq, V> NonIterMap<K, V> {
            /// Creates an empty map.
            pub fn new() -> Self {
                Self(HashMap::with_capacity_and_hasher(
                    0,
                    DefaultHashBuilder::default(),
                ))
            }

            /// Gets the given key's corresponding entry in the map for in-place manipulation.
            pub fn entry(&mut self, key: K) -> Entry<K, V> {
                self.0.entry(key)
            }

            /// Inserts a key-value pair into the map.
            /// If the map did not have this key present, None is returned.
            /// If the map did have this key present, the value is updated, and the old value is
            /// returned.
            pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                self.0.insert(key, value)
            }

            /// Returns a reference to the value corresponding to the key.
            pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
            where
                Q: Hash + Eq,
                K: Borrow<Q>,
            {
                self.0.get(key)
            }

            /// Returns a mutable reference to the value corresponding to the key.
            pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
            where
                Q: Hash + Eq,
                K: Borrow<Q>,
            {
                self.0.get_mut(key)
            }

            /// Returns true if the map contains a value for the specified key.
            pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
            where
                Q: Hash + Eq,
                K: Borrow<Q>,
            {
                self.0.contains_key(key)
            }

            /// Removes a key from the map, returning the value at the key if the key was previously
            /// in the map.
            pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
            where
                Q: Hash + Eq,
                K: Borrow<Q>,
            {
                self.0.remove(key)
            }

            /// Clears the map, removing all key-value pairs.
            pub fn clear(&mut self) {
                self.0.clear();
            }

            /// Returns the number of elements in the map.
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Returns whether the map is empty
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl<K: Hash + Eq, V> Default for NonIterMap<K, V> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<K: Hash + Eq, V> FromIterator<(K, V)> for NonIterMap<K, V> {
            fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
                Self(HashMap::from_iter(iter))
            }
        }
    }

    pub use btree_map::btreemap;
    pub use btree_map::BTreeMap;
    pub use btree_set::btreeset;
    pub use btree_set::BTreeSet;
    pub use hash_map::hashmap;
    pub use hash_map::HashMap;
    pub use hash_map::{new as hash_map_new, with_capacity as hash_map_with_capacity};
    pub use hash_set::hashset;
    pub use hash_set::HashSet;
    pub use hash_set::{new as hash_set_new, with_capacity as hash_set_with_capacity};
    pub use index_map::indexmap;
    pub use index_map::IndexMap;
    pub use index_map::{new as index_map_new, with_capacity as index_map_with_capacity};
    pub use index_set::indexset;
    pub use index_set::IndexSet;
    pub use index_set::{new as index_set_new, with_capacity as index_set_with_capacity};
    pub use non_iter_map::NonIterMap;
}
