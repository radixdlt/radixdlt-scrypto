#![no_main]

use libfuzzer_sys::fuzz_target;

use radix_engine_common::prelude::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = manifest_decode::<ManifestValue>(data) {
        match manifest_encode(&value) {
            Ok(bytes) => assert_eq!(data, bytes.as_slice()),
            e => panic!("{:?}", e),
        }
    }
});
