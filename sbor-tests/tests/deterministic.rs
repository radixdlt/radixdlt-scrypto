#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::collections::{hash_map_new, hash_set_new};
use sbor::rust::vec::Vec;
use sbor::*;

fn encode_new_hash_set(forward: bool) -> Vec<u8> {
    let mut set = hash_set_new();
    if forward {
        for i in 0u32..100u32 {
            set.insert(i);
        }
    } else {
        for i in (0u32..100u32).rev() {
            set.insert(i);
        }
    }

    basic_encode(&set).unwrap()
}

#[test]
fn encoding_of_hash_set_should_be_deterministic() {
    let encoded0 = encode_new_hash_set(true);
    let encoded1 = encode_new_hash_set(false);
    assert_eq!(encoded0, encoded1);
}

fn encode_new_hash_map(forward: bool) -> Vec<u8> {
    let mut set = hash_map_new();
    if forward {
        for i in 0u32..100u32 {
            set.insert(i, i);
        }
    } else {
        for i in (0u32..100u32).rev() {
            set.insert(i, i);
        }
    }

    basic_encode(&set).unwrap()
}

#[test]
fn encoding_of_hash_map_should_be_deterministic() {
    let encoded0 = encode_new_hash_map(true);
    let encoded1 = encode_new_hash_map(false);
    assert_eq!(encoded0, encoded1);
}
