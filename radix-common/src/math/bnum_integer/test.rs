#[cfg(test)]
use super::*;
use paste::paste;

#[cfg(test)]
use num_bigint::{BigInt, Sign};

use radix_common::*;
#[allow(unused_imports)] // It's needed by the `test_impl!` macro
use sbor::rust::cmp::Ordering;

test_impl! {I192, I256, I320, I384, I448, I512, I768}
test_impl! {U192, U256, U320, U384, U448, U512, U768}

test_add_all! {
    (I192, I256, I320, I384, I448, I512, I768,
     U192, U256, U320, U384, U448, U512, U768),
    (I192, I256, I320, I384, I448, I512, I768,
     U192, U256, U320, U384, U448, U512, U768)
}

test_signed! { I192, I256, I320, I384, I448, I512, I768 }
test_unsigned! { U192, U256, U320, U384, U448, U512, U768 }

test_from_all_types_safe_builtin! {I192, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I192, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I256, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I256, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I320, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I320, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I384, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I384, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I448, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I448, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I512, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I512, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {I768, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {I768, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {U192, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {U256, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {U320, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {U384, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {U448, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {U512, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {U768, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_safe! {I192, (I256, I320, I384, I448, I512, I768)}
test_from_all_types_safe_safe! {I256, (I192, I320, I384, I448, I512, I768)}
test_from_all_types_safe_safe! {I320, (I192, I256, I384, I448, I512, I768)}
test_from_all_types_safe_safe! {I384, (I192, I256, I320, I448, I512, I768)}
test_from_all_types_safe_safe! {I448, (I192, I256, I320, I384, I512, I768)}
test_from_all_types_safe_safe! {I512, (I192, I256, I320, I384, I448, I768)}
test_from_all_types_safe_safe! {I768, (I192, I256, I320, I384, I448, I512)}

test_from_all_types_safe_safe! {U192, (U256, U320, U384, U448, U512, U768)}
test_from_all_types_safe_safe! {U256, (U192, U320, U384, U448, U512, U768)}
test_from_all_types_safe_safe! {U320, (U192, U256, U384, U448, U512, U768)}
test_from_all_types_safe_safe! {U384, (U192, U256, U320, U448, U512, U768)}
test_from_all_types_safe_safe! {U448, (U192, U256, U320, U384, U512, U768)}
test_from_all_types_safe_safe! {U512, (U192, U256, U320, U384, U448, U768)}
test_from_all_types_safe_safe! {U768, (U192, U256, U320, U384, U448, U512)}

#[cfg(test)]
macro_rules! assert_int_size {
    ($($bits: literal $t: ident),*)  => {
        $(
            assert_eq!($t::BITS, $bits);
        )*
    }
}

#[test]
fn test_int_size() {
    assert_int_size! {
        192 I192,
        256 I256,
        320 I320,
        384 I384,
        448 I448,
        512 I512,
        768 I768,
        192 U192,
        256 U256,
        320 U320,
        384 U384,
        448 U448,
        512 U512,
        768 U768
    }
}

#[cfg(test)]
macro_rules! test_bnums {
    ($($t: ident),*)  => {
        paste! {
            $(
                #[test]
                fn [<test_ $t:lower _add>] () {
                    assert_eq!((<$t>::ONE + <$t>::ONE).to_string(), "2");
                    assert_eq!(<$t>::from(17_u32) + <$t>::from(31_u32), <$t>::from(48_u32));
                    let mut bnum = <$t>::ONE;
                    bnum += <$t>::from_str("101").unwrap();
                    assert_eq!(bnum, <$t>::from_str("102").unwrap());

                    if <$t>::MIN < <$t>::ZERO {
                        let mut bnum = <$t>::MAX;
                        bnum += <$t>::try_from(-1_i32).unwrap();
                        assert_eq!(bnum, <$t>::MAX - <$t>::ONE);

                        assert_eq!(<$t>::MIN + <$t>::MAX, <$t>::ZERO - <$t>::ONE);
                    }
                }

                #[test]
                fn [< test_ $t:lower _sub >]() {
                    assert_eq!(<$t>::ONE - <$t>::ONE, <$t>::ZERO);

                    if <$t>::MIN < <$t>::ZERO {
                        assert_eq!(<$t>::from(17_u32) - <$t>::from(31_u32), <$t>::try_from(-14).unwrap());
                        let mut bnum = <$t>::from(101_u32);
                        bnum -= <$t>::from_str("102").unwrap();
                        assert_eq!(bnum, <$t>::from_str("-1").unwrap());
                    }

                    let mut bnum = <$t>::MAX;
                    bnum -= <$t>::ONE;
                    assert_eq!(bnum, <$t>::MAX - <$t>::ONE);
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_ $t:lower _add_overflow_panic_1>] () {
                    let mut bnum = <$t>::MAX;
                    bnum += <$t>::from(1_u32);
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _sub_overflow_panic_2 >]() {
                    let mut bnum = <$t>::MIN;
                    bnum -= <$t>::ONE;
                }

                #[test]
                fn [< test_ $t:lower _mul >]() {
                    assert_eq!(<$t>::from(4_u32) * <$t>::from(5_u32), <$t>::from(20_u32));
                    let mut bnum = <$t>::from(12387_u32);
                    bnum *= <$t>::from_str("1203203031").unwrap();
                    assert_eq!(bnum, <$t>::from(14904075944997_u128));

                    if <$t>::MIN < <$t>::ZERO {
                        let mut bnum = <$t>::from(12387_u32);
                        bnum *= <$t>::from_str("-1203203031").unwrap();
                        assert_eq!(bnum, <$t>::try_from(-14904075944997_i128).unwrap());
                    }
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _mul_overflow_panic_1 >] () {
                    let mut bnum = <$t>::MAX;
                    bnum *= <$t>::from(2_u32);
                }


                #[test]
                fn [< test_ $t:lower _pow >](){
                    assert_eq!(<$t>::from(3_u32).pow(3), <$t>::from(27_u32));

                    assert_eq!(
                        <$t>::from(153_u32).pow(20),
                        <$t>::from_str("49411565790213547262766437937260727785410401").unwrap()
                    );
                    assert_eq!(
                        <$t>::from(153_u32).pow(25),
                        <$t>::from_str("4142721807044360524568533828494071080154747151557663193").unwrap()
                    );
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _pow_overflow_panic_1 >]() {
                    if <$t>::BITS == 192 {
                        let _ = <$t>::from(153_u32).pow(40);
                    } else if <$t>::BITS == 256 {
                        let _ = <$t>::from(153_u32).pow(40);
                    } else if <$t>::BITS == 320 {
                        let _ = <$t>::from(153_u32).pow(60);
                    } else if <$t>::BITS == 384 {
                        let _ = <$t>::from(153_u32).pow(60);
                    } else if <$t>::BITS == 448 {
                        let _ = <$t>::from(153_u32).pow(70);
                    } else if <$t>::BITS == 512 {
                        let _ = <$t>::from(153_u32).pow(80);
                    } else if <$t>::BITS == 768 {
                        let _ = <$t>::from(153_u32).pow(120);
                    } else {
                        panic!("Unknown bits size {}", <$t>::BITS);
                    }
                }

                #[test]
                fn [< test_ $t:lower _root >]() {
                    assert_eq!(<$t>::from(9_u32).sqrt(), <$t>::from(3_u32));
                    assert_eq!(<$t>::from(27_u32).cbrt(), <$t>::from(3_u32));

                    assert_eq!(<$t>::from(9_u32).nth_root(2), <$t>::from(3_u32));
                    assert_eq!(<$t>::from(27_u32).nth_root(3), <$t>::from(3_u32));
                    assert_eq!(<$t>::from(14966675814359580587845230627_u128).nth_root(13), <$t>::from(147_u32));
                    assert_eq!(
                        <$t>::from_str("290437112829027226192310037731274304321654649956335616").unwrap().nth_root(17),
                        <$t>::from_str("1396").unwrap()
                    );

                    if <$t>::MIN < <$t>::ZERO {
                        assert_eq!(<$t>::try_from(-27).unwrap().nth_root(3), <$t>::try_from(-3).unwrap());
                        assert_eq!(<$t>::try_from(-14966675814359580587845230627_i128).unwrap().nth_root(13), <$t>::try_from(-147).unwrap());
                    }
                }

                #[test]
                fn [< test_ $t:lower _to_string >]() {
                    assert_eq!(<$t>::ONE.to_string(), "1");
                    assert_eq!(<$t>::ZERO.to_string(), "0");
                    assert_eq!(<$t>::from_str("0").unwrap(), <$t>::ZERO);

                    if <$t>::MIN < <$t>::ZERO {
                        assert_eq!(<$t>::try_from(-1).unwrap().to_string(), "-1");

                        assert_eq!(<$t>::from_str("-1").unwrap(), <$t>::try_from(-1).unwrap());
                    }
                }

                #[test]
                fn [< test_ $t:lower _to_primitive_ints >] () {
                    assert_eq!(<$t>::from_i8(1).unwrap(), <$t>::ONE);
                    assert_eq!(<$t>::try_from(21).unwrap().to_string(), "21");
                    assert_eq!(<$t>::from(21_u8).to_string(), "21");

                    let bnum: $t = 21_u32.into();
                    assert_eq!(bnum.to_string(), 21.to_string());

                    let i: i128 = <$t>::from(21_u8).try_into().unwrap();
                    assert_eq!(i, 21_i128);

                    let val = u8::try_from(<$t>::from(300_u32)).unwrap_err();
                    assert_eq!(val, [< Parse $t Error >]::Overflow);

                    if <$t>::MIN < <$t>::ZERO {
                        let val = u8::try_from(<$t>::try_from(-300_i32).unwrap()).unwrap_err();
                        assert_eq!(val, [< Parse $t Error >]::Overflow);
                    }
                }

                #[test]
                #[should_panic(expected = "Err")]
                fn [< test_ $t:lower _from_string_panic_1 >]() {
                    assert_eq!(<$t>::from_str("0x01").unwrap(), <$t>::try_from(-1).unwrap());
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _to_u128_panic >]() {
                    if <$t>::MIN < <$t>::ZERO {
                        let _u: u128 = <$t>::try_from(-21).unwrap().try_into().unwrap();
                    } else {
                        let _u: u128 = <$t>::from_str("290437112829027226192310037731274304321654649956335616").unwrap().try_into().unwrap();
                    }
                }
                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _to_i8_panic >]() {
                    let _i: i8 = <$t>::try_from(-260).unwrap().try_into().unwrap();
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _u16_panic >]() {
                    let _i: u16 = <$t>::from(123123123_u32).try_into().unwrap();
                }

            )*
        }
    }
}

#[cfg(test)]
test_bnums! { I192, I256, I320, I384, I448, I512, I768 }
#[cfg(test)]
test_bnums! { U192, U256, U320, U384, U448, U512, U768 }

#[cfg(test)]
macro_rules! test_bnums_signed {
    ($($t: ident),*)  => {
        paste! {
            $(
                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _mul_overflow_panic_2 >] () {
                    let bnum = <$t>::MIN;
                    let _m = bnum.mul(<$t>::from(2));
                }

            )*
        }
    }
}
#[cfg(test)]
test_bnums_signed! { I192, I256, I320, I384, I448, I512, I768 }

#[test]
#[should_panic(expected = "Err")]
fn test_string_to_bnum_panic_2() {
    assert_eq!(
        I256::MAX,
        I256::from_str(
            "578960446186580977117854925043439539266349923328202820197287920039565648199670"
        )
        .unwrap()
    );
}

macro_rules! test_to_from_bigint {
    ($($t: ident),*)  => {
        paste!{
            $(
                #[test]
                fn [<test_to_from_bigint_ $t:lower>]() {
                    assert_eq!($t::try_from(BigInt::from(147)).unwrap(), $t::from(147_u32));

                    assert_eq!(
                        $t::try_from(BigInt::from(1470198230918_i128)).unwrap(),
                        $t::from(1470198230918_u128)
                    );

                    let big = BigInt::from($t::MAX) + BigInt::from(1);
                    let err = $t::try_from(big).unwrap_err();
                    assert_eq!(err, [<Parse $t Error>]::Overflow);

                    assert_eq!(BigInt::try_from($t::from(123_u32)).unwrap(), BigInt::from(123));
                    assert_eq!(BigInt::from($t::ONE), BigInt::from(1));

                    assert_eq!(
                        BigInt::from($t::MAX),
                        BigInt::from_str(
                            &$t::MAX.to_string()
                        )
                        .unwrap()
                    );

                    assert_eq!(
                        BigInt::from($t::MIN),
                        BigInt::from_str(
                            &$t::MIN.to_string()
                        )
                        .unwrap()
                    );

                    // test signed types
                    if $t::MIN != $t::ZERO {
                        assert_eq!($t::try_from(BigInt::from(-147)).unwrap(), $t::try_from(-147).unwrap());
                        assert_eq!(
                            $t::try_from(BigInt::from(-1470198230918_i128)).unwrap(),
                            $t::try_from(-1470198230918_i128).unwrap()
                        );
                        let big = BigInt::from($t::MIN) - BigInt::from(1);
                        let err = $t::try_from(big).unwrap_err();
                        assert_eq!(err, [<Parse $t Error>]::Overflow);
                    }
                }
            )*
        }
    }
}
test_to_from_bigint! { I192, I256, I320, I384, I448, I512, I768 }
test_to_from_bigint! { U192, U256, U320, U384, U448, U512, U768 }

#[test]
fn test_bnum_to_bnum() {
    let a = I192::from(1);
    let b = U192::try_from(a).unwrap();
    assert_eq!(b, U192::ONE);

    let a = I256::from(1);
    let b = U256::try_from(a).unwrap();
    assert_eq!(b, U256::ONE);

    let a = I256::from(-123);
    let b = I512::from(a);
    assert_eq!(a.to_string(), b.to_string());

    let a = I256::MAX;
    let b = U256::try_from(a).unwrap();
    assert_eq!(a.to_string(), b.to_string());

    let a = I256::MIN;
    let b = I512::from(a);
    assert_eq!(a.to_string(), b.to_string());

    let a = U256::MAX;
    let b = I512::from(a);
    assert_eq!(a.to_string(), b.to_string());

    let a = U256::MAX;
    let b = I384::from(a);
    assert_eq!(a.to_string(), b.to_string());
}

#[test]
fn test_bnum_to_bnum_errors() {
    let i512 = I512::MIN;
    let err = I192::try_from(i512).unwrap_err();
    assert_eq!(err, ParseI192Error::Overflow);

    let i512 = I512::MIN;
    let err = I256::try_from(i512).unwrap_err();
    assert_eq!(err, ParseI256Error::Overflow);

    // I256::MAX + 1
    let i256_str = I256::MAX.to_string();
    let i512 = I512::from_str(&i256_str).unwrap() + I512::ONE;
    let err = I256::try_from(i512).unwrap_err();
    assert_eq!(err, ParseI256Error::Overflow);

    // I256::MIN - 1
    let i256_str = I256::MIN.to_string();
    let i512 = I512::from_str(&i256_str).unwrap() - I512::ONE;
    let err = I256::try_from(i512).unwrap_err();
    assert_eq!(err, ParseI256Error::Overflow);

    let u256 = U256::MAX;
    let err = I256::try_from(u256).unwrap_err();
    assert_eq!(err, ParseI256Error::Overflow);

    let i512_str = I512::MAX.to_string();
    let u512 = U512::from_str(&i512_str).unwrap() + U512::ONE;
    let err = U256::try_from(u512).unwrap_err();
    assert_eq!(err, ParseU256Error::Overflow);

    let u512 = U512::MAX;
    let err = I256::try_from(u512).unwrap_err();
    assert_eq!(err, ParseI256Error::Overflow);

    let a = U256::MAX;
    let b = I256::try_from(a).unwrap_err();
    assert_eq!(b, ParseI256Error::Overflow);

    let i512 = -I512::ONE;
    let err = U256::try_from(i512).unwrap_err();
    assert_eq!(err, ParseU256Error::NegativeToUnsigned);

    let i256 = I256::from(-123);
    let err = U512::try_from(i256).unwrap_err();
    assert_eq!(err, ParseU512Error::NegativeToUnsigned);

    let i384 = I384::MAX;
    let err = U256::try_from(i384).unwrap_err();
    assert_eq!(err, ParseU256Error::Overflow);
}
