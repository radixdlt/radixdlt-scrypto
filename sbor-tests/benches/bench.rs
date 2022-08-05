#[macro_use]
extern crate bencher;
use bencher::Bencher;

mod adapter;
mod data;

const SIMPLE_REAPT: usize = 32;

fn encode_simple_json(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    b.iter(|| adapter::json_encode(&t));
}

fn encode_simple_bincode(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    b.iter(|| adapter::bincode_encode(&t));
}

fn encode_simple_sbor(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    b.iter(|| sbor::encode_with_static_info(&t));
}

fn encode_simple_sbor_no_static_info(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    b.iter(|| sbor::encode_no_static_info(&t));
}

fn decode_simple_json(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    let bytes = adapter::json_encode(&t);
    b.iter(|| adapter::json_decode::<data::simple::SimpleStruct>(&bytes));
}

fn decode_simple_bincode(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    let bytes = adapter::bincode_encode(&t);
    b.iter(|| adapter::bincode_decode::<data::simple::SimpleStruct>(&bytes));
}

fn decode_simple_sbor(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    let bytes = sbor::encode_with_static_info(&t);
    b.iter(|| sbor::decode_with_static_info::<data::simple::SimpleStruct>(&bytes));
}

fn decode_simple_sbor_no_static_info(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    let bytes = sbor::encode_no_static_info(&t);
    b.iter(|| sbor::decode_no_static_info::<data::simple::SimpleStruct>(&bytes));
}

benchmark_group!(
    encode_simple,
    encode_simple_json,
    encode_simple_bincode,
    encode_simple_sbor,
    encode_simple_sbor_no_static_info
);
benchmark_group!(
    decode_simple,
    decode_simple_json,
    decode_simple_bincode,
    decode_simple_sbor,
    decode_simple_sbor_no_static_info
);
benchmark_main!(encode_simple, decode_simple);
