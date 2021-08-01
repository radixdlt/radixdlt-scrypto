#![feature(test)]
extern crate test;
use test::Bencher;

use sbor::{Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};

mod bincode;
use bincode::{bincode_encode, bincode_decode};

#[derive(Encode, Decode, Serialize, Deserialize)]
pub enum TestEnum {
    A,
    B(u32),
    C { x: u32, y: u32 },
}

#[derive(Encode, Decode, Serialize, Deserialize)]
pub struct TestStruct {
    number: u64,
    string: String,
    vector: Vec<u8>,
    enumeration: TestEnum,
}

#[bench]
fn bench_encode_sbor(b: &mut Bencher) {
    let t = TestStruct {
        number: 0x12345678ABCDEF00,
        string: "A totally pointless string".to_owned(),
        vector: vec![1, 2, 3],
        enumeration: TestEnum::C { x: 01234, y: 66789 },
    };

    b.iter(|| {
        let mut enc = Encoder::new();
        t.encode(&mut enc);
        let bytes: Vec<u8> = enc.into();
        bytes
    });
}

#[bench]
fn bench_encode_json(b: &mut Bencher) {
    let t = TestStruct {
        number: 0x12345678ABCDEF00,
        string: "A totally pointless string".to_owned(),
        vector: vec![1, 2, 3],
        enumeration: TestEnum::C { x: 01234, y: 66789 },
    };

    b.iter(|| {
        serde_json::to_vec(&t)
    });
}

#[bench]
fn bench_encode_bincode(b: &mut Bencher) {
    let t = TestStruct {
        number: 0x12345678ABCDEF00,
        string: "A totally pointless string".to_owned(),
        vector: vec![1, 2, 3],
        enumeration: TestEnum::C { x: 01234, y: 66789 },
    };

    b.iter(|| {
        bincode_encode(&t)
    });
}
