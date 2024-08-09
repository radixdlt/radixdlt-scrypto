use scrypto::prelude::*;
use scrypto::types::Slice;

static mut START: *const u8 = std::ptr::null();
static mut STOP: *const u8 = std::ptr::null();

#[no_mangle]
pub extern "C" fn __sanitizer_cov_8bit_counters_init(start: *const u8, stop: *const u8) {
    unsafe {
        START = start;
        STOP = stop;
    }
}

#[no_mangle]
pub unsafe extern "C" fn dump_coverage_counters() -> Slice {
    let length = STOP.offset_from(START) as usize;
    Slice::new(START as u32, length as u32)
}

#[blueprint]
mod fuzz_blueprint {
    struct FuzzBlueprint;

    impl FuzzBlueprint {
        fn deposits_and_withdraw(deposits: Vec<Decimal>, withdraws: Vec<Decimal>) -> bool {
            if deposits.len() == 0 {
                return false;
            }

            let mut deposits_sum : Decimal = deposits.iter().fold(dec!(0), |acc, x| acc + *x);
            let mut deposits : IndexSet<Decimal> = deposits.into_iter().collect();

            for withdraw in withdraws {
                if !deposits.swap_remove(&withdraw) {
                    continue;
                }
                assert!(deposits_sum >= withdraw);
                deposits_sum -= withdraw;
            }
            assert!(deposits.len() != 0 || deposits_sum == dec!(0));

            return true;
        }

        pub fn fuzz(input: Vec<u8>) -> Vec<u8> {
            let mut decoder = ScryptoDecoder::new(&input, 10);
            let mut deposits = Vec::new();
            let mut withdraws = Vec::new();

            // fuzzer friendly decoding
            while let Ok(decimal) = decoder.decode_deeper_body_with_value_kind::<Decimal>(Decimal::value_kind()) {
                if decimal == dec!(0) || decimal > dec!(10000) || decimal < dec!(-10000) {
                    break;
                }
                if decimal > dec!(0) {
                    deposits.push(decimal);
                } else {
                    withdraws.push(-decimal);
                }
            }
            Self::deposits_and_withdraw(deposits, withdraws);

            unsafe { 
                let length = STOP.offset_from(START) as usize;
                let slice = std::slice::from_raw_parts(START, length);
                slice.to_vec()         
            }
        }

        pub fn get_counters_size() -> usize {
            unsafe {
                STOP.offset_from(START) as usize
            }
        }
    }
}

