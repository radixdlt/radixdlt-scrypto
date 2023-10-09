#![no_main]

use libfuzzer_sys::fuzz_target;

use cosmwasm_std::SignedDecimal256;
use radix_engine_common::prelude::*;
use scrypto::prelude::*;

fuzz_target!(|data: &[u8]| {
    if data.len() == 48 {
        let mut p = data;
        let u1 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u2 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u3 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        p = &p[8..];
        let u4 = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);

        let d1 = Decimal(I192::from_digits([u1, u2, u3]));
        let actual = d1.checked_powi(u4.into());

        let x1 = SignedDecimal256::from_str(&d1.to_string()).unwrap();
        let expected = x1.checked_pow(u4);

        match (actual, expected) {
            (Some(actual), Ok(expected)) => {
                let delta = Decimal::from_str(&actual.to_string()).unwrap()
                    - Decimal::from_str(&expected.to_string()).unwrap();
                if let Some(v) = actual.checked_abs() {
                    let min = v * Decimal::try_from("-0.00000001").unwrap();
                    let max = v * Decimal::try_from("0.00000001").unwrap();
                    assert!(
                        delta >= min && delta <= max,
                        "{}, {}, {}, {}",
                        d1,
                        u4,
                        actual,
                        expected
                    );
                }
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
