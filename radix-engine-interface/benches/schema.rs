use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine_interface::data::{
    match_schema_with_value, scrypto_decode, scrypto_encode, ScryptoCustomTypeExtension,
    ScryptoValue,
};
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Clone, Sbor, LegacyDescribe)]
pub enum SimpleEnum {
    Unit,
    Unnamed(u32),
    Named { x: u32, y: u32 },
}

#[derive(Debug, Clone, Sbor, LegacyDescribe)]
pub struct SimpleStruct {
    pub number: u64,
    pub string: String,
    pub bytes: Vec<u8>,
    pub vector: Vec<u16>,
    pub enumeration: Vec<SimpleEnum>,
    pub map: BTreeMap<String, String>,
}

pub fn get_simple_dataset(repeat: usize) -> SimpleStruct {
    let mut data = SimpleStruct {
        number: 12345678901234567890,
        string: "dummy".repeat(repeat).to_owned(),
        bytes: vec![123u8; repeat],
        vector: vec![12345u16; repeat],
        enumeration: vec![
            SimpleEnum::Named {
                x: 1234567890,
                y: 1234567890,
            };
            repeat
        ],
        map: BTreeMap::new(),
    };

    for i in 0..repeat {
        data.map
            .insert(format!("Key_{}", i), format!("Value_{}", i));
    }

    data
}

const REPEAT: usize = 1000;

fn bench_schema_legacy(b: &mut Criterion) {
    use scrypto_abi::LegacyDescribe;

    let bytes = scrypto_encode(&get_simple_dataset(REPEAT)).unwrap();
    let schema = SimpleStruct::describe();
    b.bench_function("Legacy Schema", |b| {
        b.iter(|| {
            let value: ScryptoValue = scrypto_decode(&bytes).unwrap();
            let result = match_schema_with_value(&schema, &value);
            assert!(result)
        })
    });
}

fn bench_schema_new(b: &mut Criterion) {
    let bytes = scrypto_encode(&get_simple_dataset(REPEAT)).unwrap();
    let (type_index, schema) =
        generate_full_schema_from_single_type::<SimpleStruct, ScryptoCustomTypeExtension>();
    b.bench_function("New Schema", |b| {
        b.iter(|| {
            let result = validate_payload_with_schema(&bytes, &schema, type_index);
            assert!(result.is_ok())
        })
    });
}

criterion_group!(bench_schema, bench_schema_legacy, bench_schema_new);
criterion_main!(bench_schema);
