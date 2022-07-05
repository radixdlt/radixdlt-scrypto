use super::*;
use num_traits::FromPrimitive;
use sbor::rust::str::FromStr;

macro_rules! test_from_builtin {
    ($i:ident, ($($t:ident),*)) => {
        paste! {
            $(
                #[test]
                fn [<from_builtin_$i:lower _ $t:lower>]() {
                    let b = <$i>::[<from_$t>](127).unwrap();
                    assert_eq!(b.to_string(), "127");
                }
            )*
        }
    };
}

macro_rules! test_impl {
    ($($i:ident),*) => ($(

            paste! {
                #[test]
                #[should_panic]
                fn [<test_add_overflow_$i:lower>]() {
                    let a = <$i>::MAX + <$i>::try_from(1u8).unwrap(); // panics on overflow
                    println!("{}.add({}) == {}", [<$i>]::MAX, 1, a);
                }

                #[test]
                #[should_panic]
                fn  [<test_sub_overflow_$i:lower>]() {
                    let _ = <$i>::MIN - <$i>::try_from(1u8).unwrap(); // panics on overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_mul_overflow_$i:lower>]() {
                    let _ = <$i>::MAX * <$i>::try_from(2u8).unwrap(); // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_div_overflow_$i:lower>]() {
                    let _ = <$i>::MIN / <$i>::try_from(0u8).unwrap(); // panics because of division by zero
                }

                #[test]
                #[should_panic]
                fn  [<test_rem_overflow_$i:lower>]() {
                    let _ = <$i>::MIN % $i::try_from(0u8).unwrap(); // panics because of division by zero
                }

                #[test]
                #[should_panic]
                fn  [<test_shl_overflow_$i:lower>]() {
                    let _ = <$i>::MAX << (<$i>::BITS + 1);  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_shr_overflow_$i:lower>]() {
                    let _ = <$i>::MAX >> (<$i>::BITS + 1);  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_shl_overflow_neg_$i:lower>]() {
                    let _ = <$i>::MIN << (<$i>::BITS + 1);  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_shr_overflow_neg_$i:lower>]() {
                    let _ = <$i>::MIN >> (<$i>::BITS + 1);  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_pow_overflow_$i:lower>]() {
                    let a = <$i>::MAX.pow(2u8);             // panics because of overflow
                    println!("{}.pow({}) == {}", [<$i>]::MAX, 2, a);
                }

                #[test]
                fn [<test_binary_$i:lower>]() {
                    let bin = <$i>::try_from(0x0b).unwrap();
                    assert_eq!(format!("{:b}", bin), "1011");
                }

                #[test]
                fn [<test_octal_$i:lower>]() {
                    let oct = <$i>::try_from(0x0b).unwrap();
                    assert_eq!(format!("{:o}", oct), "13");
                }

                #[test]
                fn [<test_hex_lower_$i:lower>]() {
                    let hex_lower = <$i>::try_from(0x0b).unwrap();
                    assert_eq!(format!("{:x}", hex_lower), "b");
                }

                #[test]
                fn [<test_hex_upper_$i:lower>]() {
                    let hex_upper = <$i>::try_from(0x0b).unwrap();
                    assert_eq!(format!("{:X}", hex_upper), "B");
                }

                #[test]
                fn [<test_zero_$i:lower>]() {
                    let zero = <$i>::try_from(0u8).unwrap();
                    assert_eq!(zero, <$i>::zero());
                }

                #[test]
                fn [<test_is_zero_$i:lower>]() {
                    let mut zero = <$i>::try_from(0u8).unwrap();
                    assert_eq!(zero.is_zero(), true);
                    zero = <$i>::try_from(1u8).unwrap();
                    assert_eq!(zero.is_zero(), false);
                }

                #[test]
                fn [<test_set_zero_$i:lower>]() {
                    let mut zero = <$i>::try_from(1u8).unwrap();
                    zero.set_zero();
                    assert_eq!(zero.is_zero(), true);
                }

                #[test]
                fn [<test_set_one_$i:lower>]() {
                    let mut zero = <$i>::try_from(0u8).unwrap();
                    zero.set_one();
                    assert_eq!(zero.is_one(), true);
                }


                test_from_builtin!{$i, (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)}

            }
    )*)
}
test_impl! { I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512 }

macro_rules! test_ops_output_type_builtin {
    ($i:literal, $i_bits:literal, $ops:ident, ($($t:literal, $t_bits:literal),*)) => {
        paste! {
            $(
                #[test]
                fn [<test_ $ops _output_type_ $i:lower $i_bits _ $t:lower $t_bits>]() {
                    test_ops_output_type_fn!($i, $i_bits, $ops, $t, $t_bits);
                }
            )*
        }
    };
}

macro_rules! test_ops_output_type {
    ($i:literal, $i_bits:literal, $ops:ident, ($($t:literal, $t_bits:literal),*)) => {
        paste! {
            $(
                #[test]
                fn [<test_ $ops _output_type_ $i:lower $i_bits _ $t:lower$t:lower $t_bits>]() {
                    test_ops_output_type_fn!($i, $i_bits, $ops, $t, $t_bits);
                }
            )*
        }
    };
}

macro_rules! test_ops_output_type_fn {
    ($i:literal, $i_bits:literal, $ops:ident, $t:literal, $t_bits:literal) => {
        paste! {
                {
                    let my_bits: usize = $i_bits;
                    let other_bits: usize = $t_bits;
                    let out_bits: usize = my_bits.max(other_bits);
                    let out_type_name = if $i == 'I' || $t == 'I' || $t == 'i' {
                        'I'
                    } else {
                        'U'
                    };
                    let a: [<$i $i_bits>] = [<$i $i_bits>]::from_str("2").unwrap();
                    let b: [<$t $t_bits>] = [<$t $t_bits>]::from_str("1").unwrap();
                    assert_eq!(core::any::type_name_of_val(&a.$ops(b)), format!("scrypto::math::integer::{}{}", out_type_name, out_bits));
                }
        }
    };
}

macro_rules! test_otput_type {
    ($i:literal, $ops:ident, ($($i_bits:literal),*)) => {
        $(

            test_ops_output_type!{ $i, $i_bits, $ops, ('I', 8, 'I', 16, 'I', 32, 'I', 64, 'I', 128, 'I', 256, 'I', 384, 'I', 512, 'U', 8, 'U', 16, 'U', 32, 'U', 64, 'U', 128, 'U', 256, 'U', 384, 'U', 512) }
            test_ops_output_type_builtin!{ $i, $i_bits, $ops, ('i', 8, 'i', 16, 'i', 32, 'i', 64, 'i', 128, 'u', 8, 'u', 16, 'u', 32, 'u', 64, 'u', 128) }
        )*
    };
}

macro_rules! test_otput_type_all {
    ($($ops:ident),*) => {
        $(
            test_otput_type! { 'I', $ops, (8, 16, 32, 64, 128, 256, 384, 512) }
            test_otput_type! { 'U', $ops, (8, 16, 32, 64, 128, 256, 384, 512) }
        )*
    };
}

test_otput_type_all! { add, sub, mul, div, rem }

macro_rules! test_ops_output_type_builtin_simple {
    ($i:literal, $i_bits:literal, $ops:ident, ($($t:literal, $t_bits:literal),*)) => {
        paste! {
            $(
                #[test]
                fn [<test_simple_ $ops _output_type_ $i:lower $i_bits _ $t:lower $t_bits>]() {
                    test_ops_output_type_simple_fn!($i, $i_bits, $ops, $t, $t_bits);
                }
            )*
        }
    };
}

macro_rules! test_ops_output_type_simple {
    ($i:literal, $i_bits:literal, $ops:ident, ($($t:literal, $t_bits:literal),*)) => {
        paste! {
            $(
                #[test]
                fn [<test_simple_ $ops _output_type_ $i:lower $i_bits _ $t:lower$t:lower $t_bits>]() {
                    test_ops_output_type_simple_fn!($i, $i_bits, $ops, $t, $t_bits);
                }

            )*
        }
    };
}

macro_rules! test_ops_output_type_simple_fn {
    ($i:literal, $i_bits:literal, $ops:ident, $t:literal, $t_bits:literal) => {
        paste! {
            let a: [<$i $i_bits>] = [<$i $i_bits>]::from_str("2").unwrap();
            let b: [<$t $t_bits>] = [<$t $t_bits>]::from_str("1").unwrap();
            assert_eq!(core::any::type_name_of_val(&a.$ops(b)), format!("scrypto::math::integer::{}{}", $i, $i_bits));
        }
    };
}

macro_rules! test_otput_type_simple {
    ($i:literal, $ops:ident, ($($i_bits:literal),*)) => {
        $(

            test_ops_output_type_simple!{ $i, $i_bits, $ops, ('I', 8, 'I', 16, 'I', 32, 'I', 64, 'I', 128, 'I', 256, 'I', 384, 'I', 512, 'U', 8, 'U', 16, 'U', 32, 'U', 64, 'U', 128, 'U', 256, 'U', 384, 'U', 512) }
            test_ops_output_type_builtin_simple!{ $i, $i_bits, $ops, ('i', 8, 'i', 16, 'i', 32, 'i', 64, 'i', 128, 'u', 8, 'u', 16, 'u', 32, 'u', 64, 'u', 128) }
        )*
    };
}

macro_rules! test_otput_type_all_simple {
    ($($ops:ident),*) => {
        $(
            test_otput_type_simple! { 'I', $ops, (8, 16, 32, 64, 128, 256, 384, 512) }
            test_otput_type_simple! { 'U', $ops, (8, 16, 32, 64, 128, 256, 384, 512) }
        )*
    };
}

test_otput_type_all_simple! { pow }

macro_rules! test_math {
    ($i:ident, $t:ident) => {
        paste! {
            #[test]
            fn [<test_2_add_2_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                const B_BYTES: usize = (<$t>::BITS / 8) as usize;
                const MAX_BYTES: usize = if A_BYTES > B_BYTES { A_BYTES } else { B_BYTES };
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[2u8; A_BYTES]).try_into().unwrap();
                let b: $t = BigInt::from_bytes_le(Sign::Plus, &[2u8; B_BYTES]).try_into().unwrap();
                let expect = BigInt::from_bytes_le(Sign::Plus, &[4u8; MAX_BYTES]);
                assert_eq!(a.add(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_0x88_add_0x88_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                const B_BYTES: usize = (<$t>::BITS / 8) as usize;
                const MAX_BYTES: usize = if A_BYTES > B_BYTES { A_BYTES } else { B_BYTES };
                let mut a_arr = [0x88u8; A_BYTES];
                let mut b_arr = [0x88u8; B_BYTES];
                let mut expect_arr = [0x11u8; MAX_BYTES];
                if A_BYTES == 1 {
                    a_arr[0] = 0x03;
                    b_arr[0] = 0x02;
                    expect_arr[0] = 0x05;
                } else {
                    a_arr[A_BYTES - 1] = 0x00;
                    b_arr[B_BYTES - 1] = 0x00;
                    expect_arr[MAX_BYTES - 1] = 0x01;
                    expect_arr[0] = 0x10;
                }
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &a_arr).try_into().unwrap();
                let b: $t = BigInt::from_bytes_le(Sign::Plus, &b_arr).try_into().unwrap();
                let expect = BigInt::from_bytes_le(Sign::Plus, &expect_arr);
                assert_eq!(a.add(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_2_sub_2_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                const B_BYTES: usize = (<$t>::BITS / 8) as usize;
                const MAX_BYTES: usize = if A_BYTES > B_BYTES { A_BYTES } else { B_BYTES };
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[2u8; A_BYTES]).try_into().unwrap();
                let b: $t = BigInt::from_bytes_le(Sign::Plus, &[2u8; B_BYTES]).try_into().unwrap();
                let expect = BigInt::from_bytes_le(Sign::Plus, &[0u8; MAX_BYTES]);
                assert_eq!(a.sub(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_2_mul_2_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                const B_BYTES: usize = (<$t>::BITS / 8) as usize;
                const MAX_BYTES: usize = if A_BYTES > B_BYTES { A_BYTES } else { B_BYTES };
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[2u8; A_BYTES]).try_into().unwrap();
                let b: $t = 2u8.try_into().unwrap();
                let expect = BigInt::from_bytes_le(Sign::Plus, &[4u8; MAX_BYTES]);
                assert_eq!(a.mul(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_2_div_2_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[4u8; A_BYTES]).try_into().unwrap();
                let b: $i = BigInt::from_bytes_le(Sign::Plus, &[4u8; A_BYTES]).try_into().unwrap();
                assert_eq!(a.div(b).to_string(), "1");
            }

            #[test]
            fn [<test_10_rem_8_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[10u8; A_BYTES]).try_into().unwrap();
                let b = 8u8;
                let expect = 2u8;
                assert_eq!(a.rem(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_10_pow_0_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[10u8; A_BYTES]).try_into().unwrap();
                let b = 0u8;
                let expect = 1u8;
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_0_pow_0_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0u8; A_BYTES]).try_into().unwrap();
                let b = 0u8;
                let expect = 1u8;
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_0_pow_1000_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0u8; A_BYTES]).try_into().unwrap();
                let b = 1000u32;
                let expect = 0u8;
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_10_pow_1_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[10u8; A_BYTES]).try_into().unwrap();
                let b = 1u8;
                let expect = a;
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_10_pow_2_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[107u8; A_BYTES / 2]).try_into().unwrap();
                let b = 2u8;
                let expect = a.mul(a);
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_5_pow_3_ $i:lower _ u8>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[5u8; A_BYTES / 3]).try_into().unwrap();
                let b = 3u8;
                let expect = a.mul(a).mul(a);
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_10_not_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i;
                let expect: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b10101010u8; A_BYTES]).try_into().unwrap();
                    expect = BigInt::from_bytes_le(Sign::Plus, &[0b01010101u8; A_BYTES]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b10101010u8; A_BYTES]).try_into().unwrap();
                    expect = BigInt::from_signed_bytes_le(&[0b01010101u8; A_BYTES]).try_into().unwrap();
                }
                assert_eq!(a.not(), expect);
            }

            #[test]
            fn [<test_bits_signed_ $i:lower >]() {
                let expect: String = String::from(stringify!($i));
                assert_eq!(<$i>::BITS, expect[1..].parse::<u32>().unwrap());
            }

            #[test]
            fn [<test_0b10101010_count_ones_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }

                assert_eq!(a.count_ones(), (<$i>::BITS / 2) as u32);
            }

            #[test]
            fn [<test_0_count_ones_ $i:lower >]() {
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0b00000000u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                assert_eq!(a.count_ones(), 0u32);
            }

            #[test]
            fn [<test_1_count_ones_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                a = BigInt::from_bytes_le(Sign::Plus, &[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }
                assert_eq!(a.count_ones(), (<$i>::BITS) as u32);
            }

            #[test]
            fn [<test_0b10101010_count_zeros_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }

                assert_eq!(a.count_zeros(), (<$i>::BITS / 2) as u32);
            }

            #[test]
            fn [<test_0_count_zeros_ $i:lower >]() {
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0b00000000u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                assert_eq!(a.count_zeros(), (<$i>::BITS) as u32);
            }

            #[test]
            fn [<test_1_count_zeros_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }
                assert_eq!(a.count_zeros(), 0u32);
            }
        }
    };
}

macro_rules! test_add_all {
    ($($i:ident),*) => {
        $(
            test_math!{$i, $i}
        )*
    };
}

test_add_all! { I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512}

macro_rules! test_signed {
    ($($i:ident),*) => {
        paste! {
            $(
                #[test]
                fn [<test_neg_ $i:lower>]() {
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &[10u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    let expect: $i = BigInt::from_bytes_le(Sign::Minus, &[10u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    assert_eq!(a.neg(), expect.try_into().unwrap());
                }

                #[test]
                fn [<test_8_abs_pos_ $i:lower>]() {
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &[8u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    let expect: $i = BigInt::from_bytes_le(Sign::Plus, &[8u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    assert_eq!(a.abs(), expect.try_into().unwrap());
                }

                #[test]
                fn [<test_8_abs_neg_ $i:lower>]() {
                    let a: $i = BigInt::from_bytes_le(Sign::NoSign, &[0u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    let expect: $i = BigInt::from_bytes_le(Sign::NoSign, &[0u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    assert_eq!(a.abs(), expect.try_into().unwrap());
                }

                #[test]
                fn [<test_minus1_signum_neg_ $i:lower>]() {
                    let a: $i = BigInt::from_bytes_le(Sign::Minus, &[13u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    let expect: $i = <$i>::from(-1i8);
                    assert_eq!(a.signum(), expect);
                }

                #[test]
                fn [<test_8_signum_0_ $i:lower>]() {
                    let a: $i = Zero::zero();
                    let expect: $i = Zero::zero();
                    assert_eq!(a.signum(), expect);
                }

                #[test]
                fn [<test_1_signum_0_ $i:lower>]() {
                    let a: $i = One::one();
                    let expect: $i = One::one();
                    assert_eq!(a.signum(), expect);
                }

                #[test]
                fn [<test_1_is_positive_ $i:lower>]() {
                    let a: $i = One::one();
                    let expect = true;
                    assert_eq!(a.is_positive(), expect);
                }

                #[test]
                fn [<test_0_is_positive_ $i:lower>]() {
                    let a: $i = Zero::zero();
                    let expect = false;
                    assert_eq!(a.is_positive(), expect);
                }

                #[test]
                fn [<test_minus1_is_positive_ $i:lower>]() {
                    let a: $i = <$i>::from(-1i8);
                    let expect = false;
                    assert_eq!(a.is_positive(), expect);
                }

                #[test]
                fn [<test_minus13_is_positive_ $i:lower>]() {
                    let a: $i = BigInt::from_bytes_le(Sign::Minus, &[13u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    let expect = false;
                    assert_eq!(a.is_positive(), expect);
                }

                #[test]
                fn [<test_1_is_negative_ $i:lower>]() {
                    let a: $i = One::one();
                    let expect = false;
                    assert_eq!(a.is_negative(), expect);
                }

                #[test]
                fn [<test_0_is_negative_ $i:lower>]() {
                    let a: $i = Zero::zero();
                    let expect = false;
                    assert_eq!(a.is_negative(), expect);
                }

                #[test]
                fn [<test_minus1_is_negative_ $i:lower>]() {
                    let a: $i = <$i>::from(-1i8);
                    let expect = true;
                    assert_eq!(a.is_negative(), expect);
                }

                #[test]
                fn [<test_minus13_is_negative_ $i:lower>]() {
                    let a: $i = BigInt::from_bytes_le(Sign::Minus, &[13u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                    let expect = true;
                    assert_eq!(a.is_negative(), expect);
                }

                #[test]
                fn [<test_min_signed_ $i:lower >]() {
                    let mut bytes = [0x00u8; (<$i>::BITS / 8) as usize];
                    bytes[bytes.len() - 1] = 0x80;
                    let expect: $i = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    assert_eq!(<$i>::MIN, expect);
                }

                #[test]
                fn [<test_max_signed_ $i:lower >]() {
                    let mut bytes = [0xffu8; (<$i>::BITS / 8) as usize];
                    bytes[bytes.len() - 1] = 0x7f;
                    let expect: $i = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    assert_eq!(<$i>::MAX, expect);
                }

            )*
        }
    };
}

test_signed! { I8, I16, I32, I64, I128, I256, I384, I512 }

macro_rules! test_unsigned {
    ($($i:ident),*) => {
        paste! {
            $(
                #[test]
                fn [<test_8_is_power_of_two_ $i:lower>]() {
                    let mut bytes = [0u8; (<$i>::BITS / 8) as usize];
                    bytes[bytes.len() - 1] = 8u8;
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    let expect = true;
                    assert_eq!(a.is_power_of_two(), expect);
                }

                #[test]
                fn [<test_3_is_power_of_two_3_ $i:lower>]() {
                    let mut bytes = [0u8; (<$i>::BITS / 8) as usize];
                    bytes[bytes.len() - 1] = 3;
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    let expect = false;
                    assert_eq!(a.is_power_of_two(), expect);
                }

                #[test]
                fn [<test_0xff_is_power_of_two_ $i:lower>]() {
                    let bytes = [0xffu8; (<$i>::BITS / 8) as usize];
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    let expect = false;
                    assert_eq!(a.is_power_of_two(), expect);
                }

                #[test]
                fn [<test_0_is_power_of_two_ $i:lower>]() {
                    let bytes = [0u8; (<$i>::BITS / 8) as usize];
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    let expect = false;
                    assert_eq!(a.is_power_of_two(), expect);
                }

                #[test]
                fn [<test_8_next_power_of_two_ $i:lower>]() {
                    let mut bytes = [0u8; (<$i>::BITS / 8) as usize];
                    bytes[bytes.len() - 1] = 0b100u8;
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    bytes[bytes.len() - 1] = 0b100u8;
                    let expect: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    assert_eq!(a.next_power_of_two(), expect);
                }

                #[test]
                fn [<test_0b01011111_next_power_of_two_ $i:lower>]() {
                    let mut bytes = [0b01011111u8; (<$i>::BITS / 8) as usize];
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    bytes = [0u8; (<$i>::BITS / 8) as usize];
                    bytes[bytes.len() - 1] = 0b10000000u8;
                    let expect: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    assert_eq!(a.next_power_of_two(), expect);
                }

                #[test]
                fn [<test_0_next_power_of_two_ $i:lower>]() {
                    let bytes = [0u8; (<$i>::BITS / 8) as usize];
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    let expect = <$i>::one();
                    assert_eq!(a.next_power_of_two(), expect);
                }

                #[test]
                #[should_panic]
                fn [<test_0b10000001_next_power_of_two_ $i:lower>]() {
                    let mut bytes = [0b10000000u8; (<$i>::BITS / 8) as usize];
                    if <$i>::BITS > 8 {
                        bytes[0] = 0b0000000001u8;
                    } else {
                        bytes[0] = 0b10000001u8;
                    }
                    let a: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    let _ = a.next_power_of_two();
                }

                #[test]
                fn [<test_min_unsigned_ $i:lower >]() {
                    let expect = <$i>::from(0u8);
                    assert_eq!(<$i>::MIN, expect);
                }

                #[test]
                fn [<test_max_unsigned_ $i:lower >]() {
                    let bytes = [0xffu8; (<$i>::BITS / 8) as usize];
                    let expect: $i = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    assert_eq!(<$i>::MAX, expect);
                }
            )*
        }
    };
}

test_unsigned! { U8, U16, U32, U64, U128, U256, U384, U512 }
