#![cfg_attr(not(feature = "std"), no_std)]
#![no_std]

use sbor::rust::collections::*;

#[test]
pub fn btreemap_macros_compile() {
    assert_eq!(btreemap!(1u32 => "entry_one", 5u32 => "entry_two").len(), 2);
    assert_eq!(
        btreemap!(1u32 => "entry_one", 5u32 => "entry_two",).len(),
        2
    );
}

#[test]
pub fn btreeset_macros_compile() {
    assert_eq!(btreeset!(1u32, 5u32).len(), 2);
    assert_eq!(btreeset!(1u32, 5u32,).len(), 2);
}

#[test]
pub fn hashmap_macros_compile() {
    assert_eq!(hashmap!(1u32 => "entry_one", 5u32 => "entry_two").len(), 2);
    assert_eq!(hashmap!(1u32 => "entry_one", 5u32 => "entry_two",).len(), 2);
}

#[test]
pub fn hashset_macros_compile() {
    assert_eq!(hashset!(1u32, 5u32).len(), 2);
    assert_eq!(hashset!(1u32, 5u32,).len(), 2);
}

/// Also tests that IndexMap compiles with no_std etc
#[test]
pub fn index_map_creations_compile() {
    type K = u32;
    type V = u32;
    let n: usize = 4;

    let _: IndexMap<K, V> = index_map_new();
    let _: IndexMap<K, V> = IndexMap::default();
    let _: IndexMap<K, V> = index_map_with_capacity(n);
    let _ = indexmap!(1u32 => "entry_one", 5u32 => "entry_two");
    let _ = indexmap!(1u32 => "entry_one", 5u32 => "entry_two",);
    let _: IndexMap<_, _> = [(1u32, "entry_one")].into_iter().collect();

    // Note - these don't work with no_std on:
    // let _: IndexMap<K, V> = IndexMap::new();
    // let _: IndexMap<K, V> = IndexMap::with_capacity(n);
}

/// Tests that IndexSet compiles with no_std etc
#[test]
pub fn index_set_creations_compile() {
    type K = u32;
    let n: usize = 4;

    let _: IndexSet<K> = index_set_new();
    let _: IndexSet<K> = IndexSet::default();
    let _: IndexSet<K> = index_set_with_capacity(n);
    let _ = indexset!(1u32, 5u32);
    let _ = indexset!(1u32, 5u32,);
    let _: IndexSet<_> = [1u32, 2u32].into_iter().collect();

    // Note - these don't work with no_std on:
    // let _: IndexMap<K, V> = IndexMap::new();
    // let _: IndexMap<K, V> = IndexMap::with_capacity(n);
}
