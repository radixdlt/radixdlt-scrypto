#![feature(test)]
extern crate test;
use test::Bencher;

use sbor::{
    sbor_decode, sbor_decode_no_schema, sbor_encode, sbor_encode_no_schema, Decode, Encode,
};
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

encode!(encode_small_sbor, 10, 10, sbor_encode);
encode!(encode_median_sbor, 100, 100, sbor_encode);
encode!(encode_large_sbor, 1000, 1000, sbor_encode);

#[rustfmt::skip]
encode!(encode_small_sbor_no_schema, 10, 10, sbor_encode_no_schema);
#[rustfmt::skip]
encode!(encode_median_sbor_no_schema, 100, 100, sbor_encode_no_schema);
#[rustfmt::skip]
encode!(encode_large_sbor_no_schema, 1000, 1000, sbor_encode_no_schema);

encode!(encode_small_bincode, 10, 10, bincode_encode);
encode!(encode_median_bincode, 100, 100, bincode_encode);
encode!(encode_large_bincode, 1000, 1000, bincode_encode);

encode!(encode_small_json, 10, 10, json_encode);
encode!(encode_median_json, 100, 100, json_encode);
encode!(encode_large_json, 1000, 1000, json_encode);

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

decode!(decode_small_sbor, 10, 10, sbor_encode, sbor_decode);
decode!(decode_median_sbor, 100, 100, sbor_encode, sbor_decode);
decode!(decode_large_sbor, 1000, 1000, sbor_encode, sbor_decode);

#[rustfmt::skip]
decode!(decode_small_sbor_no_schema, 10, 10, sbor_encode_no_schema, sbor_decode_no_schema);
#[rustfmt::skip]
decode!(decode_median_sbor_no_schema, 100, 100, sbor_encode_no_schema, sbor_decode_no_schema);
#[rustfmt::skip]
decode!(decode_large_sbor_no_schema, 1000, 1000, sbor_encode_no_schema, sbor_decode_no_schema);

#[rustfmt::skip]
decode!(decode_small_bincode, 10, 10, bincode_encode, bincode_decode);
#[rustfmt::skip]
decode!(decode_median_bincode, 100, 100, bincode_encode, bincode_decode);
#[rustfmt::skip]
decode!(decode_large_bincode, 1000, 1000, bincode_encode, bincode_decode);

decode!(decode_small_json, 10, 10, json_encode, json_decode);
decode!(decode_median_json, 100, 100, json_encode, json_decode);
decode!(decode_large_json, 1000, 1000, json_encode, json_decode);
