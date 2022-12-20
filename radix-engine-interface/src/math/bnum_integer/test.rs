use super::*;
#[cfg(test)]
use num_bigint::{BigInt,Sign};

use radix_engine_interface::*;

test_impl! {BnumI256, BnumI512, BnumU256, BnumU512}

test_add_all! {
    (BnumI256, BnumI512, BnumU256, BnumU512),
    (BnumI256, BnumI512, BnumU256, BnumU512)
}

test_signed! { BnumI256, BnumI512 }
test_unsigned! { BnumU256, BnumU512 }

test_from_all_types_safe_builtin! {BnumI256, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {BnumI256, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {BnumI512, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {BnumI512, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {BnumU256, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {BnumU512, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_safe! {BnumI512, (BnumI256)}
test_from_all_types_safe_safe! {BnumU512, (BnumU256)}

