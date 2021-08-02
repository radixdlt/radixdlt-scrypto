#[macro_use]
extern crate bencher;
use bencher::Bencher;

mod adapter;
mod data;

fn encode_twitter_json(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    b.iter(|| adapter::json_encode(&t));
}

fn encode_twitter_bincode(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    b.iter(|| adapter::bincode_encode(&t));
}

fn encode_twitter_sbor(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    b.iter(|| sbor::sbor_encode(&t));
}

fn encode_twitter_sbor_no_metadata(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    b.iter(|| sbor::sbor_encode_no_metadata(&t));
}

fn decode_twitter_json(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    let bytes = adapter::json_encode(&t);
    b.iter(|| adapter::json_decode::<data::twitter::Twitter>(&bytes));
}

fn decode_twitter_bincode(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    let bytes = adapter::bincode_encode(&t);
    b.iter(|| adapter::bincode_decode::<data::twitter::Twitter>(&bytes));
}

fn decode_twitter_sbor(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    let bytes = sbor::sbor_encode(&t);
    b.iter(|| sbor::sbor_decode::<data::twitter::Twitter>(&bytes));
}

fn decode_twitter_sbor_no_metadata(b: &mut Bencher) {
    let t = data::get_twitter_dataset();
    let bytes = sbor::sbor_encode_no_metadata(&t);
    b.iter(|| sbor::sbor_decode_no_metadata::<data::twitter::Twitter>(&bytes));
}

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
    b.iter(|| sbor::sbor_encode(&t));
}

fn encode_simple_sbor_no_metadata(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    b.iter(|| sbor::sbor_encode_no_metadata(&t));
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
    let bytes = sbor::sbor_encode(&t);
    b.iter(|| sbor::sbor_decode::<data::simple::SimpleStruct>(&bytes));
}

fn decode_simple_sbor_no_metadata(b: &mut Bencher) {
    let t = data::get_simple_dataset(SIMPLE_REAPT);
    let bytes = sbor::sbor_encode_no_metadata(&t);
    b.iter(|| sbor::sbor_decode_no_metadata::<data::simple::SimpleStruct>(&bytes));
}

benchmark_group!(
    encode_twitter,
    encode_twitter_json,
    encode_twitter_bincode,
    encode_twitter_sbor,
    encode_twitter_sbor_no_metadata
);

benchmark_group!(
    decode_twitter,
    decode_twitter_json,
    decode_twitter_bincode,
    decode_twitter_sbor,
    decode_twitter_sbor_no_metadata
);
benchmark_group!(
    encode_simple,
    encode_simple_json,
    encode_simple_bincode,
    encode_simple_sbor,
    encode_simple_sbor_no_metadata
);
benchmark_group!(
    decode_simple,
    decode_simple_json,
    decode_simple_bincode,
    decode_simple_sbor,
    decode_simple_sbor_no_metadata
);
benchmark_main!(encode_twitter, decode_twitter, encode_simple, decode_simple);
