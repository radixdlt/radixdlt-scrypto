#![feature(test)]
extern crate test;
use test::Bencher;

use sbor::{sbor_decode, sbor_encode, Decode, Encode};
use serde::{Deserialize, Serialize};

mod helper;
use helper::{bincode_decode, bincode_encode, json_decode, json_encode};

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

fn make_struct(str_len: usize, vec_len: usize) -> TestStruct {
    TestStruct {
        number: 1234,
        string: "t".repeat(str_len).to_owned(),
        vector: vec![1u8; vec_len],
        enumeration: TestEnum::C { x: 1234, y: 5678 },
    }
}

macro_rules! encode {
    ($name:ident, $str_len:expr, $vec_len:expr, $enc:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let t = make_struct($str_len, $vec_len);
            b.iter(|| $enc(&t));
        }
    };
}

encode!(enc_small_sbor, 10, 10, sbor_encode);
encode!(enc_median_sbor, 100, 100, sbor_encode);
encode!(enc_large_sbor, 1000, 1000, sbor_encode);

encode!(enc_small_bincode, 10, 10, bincode_encode);
encode!(enc_median_bincode, 100, 100, bincode_encode);
encode!(enc_large_bincode, 1000, 1000, bincode_encode);

encode!(enc_small_json, 10, 10, json_encode);
encode!(enc_median_json, 100, 100, json_encode);
encode!(enc_large_json, 1000, 1000, json_encode);

macro_rules! decode {
    ($name:ident, $str_len:expr, $vec_len:expr, $enc:expr, $dec:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let t = make_struct($str_len, $vec_len);
            let bytes = $enc(&t);
            b.iter(|| {
                let x: TestStruct = $dec(&bytes).unwrap();
                x
            });
        }
    };
}

decode!(dec_small_sbor, 10, 10, sbor_encode, sbor_decode);
decode!(dec_median_sbor, 100, 100, sbor_encode, sbor_decode);
decode!(dec_large_sbor, 1000, 1000, sbor_encode, sbor_decode);

decode!(dec_small_bincode, 10, 10, bincode_encode, bincode_decode);
decode!(dec_median_bincode, 100, 100, bincode_encode, bincode_decode);
decode!(
    dec_large_bincode,
    1000,
    1000,
    bincode_encode,
    bincode_decode
);

decode!(dec_small_json, 10, 10, json_encode, json_decode);
decode!(dec_median_json, 100, 100, json_encode, json_decode);
decode!(dec_large_json, 1000, 1000, json_encode, json_decode);
