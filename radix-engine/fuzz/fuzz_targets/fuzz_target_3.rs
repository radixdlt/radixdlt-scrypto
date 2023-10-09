#![no_main]

use libfuzzer_sys::fuzz_target;

use cosmwasm_std::SignedDecimal256;
use radix_engine_common::prelude::*;

fuzz_target!(|data: &[u8]| {
    if data.len() == 48 {
        let mut p = data;
        let u1 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u2 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u3 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u4 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u5 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u6 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);

        let d1 = Decimal(I192::from_digits([u1, u2, u3]));
        let d2 = Decimal(I192::from_digits([u4, u5, u6]));
        let actual = d1.checked_mul(d2);

        let x1 = SignedDecimal256::from_str(&d1.to_string()).unwrap();
        let x2 = SignedDecimal256::from_str(&d2.to_string()).unwrap();
        let expected = x1.checked_mul(x2);

        match (actual, expected) {
            (Some(actual), Ok(expected)) => {
                assert_eq!(actual.to_string(), expected.to_string());
            }
            (Some(_), Err(_)) => {
                panic!();
            }
            (None, Ok(_)) => {
                // This is fine as we have less bytes!
            }
            (None, Err(_)) => {}
        }
    }
});
