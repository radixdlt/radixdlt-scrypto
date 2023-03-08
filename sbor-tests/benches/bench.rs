#[macro_use]
extern crate bencher;
use bencher::Bencher;
use sbor::{basic_decode, basic_encode};
use sbor_tests::adapter;

mod data;

const REPEAT: usize = 1000;

fn encode_simple_json(b: &mut Bencher) {
    let t = data::get_simple_dataset(REPEAT);
    b.iter(|| adapter::json_encode(&t));
}

fn encode_simple_bincode(b: &mut Bencher) {
    let t = data::get_simple_dataset(REPEAT);
    b.iter(|| adapter::bincode_encode(&t));
}

fn encode_simple_sbor(b: &mut Bencher) {
    let t = data::get_simple_dataset(REPEAT);
    b.iter(|| basic_encode(&t));
}

fn decode_simple_json(b: &mut Bencher) {
    let t = data::get_simple_dataset(REPEAT);
    let bytes = adapter::json_encode(&t);
    b.iter(|| adapter::json_decode::<data::SimpleStruct>(&bytes));
}

fn decode_simple_bincode(b: &mut Bencher) {
    let t = data::get_simple_dataset(REPEAT);
    let bytes = adapter::bincode_encode(&t);
    b.iter(|| adapter::bincode_decode::<data::SimpleStruct>(&bytes));
}

fn decode_simple_sbor(b: &mut Bencher) {
    let t = data::get_simple_dataset(REPEAT);
    let bytes = basic_encode(&t).unwrap();
    b.iter(|| basic_decode::<data::SimpleStruct>(&bytes));
}

benchmark_group!(
    encode_simple,
    encode_simple_json,
    encode_simple_bincode,
    encode_simple_sbor,
);
benchmark_group!(
    decode_simple,
    decode_simple_json,
    decode_simple_bincode,
    decode_simple_sbor,
);
benchmark_main!(encode_simple, decode_simple);
