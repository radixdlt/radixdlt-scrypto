use fuzz_tests::fuzz_template;
use radix_engine_common::prelude::*;

fuzz_template!(|data: &[u8]| {
    if let Ok(value) = scrypto_decode::<ScryptoValue>(data) {
        match scrypto_encode(&value) {
            Ok(bytes) => assert_eq!(data, bytes.as_slice()),
            e => panic!("{:?}", e),
        }
    }
});

#[test]
fn test_sbor_generate_fuzz_input_data() {
    use bincode::serialize;
    use std::fs;

    let val = scrypto_encode::<i32>(&20i32).unwrap();
    let serialized = serialize(&val).unwrap();
    fs::write(format!("sbor_{:03?}.raw", 0), serialized).expect("Unable to write file");

    let val = scrypto_encode::<Decimal>(&Decimal::ONE).unwrap();
    let serialized = serialize(&val).unwrap();
    fs::write(format!("sbor_{:03?}.raw", 1), serialized).expect("Unable to write file");
}
