use super::*;
#[cfg(test)]
use sbor::rust::format;

use radix_engine_interface::*;

test_impl! { I8, I16, I32, I64, I128, I256, I384, I512, I768, U8, U16, U32, U64, U128, U256, U384, U512, U768 }

test_add_all! {
    (I8, I16, I32, I64, I128, I256, I384, I512, I768, U8, U16, U32, U64, U128, U256, U384, U512, U768),
    (U8, U16, U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)
}

test_signed! { I8, I16, I32, I64, I128, I256, I384, I512, I768 }

test_unsigned! { U8, U16, U32, U64, U128, U256, U384, U512, U768 }

test_from_all_types_builtin_safe! {i8, (I8)}

test_from_all_types_builtin_safe! {i16, (I8, I16)}
test_from_all_types_builtin_safe! {i16, (U8)}

test_from_all_types_builtin_safe! {i32, (I8, I16, I32)}
test_from_all_types_builtin_safe! {i32, (U8, U16)}

test_from_all_types_builtin_safe! {i64, (I8, I16, I32, I64)}
test_from_all_types_builtin_safe! {i64, (U8, U16, U32)}

test_from_all_types_builtin_safe! {i128, (I8, I16, I32, I64, I128)}
test_from_all_types_builtin_safe! {i128, (U8, U16, U32, U64)}

test_from_all_types_builtin_safe! {u8, (U8)}

test_from_all_types_builtin_safe! {u16, (U8, U16)}

test_from_all_types_builtin_safe! {u32, (U8, U16, U32)}

test_from_all_types_builtin_safe! {u64, (U8, U16, U32, U64)}

test_from_all_types_builtin_safe! {u128, (U8, U16, U32, U64, U128)}

test_from_all_types_safe_builtin! {I8, (i8)}

test_from_all_types_safe_builtin! {I16, (i8, i16)}
test_from_all_types_safe_builtin! {I16, (u8)}

test_from_all_types_safe_builtin! {I32, (i8, i16, i32)}
test_from_all_types_safe_builtin! {I32, (u8, u16)}

test_from_all_types_safe_builtin! {I64, (i8, i16, i32, i64)}
test_from_all_types_safe_builtin! {I64, (u8, u16, u32)}

test_from_all_types_safe_builtin! {I128, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I128, (u8, u16, u32, u64)}

test_from_all_types_safe_builtin! {I256, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I256, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I384, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I384, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I512, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I512, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {U8, (u8)}

test_from_all_types_safe_builtin! {U16, (u8, u16)}

test_from_all_types_safe_builtin! {U32, (u8, u16, u32)}

test_from_all_types_safe_builtin! {U64, (u8, u16, u32, u64)}

test_from_all_types_safe_builtin! {U128, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {U256, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {U384, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {U512, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_safe! {I16, (I8)}
test_from_all_types_safe_safe! {I16, (U8)}

test_from_all_types_safe_safe! {I32, (I8, I16)}
test_from_all_types_safe_safe! {I32, (U8, U16)}

test_from_all_types_safe_safe! {I64, (I8, I16, I32)}
test_from_all_types_safe_safe! {I64, (U8, U16, U32)}

test_from_all_types_safe_safe! {I128, (I8, I16, I32, I64)}
test_from_all_types_safe_safe! {I128, (U8, U16, U32, U64)}

test_from_all_types_safe_safe! {I256, (I8, I16, I32, I64, I128)}
test_from_all_types_safe_safe! {I256, (U8, U16, U32, U64, U128)}

test_from_all_types_safe_safe! {I384, (I8, I16, I32, I64, I128, I256)}
test_from_all_types_safe_safe! {I384, (U8, U16, U32, U64, U128, U256)}

test_from_all_types_safe_safe! {I512, (I8, I16, I32, I64, I128, I256, I384)}
test_from_all_types_safe_safe! {I512, (U8, U16, U32, U64, U128, U256, U384)}

test_from_all_types_safe_safe! {I768, (I8, I16, I32, I64, I128, I256, I384, I512)}
test_from_all_types_safe_safe! {I768, (U8, U16, U32, U64, U128, U256, U384, U512)}

test_from_all_types_safe_safe! {U16, (U8)}

test_from_all_types_safe_safe! {U32, (U8, U16)}

test_from_all_types_safe_safe! {U64, (U8, U16, U32)}

test_from_all_types_safe_safe! {U128, (U8, U16, U32, U64)}

test_from_all_types_safe_safe! {U256, (U8, U16, U32, U64, U128)}

test_from_all_types_safe_safe! {U384, (U8, U16, U32, U64, U128, U256 )}

test_from_all_types_safe_safe! {U512, (U8, U16, U32, U64, U128, U256, U384)}

#[test]
fn test_format_i256() {
    let i256 = I256::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i256),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i256() {
    let i256 =
        I256::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i256),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i256() {
    let i256 =
        I256::from("10000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i256),
        "10000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_i256() {
    let i256 = I256::from("0");
    assert_eq!(format!("{}", i256), "0");
}

#[test]
fn test_format_121_i256() {
    let i256 = I256::from("121");
    assert_eq!(format!("{}", i256), "121");
}

#[test]
fn test_format_i256_minus() {
    let i256 = I256::from("-12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i256),
        "-12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i256_minus() {
    let i256 =
        I256::from("-1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i256),
        "-1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i256_minus() {
    let i256 = I256::from(
        "-10000000000000000000000000000000000000012345678901234567890123456789012345678",
    );
    assert_eq!(
        format!("{}", i256),
        "-10000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_121_i256_minus() {
    let i256 = I256::from("-121");
    assert_eq!(format!("{}", i256), "-121");
}
//---------------
#[test]
fn test_format_i384() {
    let i384 = I384::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i384),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i384() {
    let i384 =
        I384::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i384),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i384() {
    let i384 =
        I384::from("1000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i384),
        "1000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_i384() {
    let i384 = I384::from("0");
    assert_eq!(format!("{}", i384), "0");
}

#[test]
fn test_format_121_i384() {
    let i384 = I384::from("121");
    assert_eq!(format!("{}", i384), "121");
}

#[test]
fn test_format_i384_minus() {
    let i384 = I384::from("-12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i384),
        "-12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i384_minus() {
    let i384 =
        I384::from("-1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i384),
        "-1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i384_minus() {
    let i384 =
        I384::from("-1000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i384),
        "-1000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_121_i384_minus() {
    let i384 = I384::from("-121");
    assert_eq!(format!("{}", i384), "-121");
}

#[test]
fn test_format_i512() {
    let i512 = I512::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i512),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i512() {
    let i512 =
        I512::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i512),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i512() {
    let i512 =
        I512::from("100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i512),
        "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_i512() {
    let i512 = I512::from("0");
    assert_eq!(format!("{}", i512), "0");
}

#[test]
fn test_format_121_i512() {
    let i512 = I512::from("121");
    assert_eq!(format!("{}", i512), "121");
}

#[test]
fn test_format_i512_minus() {
    let i512 = I512::from("-12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i512),
        "-12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i512_minus() {
    let i512 =
        I512::from("-1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i512),
        "-1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i512_minus() {
    let i512 =
        I512::from("-100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i512),
        "-100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_121_i512_minus() {
    let i512 = I512::from("-121");
    assert_eq!(format!("{}", i512), "-121");
}

#[test]
fn test_format_i768() {
    let i768 = I768::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i768),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i768() {
    let i768 =
        I768::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i768),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i768() {
    let i768 =
        I768::from("100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i768),
        "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_i768() {
    let i768 = I768::from("0");
    assert_eq!(format!("{}", i768), "0");
}

#[test]
fn test_format_121_i768() {
    let i768 = I768::from("121");
    assert_eq!(format!("{}", i768), "121");
}

#[test]
fn test_format_i768_minus() {
    let i768 = I768::from("-12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i768),
        "-12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_i768_minus() {
    let i768 =
        I768::from("-1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i768),
        "-1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_i768_minus() {
    let i768 =
        I768::from("-100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", i768),
        "-100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_121_i768_minus() {
    let i768 = I768::from("-121");
    assert_eq!(format!("{}", i768), "-121");
}

#[test]
fn test_format_u256() {
    let u256 = U256::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u256),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_u256() {
    let u256 =
        U256::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u256),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_u256() {
    let u256 =
        U256::from("10000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u256),
        "10000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_u256() {
    let u256 = U256::from("0");
    assert_eq!(format!("{}", u256), "0");
}

#[test]
fn test_format_121_u256() {
    let u256 = U256::from("121");
    assert_eq!(format!("{}", u256), "121");
}

#[test]
fn test_format_u384() {
    let u384 = U384::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u384),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_u384() {
    let u384 =
        U384::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u384),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_u384() {
    let u384 =
        U384::from("1000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u384),
        "1000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_u384() {
    let u384 = U384::from("0");
    assert_eq!(format!("{}", u384), "0");
}

#[test]
fn test_format_121_u384() {
    let u384 = U384::from("121");
    assert_eq!(format!("{}", u384), "121");
}

#[test]
fn test_format_u512() {
    let u512 = U512::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u512),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_u512() {
    let u512 =
        U512::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u512),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_u512() {
    let u512 =
        U512::from("100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u512),
        "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_u512() {
    let u512 = U512::from("0");
    assert_eq!(format!("{}", u512), "0");
}

#[test]
fn test_format_121_u512() {
    let u512 = U512::from("121");
    assert_eq!(format!("{}", u512), "121");
}

#[test]
fn test_format_u768() {
    let u768 = U768::from("12345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u768),
        "12345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_long_u768() {
    let u768 =
        U768::from("1234567890123456789012345678901234567812345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u768),
        "1234567890123456789012345678901234567812345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_longer_u768() {
    let u768 =
        U768::from("10000000000000000000000000000000000000012345678901234567890123456789012345678");
    assert_eq!(
        format!("{}", u768),
        "10000000000000000000000000000000000000012345678901234567890123456789012345678"
    );
}

#[test]
fn test_format_zero_u768() {
    let u768 = U768::from("0");
    assert_eq!(format!("{}", u768), "0");
}

#[test]
fn test_format_121_u768() {
    let u768 = U768::from("121");
    assert_eq!(format!("{}", u768), "121");
}
