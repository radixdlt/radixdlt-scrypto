#[cfg(test)]
use super::*;
use paste::paste;

#[cfg(test)]
use num_bigint::{BigInt, Sign};

use radix_engine_interface::*;

test_impl! {BnumI256, BnumI384, BnumI512, BnumI768, BnumU256, BnumU384, BnumU512, BnumU768}

test_add_all! {
    (BnumI256, BnumI384, BnumI512, BnumI768, BnumU256, BnumU384, BnumU512, BnumU768),
    (BnumI256, BnumI384, BnumI512, BnumI768, BnumU256, BnumU384, BnumU512, BnumU768)
}

test_signed! { BnumI256, BnumI384, BnumI512, BnumI768 }
test_unsigned! { BnumU256, BnumU384, BnumU512, BnumU768 }

test_from_all_types_safe_builtin! {BnumI256, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {BnumI256, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {BnumI384, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {BnumI384, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {BnumI512, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {BnumI512, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {BnumI768, (i8, i16, i32, i64, i128)}
test_from_all_types_safe_builtin! {BnumI768, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_builtin! {BnumU256, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {BnumU384, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {BnumU512, (u8, u16, u32, u64, u128)}
test_from_all_types_safe_builtin! {BnumU768, (u8, u16, u32, u64, u128)}

test_from_all_types_safe_safe! {BnumI256, (BnumI384, BnumI512, BnumI768)}
test_from_all_types_safe_safe! {BnumI384, (BnumI256, BnumI512, BnumI768)}
test_from_all_types_safe_safe! {BnumI512, (BnumI256, BnumI384, BnumI768)}
test_from_all_types_safe_safe! {BnumI768, (BnumI256, BnumI384, BnumI512)}

test_from_all_types_safe_safe! {BnumU256, (BnumU384, BnumU512, BnumU768)}
test_from_all_types_safe_safe! {BnumU384, (BnumU256, BnumU512, BnumU768)}
test_from_all_types_safe_safe! {BnumU512, (BnumU256, BnumU384, BnumU768)}
test_from_all_types_safe_safe! {BnumU768, (BnumU256, BnumU384, BnumU512)}

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
        256 BnumI256,
        384 BnumI384,
        512 BnumI512,
        768 BnumI768,
        256 BnumU256,
        384 BnumU384,
        512 BnumU512,
        768 BnumU768
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
                    assert_eq!(<$t>::from(17) + <$t>::from(31), <$t>::from(48));
                    let mut bnum = <$t>::ONE;
                    bnum += <$t>::from("101");
                    assert_eq!(bnum, <$t>::from("102"));

                    if <$t>::MIN < <$t>::ZERO {
                        let mut bnum = <$t>::MAX;
                        bnum += <$t>::from(-1);
                        assert_eq!(bnum, <$t>::MAX - <$t>::ONE);

                        assert_eq!(<$t>::MIN + <$t>::MAX, <$t>::ZERO - <$t>::ONE);
                    }
                }

                #[test]
                fn [< test_ $t:lower _sub >]() {
                    assert_eq!(<$t>::ONE - <$t>::ONE, <$t>::ZERO);

                    if <$t>::MIN < <$t>::ZERO {
                        assert_eq!(<$t>::from(17) - <$t>::from(31), <$t>::from(-14));
                        let mut bnum = <$t>::from(101);
                        bnum -= <$t>::from("102");
                        assert_eq!(bnum, <$t>::from("-1"));
                    }

                    let mut bnum = <$t>::MAX;
                    bnum -= <$t>::ONE;
                    assert_eq!(bnum, <$t>::MAX - <$t>::ONE);
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_ $t:lower _add_overflow_panic_1>] () {
                    let mut bnum = <$t>::MAX;
                    bnum += <$t>::from(1);
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _sub_overflow_panic_2 >]() {
                    let mut bnum = <$t>::MIN;
                    bnum -= <$t>::ONE;
                }

                #[test]
                fn [< test_ $t:lower _mul >]() {
                    assert_eq!(<$t>::from(4) * <$t>::from(5), <$t>::from(20));
                    let mut bnum = <$t>::from(12387);
                    bnum *= <$t>::from("1203203031");
                    assert_eq!(bnum, <$t>::from(14904075944997_i128));

                    if <$t>::MIN < <$t>::ZERO {
                        let mut bnum = <$t>::from(12387);
                        bnum *= <$t>::from("-1203203031");
                        assert_eq!(bnum, <$t>::from(-14904075944997_i128));
                    }
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _mul_overflow_panic_1 >] () {
                    let mut bnum = <$t>::MAX;
                    bnum *= <$t>::from(2);
                }


                #[test]
                fn [< test_ $t:lower _pow >](){
                    assert_eq!(<$t>::from(3).pow(3), <$t>::from(27));

                    assert_eq!(
                        <$t>::from(153).pow(20),
                        <$t>::from("49411565790213547262766437937260727785410401")
                    );
                    assert_eq!(
                        <$t>::from(153).pow(30),
                        <$t>::from("347330502405572936124071262363392351825462559418275421545603605649")
                    );
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [< test_ $t:lower _pow_overflow_panic_1 >]() {
                    if <$t>::BITS == 256 {
                        let _ = <$t>::from(153).pow(40);
                    } else if <$t>::BITS == 384 {
                        let _ = <$t>::from(153).pow(60);
                    } else if <$t>::BITS == 512 {
                        let _ = <$t>::from(153).pow(80);
                    } else if <$t>::BITS == 768 {
                        let _ = <$t>::from(153).pow(120);
                    } else {
                        panic!("Unknown bits size {}", <$t>::BITS);
                    }
                }

                #[test]
                fn [< test_ $t:lower _root >]() {
                    assert_eq!(<$t>::from(9).sqrt(), <$t>::from(3));
                    assert_eq!(<$t>::from(27).cbrt(), <$t>::from(3));

                    assert_eq!(<$t>::from(9).nth_root(2), <$t>::from(3));
                    assert_eq!(<$t>::from(27).nth_root(3), <$t>::from(3));
                    assert_eq!(<$t>::from(14966675814359580587845230627_i128).nth_root(13), <$t>::from(147));
                    assert_eq!(
                        <$t>::from("290437112829027226192310037731274304321654649956335616").nth_root(17),
                        <$t>::from("1396")
                    );

                    if <$t>::MIN < <$t>::ZERO {
                        assert_eq!(<$t>::from(-27).nth_root(3), <$t>::from(-3));
                        assert_eq!(<$t>::from(-14966675814359580587845230627_i128).nth_root(13), <$t>::from(-147));
                    }
                }

                #[test]
                fn [< test_ $t:lower _to_string >]() {
                    assert_eq!(<$t>::ONE.to_string(), "1");
                    assert_eq!(<$t>::ZERO.to_string(), "0");
                    assert_eq!(<$t>::from("0"), <$t>::ZERO);

                    if <$t>::MIN < <$t>::ZERO {
                        assert_eq!(<$t>::from(-1).to_string(), "-1");

                        assert_eq!(<$t>::from_str("-1").unwrap(), <$t>::from(-1));
                    }
                }

                #[test]
                fn [< test_ $t:lower _to_primitive_ints >] () {
                    assert_eq!(<$t>::from_i8(1).unwrap(), <$t>::ONE);
                    assert_eq!(<$t>::try_from(21).unwrap().to_string(), "21");
                    assert_eq!(<$t>::from(21_u8).to_string(), "21");

                    let bnum: $t = 21.into();
                    assert_eq!(bnum.to_string(), 21.to_string());

                    let i: i128 = <$t>::from(21_u8).into();
                    assert_eq!(i, 21_i128);
                }

                #[test]
                #[should_panic(expected = "Err")]
                fn [< test_ $t:lower _from_string_panic_1 >]() {
                    assert_eq!(<$t>::from_str("0x01").unwrap(), <$t>::from(-1));
                }

                #[test]
                #[should_panic(expected = "TryFromIntError")]
                fn [< test_ $t:lower _to_u128_panic >]() {
                    if <$t>::MIN < <$t>::ZERO {
                        let _u: u128 = <$t>::from(-21).into();
                    } else {
                        let _u: u128 = <$t>::from("290437112829027226192310037731274304321654649956335616").into();
                    }
                }
                #[test]
                #[should_panic(expected = "TryFromIntError")]
                fn [< test_ $t:lower _to_i8_panic >]() {
                    let _i: i8 = <$t>::from(-260).into();
                }

                #[test]
                #[should_panic(expected = "TryFromIntError")]
                fn [< test_ $t:lower _u16_panic >]() {
                    let _i: u16 = <$t>::from(123123123).into();
                }

            )*
        }
    }
}
#[cfg(test)]
test_bnums! { BnumI256, BnumI384, BnumI512, BnumI768, BnumU256, BnumU384, BnumU512, BnumU768 }

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
test_bnums_signed! { BnumI256, BnumI384, BnumI512, BnumI768 }

#[test]
#[should_panic(expected = "Err")]
fn test_string_to_bnum_panic_2() {
    assert_eq!(
        BnumI256::MAX,
        BnumI256::from(
            "578960446186580977117854925043439539266349923328202820197287920039565648199670"
        )
    );
}

#[test]
fn test_bnum_to_bigint() {
    assert_eq!(BnumI256::from(BigInt::from(147)), BnumI256::from(147));
    assert_eq!(BnumI256::from(BigInt::from(-147)), BnumI256::from(-147));
    assert_eq!(
        BnumI256::from(BigInt::from(1470198230918_i128)),
        BnumI256::from(1470198230918_i128)
    );
    assert_eq!(
        BnumI256::from(BigInt::from(-1470198230918_i128)),
        BnumI256::from(-1470198230918_i128)
    );

    assert_eq!(BigInt::from(BnumI256::from(123)), BigInt::from(123));
    assert_eq!(BigInt::from(BnumI256::ONE), BigInt::from(1));

    assert_eq!(
        BigInt::from(BnumI256::MAX),
        BigInt::from_str(
            "57896044618658097711785492504343953926634992332820282019728792003956564819967"
        )
        .unwrap()
    );

    assert_eq!(
        BigInt::from(BnumI256::MIN),
        BigInt::from_str(
            "-57896044618658097711785492504343953926634992332820282019728792003956564819968"
        )
        .unwrap()
    );
}

#[test]
fn test_bnum_to_bnum() {
    let a = BnumI256::from(1);
    let b = BnumU256::from(a);
    assert_eq!(b, BnumU256::ONE);

    let a = BnumI256::from(-123);
    let b = BnumI512::from(a);
    assert_eq!(a.to_string(), b.to_string());

    let a = BnumI256::MAX;
    let b = BnumU256::from(a);
    assert_eq!(a.to_string(), b.to_string());

    let a = BnumI256::MIN;
    let b = BnumI512::from(a);
    assert_eq!(a.to_string(), b.to_string());

    let a = BnumU256::MAX;
    let b = BnumI512::from(a);
    assert_eq!(a.to_string(), b.to_string());
}

#[test]
#[should_panic(expected = "Overflow")]
fn test_bnum_to_bnum_panic_1() {
    let i512 = BnumI512::MIN;
    let _ = BnumI256::from(i512);
}

#[test]
#[should_panic(expected = "Overflow")]
fn test_bnum_to_bnum_panic_2() {
    // I256::MAX + 1
    let i256_str = BnumI256::MAX.to_string();
    let i512 = BnumI512::from(i256_str) + BnumI512::ONE;
    let _ = BnumI256::from(i512);
}

#[test]
#[should_panic(expected = "Overflow")]
fn test_bnum_to_bnum_panic_3() {
    // I256::MIN - 1
    let i256_str = BnumI256::MIN.to_string();
    let i512 = BnumI512::from(i256_str) - BnumI512::ONE;
    let _ = BnumI256::try_from(i512).unwrap();
}

#[test]
#[should_panic(expected = "NegativeToUnsigned")]
fn test_bnum_to_bnum_panic_4() {
    let i512 = -BnumI512::ONE;
    let _ = BnumU256::try_from(i512).unwrap();
}

#[test]
#[should_panic(expected = "Overflow")]
fn test_bnum_to_bnum_panic_5() {
    let u256 = BnumU256::MAX;
    let _ = BnumI256::try_from(u256).unwrap();
}

#[test]
#[should_panic(expected = "Overflow")]
fn test_bnum_to_bnum_panic_6() {
    let i512_str = BnumI512::MAX.to_string();
    let u512 = BnumU512::from(i512_str) + BnumU512::ONE;
    let _ = BnumI512::try_from(u512).unwrap();
}

#[test]
#[should_panic(expected = "Overflow")]
fn test_bnum_to_bnum_panic_7() {
    let u512 = BnumU512::MAX;
    let _ = BnumI512::from(u512);
}
