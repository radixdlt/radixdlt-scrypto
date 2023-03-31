#[macro_export]
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
#[macro_export]
#[cfg(test)]
macro_rules! from_builtin {
        ($a:expr, $i:ty, ($($t:ident),*)) => {
            paste!{
                $(
                    $a = <$i>::[<from_$t>]( 8 as $t ).unwrap();
                    assert_eq!($a.to_string(), "8");
                )*
            }
        }
}

#[macro_export]
#[cfg(test)]
macro_rules! try_from_safe {
        ($a:expr, $i:ty, ($($t:ident),*)) => {
            paste!{
                $(
                    $a = <$i>::try_from(<$t>::try_from(19u8).unwrap()).unwrap();
                    assert_eq!($a.to_string(), "19");
                )*
            }
        }
}

#[macro_export]
#[cfg(test)]
macro_rules! to_builtin {
        ($a:expr, $i:ty, ($($t:ident),*)) => {
            paste!{
                $(
                    let builtin: $t = $a.[<to_$t>]().unwrap();
                    assert_eq!(builtin.to_string(), "11");
                )*
            }
        }
}

#[macro_export]
macro_rules! test_impl {
    ($($i:ident),*) => ($(

            paste! {
                #[test]
                #[should_panic]
                fn [<test_add_overflow_$i:lower>]() {
                    let _ = <$i>::MAX + <$i>::try_from(1u8).unwrap(); // panics on overflow
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
                    let _ = <$i>::MAX << (<$i>::try_from(<$i>::BITS).unwrap() + <$i>::one());  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_shr_overflow_$i:lower>]() {
                    let _ = <$i>::MAX >> (<$i>::try_from(<$i>::BITS).unwrap() + <$i>::one());  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_shl_overflow_neg_$i:lower>]() {
                    let _ = <$i>::MIN << (<$i>::try_from(<$i>::BITS).unwrap() + <$i>::one());  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_shr_overflow_neg_$i:lower>]() {
                    let _ = <$i>::MIN >> (<$i>::try_from(<$i>::BITS).unwrap() + <$i>::one());  // panics because of overflow
                }

                #[test]
                #[should_panic]
                fn  [<test_pow_overflow_$i:lower>]() {
                    let _ = <$i>::MAX.pow(2u32);             // panics because of overflow
                }

                #[test]
                fn  [<test_max_to_string_$i:lower>]() {
                    let a = <$i>::MAX;
                    let b = BigInt::from(a);
                    assert_eq!(a.to_string(), b.to_string());
                }

                #[test]
                fn  [<test_min_to_string_$i:lower>]() {
                    let a = <$i>::MIN;
                    let b = BigInt::from(a);
                    assert_eq!(a.to_string(), b.to_string());
                }

//                #[test]
//                fn [<test_binary_$i:lower>]() {
//                    let bin = <$i>::try_from(0x0b).unwrap();
//                    assert_eq!(format!("{:b}", bin), "1011");
//                }
//
//                #[test]
//                fn [<test_octal_$i:lower>]() {
//                    let oct = <$i>::try_from(0x0b).unwrap();
//                    assert_eq!(format!("{:o}", oct), "13");
//                }
//
//                #[test]
//                fn [<test_hex_lower_$i:lower>]() {
//                    let hex_lower = <$i>::try_from(0x0b).unwrap();
//                    assert_eq!(format!("{:x}", hex_lower), "b");
//                }
//
//                #[test]
//                fn [<test_hex_upper_$i:lower>]() {
//                    let hex_upper = <$i>::try_from(0x0b).unwrap();
//                    assert_eq!(format!("{:X}", hex_upper), "B");
//                }
//
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

                #[test]
                fn [<test_ord_ $i:lower>]() {
                    let zero = <$i>::try_from(0u8).unwrap();
                    let one = <$i>::try_from(1u8).unwrap();
                    assert_eq!(zero.cmp(&one), Ordering::Less);
                }

                #[test]
                fn [<test_ord_5_1_ $i:lower>]() {
                    let five = <$i>::try_from(5u8).unwrap();
                    let one = <$i>::try_from(1u8).unwrap();
                    assert_eq!(five.cmp(&one), Ordering::Greater);
                }

                #[test]
                fn [<test_ord_5_5_ $i:lower>]() {
                    let five = <$i>::try_from(5u8).unwrap();
                    assert_eq!(five.cmp(&five), Ordering::Equal);
                }

                #[test]
                fn [<test_ord_min_min_ $i:lower>]() {
                    let min = <$i>::MIN;
                    assert_eq!(min.cmp(&min), Ordering::Equal);
                }

                #[test]
                fn [<test_ord_max_max_ $i:lower>]() {
                    let max = <$i>::MAX;
                    assert_eq!(max.cmp(&max), Ordering::Equal);
                }

                #[test]
                fn [<test_ord_max_min_ $i:lower>]() {
                    let max = <$i>::MAX;
                    let min = <$i>::MIN;
                    assert_eq!(max.cmp(&min), Ordering::Greater);
                }

                #[test]
                fn [<test_ord_min_max_ $i:lower>]() {
                    let max = <$i>::MAX;
                    let min = <$i>::MIN;
                    assert_eq!(min.cmp(&max), Ordering::Less);
                }

                test_from_builtin!{$i, (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)}

            }
    )*)
}

#[macro_export]
macro_rules! test_math {
    ($i:ident, $t:ident, $tlst:tt) => {
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
                let b: $i = 8u8.try_into().unwrap();
                let expect = 2u8;
                assert_eq!(a.rem(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_10_pow_0_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[10u8; A_BYTES]).try_into().unwrap();
                let b = 0u32;
                let expect = 1u8;
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_0_pow_0_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0u8; A_BYTES]).try_into().unwrap();
                let b = 0u32;
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
            fn [<test_10x_pow_1_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[10u8; A_BYTES]).try_into().unwrap();
                let b = 1u32;
                let expect = a;
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_10x_pow_2_ $i:lower _ $t:lower>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[107u8; A_BYTES / 2]).try_into().unwrap();
                let b = 2u32;
                let expect = a.mul(a);
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_5x_pow_3_ $i:lower _ u8>]() {
                const A_BYTES: usize = (<$i>::BITS / 8) as usize;
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[5u8; A_BYTES / 3]).try_into().unwrap();
                let b = 3u32;
                let expect = a.mul(a).mul(a);
                assert_eq!(a.pow(b), expect.try_into().unwrap());
            }

            #[test]
            fn [<test_bits_10_not_ $i:lower _ $t:lower>]() {
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
            fn [<test_bits_bits_signed_ $i:lower >]() {
                let expect: String = String::from(stringify!($i));
                let mut i = 0;
                for (idx, c) in expect.chars().enumerate() {
                    if c.is_numeric() {
                        i = idx;
                        break;
                    }
                }
                assert_eq!(<$i>::BITS, expect[i..].parse::<u32>().unwrap());
            }

            #[test]
            fn [<test_bits_0b10101010_count_ones_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }

                assert_eq!(a.count_ones(), (<$i>::BITS / 2) as u32);
            }

            #[test]
            fn [<test_bits_0_count_ones_ $i:lower >]() {
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0b00000000u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                assert_eq!(a.count_ones(), 0u32);
            }

            #[test]
            fn [<test_bits_1_count_ones_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                a = BigInt::from_bytes_le(Sign::Plus, &[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }
                assert_eq!(a.count_ones(), (<$i>::BITS) as u32);
            }

            #[test]
            fn [<test_bits_0b10101010_count_zeros_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b01101010u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }

                assert_eq!(a.count_zeros(), (<$i>::BITS / 2) as u32);
            }

            #[test]
            fn [<test_bits_0_count_zeros_ $i:lower >]() {
                let a: $i = BigInt::from_bytes_le(Sign::Plus, &[0b00000000u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                assert_eq!(a.count_zeros(), (<$i>::BITS) as u32);
            }

            #[test]
            fn [<test_bits_1_count_zeros_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }
                assert_eq!(a.count_zeros(), 0u32);
            }

            #[test]
            fn [<test_bits_0_trailing_zeros_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b00000000u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b00000000u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }
                assert_eq!(a.trailing_zeros(), (<$i>::BITS) as u32);
            }


            #[test]
            fn [<test_bits_1_trailing_zeros_ $i:lower >]() {
                let a: $i;
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&[0b11111111u8; (<$i>::BITS / 8) as usize]).try_into().unwrap();
                }
                assert_eq!(a.trailing_zeros(), 0u32);
            }

            #[test]
            fn [<test_bits_all_trailing_zeros_ $i:lower >]() {
                let mut a: $i;
                let bytes: [u8; (<$i>::BITS / 8) as usize] = [0b01010101u8; (<$i>::BITS / 8) as usize];
                for i in 0..<$i>::BITS {
                    if <$i>::MIN == Zero::zero() {
                        a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    } else {
                        a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    }
                    a <<= i.try_into().unwrap();
                    assert_eq!(a.trailing_zeros(), i as u32);
                }
            }

            #[test]
            fn [<test_bits_swap_bytes_ $i:lower >]() {
                let a: $i;
                let exp: $i;
                let mut bytes: [u8; (<$i>::BITS / 8) as usize] = [0b00000000u8; (<$i>::BITS / 8) as usize];
                let mut expect: [u8; (<$i>::BITS / 8) as usize] = [0b00000000u8; (<$i>::BITS / 8) as usize];
                for i in 0..(<$i>::BITS / 8) as usize {
                    bytes[i] = i as u8;
                    expect[i] = (<$i>::BITS / 8 - i as u32 - 1) as u8;
                }
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    exp = BigInt::from_bytes_le(Sign::Plus, &expect).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    exp = BigInt::from_signed_bytes_le(&expect).try_into().unwrap();
                }
                assert_eq!(a.swap_bytes(), exp);
            }

            #[test]
            fn [<test_bits_reverse_bits_ $i:lower >]() {
                let a: $i;
                let exp: $i;
                const LEN: usize = (<$i>::BITS / 8) as usize;
                let mut bytes: [u8; LEN] = [0b00001111u8; LEN];
                let mut expect: [u8; LEN] = [0b11110000u8; LEN];
                for i in 0..LEN / 2 {
                    bytes[i] = 0b01010101 as u8;
                    expect[LEN - i - 1] = 0b10101010 as u8;
                }
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    exp = BigInt::from_bytes_le(Sign::Plus, &expect).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    exp = BigInt::from_signed_bytes_le(&expect).try_into().unwrap();
                }
                assert_eq!(a.reverse_bits(), exp);
            }

            #[test]
            fn [<test_bits_zero_leading_zeros_ $i:lower >]() {
                let a: $i;
                let bytes: [u8; (<$i>::BITS / 8) as usize] = [0b11010101u8; (<$i>::BITS / 8) as usize];
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                }
                assert_eq!(a.leading_zeros(), 0);
            }

            #[test]
            fn [<test_bits_all_leading_zeros_ $i:lower >]() {
                let mut a: $i;
                let bytes: [u8; (<$i>::BITS / 8) as usize] = [0b01010101u8; (<$i>::BITS / 8) as usize];
                for i in 0..<$i>::BITS {
                    if <$i>::MIN == Zero::zero() {
                        a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    } else {
                        a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    }
                    a >>= i.try_into().unwrap();
                    assert_eq!(a.leading_zeros(), i as u32 + 1);
                }
            }

            #[test]
            fn [<test_bits_bitxor_ $i:lower >]() {
                let mut a: $i;
                let b: $i;
                let exp: $i;
                const LEN: usize = (<$i>::BITS / 8) as usize;
                let bytes: [u8; LEN] =    [0b11001100u8; LEN];
                let bytes_b: [u8; LEN] =  [0b10101010u8; LEN];
                let expected: [u8; LEN] = [0b01100110u8; LEN];
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    b = BigInt::from_bytes_le(Sign::Plus, &bytes_b).try_into().unwrap();
                    exp = BigInt::from_bytes_le(Sign::Plus, &expected).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    b = BigInt::from_signed_bytes_le(&bytes_b).try_into().unwrap();
                    exp = BigInt::from_signed_bytes_le(&expected).try_into().unwrap();
                }
                assert_eq!(a.bitxor(b), exp);
                a ^= b;
                assert_eq!(a, exp);
            }

            #[test]
            fn [<test_bits_bitor_ $i:lower >]() {
                let mut a: $i;
                let b: $i;
                let exp: $i;
                const LEN: usize = (<$i>::BITS / 8) as usize;
                let bytes: [u8; LEN] =    [0b11001100u8; LEN];
                let bytes_b: [u8; LEN] =  [0b10101010u8; LEN];
                let expected: [u8; LEN] = [0b11101110u8; LEN];
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    b = BigInt::from_bytes_le(Sign::Plus, &bytes_b).try_into().unwrap();
                    exp = BigInt::from_bytes_le(Sign::Plus, &expected).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    b = BigInt::from_signed_bytes_le(&bytes_b).try_into().unwrap();
                    exp = BigInt::from_signed_bytes_le(&expected).try_into().unwrap();
                }
                assert_eq!(a.bitor(b), exp);
                a |= b;
                assert_eq!(a, exp);
            }


            #[test]
            fn [<test_bits_bitand_ $i:lower >]() {
                let mut a: $i;
                let b: $i;
                let exp: $i;
                const LEN: usize = (<$i>::BITS / 8) as usize;
                let bytes: [u8; LEN] =    [0b11001100u8; LEN];
                let bytes_b: [u8; LEN] =  [0b10101010u8; LEN];
                let expected: [u8; LEN] = [0b10001000u8; LEN];
                if <$i>::MIN == Zero::zero() {
                    a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    b = BigInt::from_bytes_le(Sign::Plus, &bytes_b).try_into().unwrap();
                    exp = BigInt::from_bytes_le(Sign::Plus, &expected).try_into().unwrap();
                } else {
                    a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    b = BigInt::from_signed_bytes_le(&bytes_b).try_into().unwrap();
                    exp = BigInt::from_signed_bytes_le(&expected).try_into().unwrap();
                }
                assert_eq!(a.bitand(b), exp);
                a &= b;
                assert_eq!(a, exp);
            }

            #[test]
            fn [<test_bits_shl_shr_combined_ $i:lower >]() {
                let mut a: $i;
                let mut b: $i;
                let mut shift: $i;
                let bytes: [u8; (<$i>::BITS / 8) as usize] = [0b01010101u8; (<$i>::BITS / 8) as usize];
                let bits: u32 = if <$i>::MIN == Zero::zero() {
                    <$i>::BITS
                } else {
                    <$i>::BITS - 1
                };
                for i in 0..bits {
                    if <$i>::MIN == Zero::zero() {
                        a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    } else {
                        a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    }
                    b = a;
                    a >>= i.try_into().unwrap();
                    a <<= i.try_into().unwrap();
                    shift = <$i>::try_from(2).unwrap().pow(i as u32);
                    assert_eq!(a, b / shift * shift);
                }
            }


            #[test]
            fn [<test_bits_shl_ $i:lower >]() {
                let mut a: $i;
                let mut expect: BigInt;
                let bytes: [u8; (<$i>::BITS / 8) as usize] = [0b01010101u8; (<$i>::BITS / 8) as usize];
                let bits: u32 = if <$i>::MIN == Zero::zero() {
                    <$i>::BITS
                } else {
                    <$i>::BITS - 1
                };
                for i in 0..bits {
                    if <$i>::MIN == Zero::zero() {
                        a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    } else {
                        a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    }
                    expect = a.into();
                    expect <<= i;
                    a <<= i.try_into().unwrap();
                    let mut expect_bytes = expect.to_signed_bytes_le();
                    expect_bytes.resize((<$i>::BITS / 8) as usize, 0);
                    println!("expect_bytes= {:?} len={}", expect_bytes, expect_bytes.len());
                    assert_eq!(a, <$i>::from_le_bytes(expect_bytes.as_slice().try_into().unwrap()));
                }
            }

            #[test]
            fn [<test_bits_shr_ $i:lower >]() {
                let mut a: $i;
                let mut expect: BigInt;
                let bytes: [u8; (<$i>::BITS / 8) as usize] = [0b01010101u8; (<$i>::BITS / 8) as usize];
                let bits: u32 = if <$i>::MIN == Zero::zero() {
                    <$i>::BITS
                } else {
                    <$i>::BITS - 1
                };
                for i in 0..bits {
                    if <$i>::MIN == Zero::zero() {
                        a = BigInt::from_bytes_le(Sign::Plus, &bytes).try_into().unwrap();
                    } else {
                        a = BigInt::from_signed_bytes_le(&bytes).try_into().unwrap();
                    }
                    expect = a.into();
                    expect >>= i;
                    a >>= i.try_into().unwrap();
                    assert_eq!(a, expect.try_into().unwrap());
                }
            }

            #[test]
            fn [<test_try_from_builtin_ $i:lower>]() {
               let mut a: $i;
               from_builtin!{a, $i, (i8, u8, i16, u16, i32, u32, i64, u64, i128, u128)}
            }

            #[test]
            fn [<test_try_to_builtin_ $i:lower>]() {
               let a: $i = <$i>::try_from(11u8).unwrap();
               to_builtin!{a, $i, (i8, u8, i16, u16, i32, u32, i64, u64, i128, u128)}
            }

            #[test]
            fn [<test_try_from_safe_ $i:lower>]() {
               let mut a: $i;
               try_from_safe!{a, $i, $tlst}
            }

            #[test]
            #[should_panic]
            fn [<test_try_from_panic_ $i:lower>]() {
                let mut expect: BigInt = 1.into();
                expect = expect.shl(<$i>::BITS as u32);
                let _:$i = expect.try_into().unwrap();
            }

            #[test]
            fn [<test_try_from_bigint_ $i:lower>]() {
                let mut a: $i;
                const LEN: usize = (<$i>::BITS / 8) as usize;
                let mut expect: BigInt = BigInt::from_signed_bytes_le(&[78u8; LEN]);
                let bits: u32 = <$i>::BITS as u32;
                for _ in 0..bits {
                    a = expect.clone().try_into().unwrap();
                    assert_eq!(a.to_string(), expect.clone().to_string());
                    expect >>= 1;
                }
            }

            #[test]
            fn [<test_try_from_bigint_negative_ $i:lower>]() {
                let mut a: $i;
                const LEN: usize = (<$i>::BITS / 8) as usize;
                let mut expect: BigInt;
                if <$i>::MIN < Zero::zero() {
                    expect = BigInt::from_signed_bytes_le(&[0x81u8; LEN]);
                } else {
                    // we do not thest negative numbers on unsigned types
                    expect = BigInt::from_bytes_le(Sign::Plus, &[0x81u8; LEN]);
                }
                let bits: u32 = <$i>::BITS as u32;
                for _ in 0..bits {
                    if expect.clone().to_signed_bytes_le().len() > LEN {
                        expect = expect.clone().shr((expect.clone().to_signed_bytes_le().len() - LEN) * 8);
                    }
                    a = <$i>::try_from(expect.clone()).unwrap();
                    assert_eq!(a.to_string(), expect.clone().to_string());
                    expect >>= 1;
                }
            }

            #[test]
            fn [<test_try_from_vector_ $i:lower>]() {
                let mut a: $i;
                let mut expect: BigInt;
                let bits: usize = <$i>::BITS as usize;
                for bytes in 0..bits/8 {
                    expect = BigInt::from_signed_bytes_le(&vec![78u8; bytes]);
                    a = <$i>::try_from(vec![78u8; bytes]).unwrap();
                    assert_eq!(a.to_string(), expect.clone().to_string());
                }
            }

            #[test]
            fn [<test_try_from_slice_ $i:lower>]() {
                let mut a: $i;
                let mut expect: BigInt;
                let bits: usize = <$i>::BITS as usize;
                let mut slice: &[u8];
                let mut vec: Vec<u8>;
                for bytes in 0..bits/8 {
                    vec = vec![78u8; bytes];
                    slice = &vec[..];
                    expect = BigInt::from_signed_bytes_le(&slice);
                    a = <$i>::try_from(slice).unwrap();
                    assert_eq!(a.to_string(), expect.clone().to_string());
                }
            }

            #[test]
            fn [<test_from_string_ $i:lower>]() {
                let mut a: $i = <$i>::from_str("118").unwrap();
                assert_eq!(a.to_string(), "118");
                let b: String = a.to_string();
                a = <$i>::from_str(&b).unwrap();
                assert_eq!(a.to_string(), "118");
                let c: &str = &a.to_string();
                a = <$i>::from_str(c).unwrap();
                assert_eq!(a.to_string(), "118");
            }

            #[test]
            fn [<test_bigint_from_ $i:lower>]() {
                let a: $i = 119u8.try_into().unwrap();
                let expect: BigInt = BigInt::from(a);
                assert_eq!(a.to_string(), expect.to_string());
            }
        }
    };
}

#[macro_export]
macro_rules! test_add_all {
    (($($i:ident),*), $tlst:tt) => {
        $(
            test_math!{$i, $i, $tlst}
        )*
    };
}

#[macro_export]
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

                #[test]
                fn [<test_ord_neg_ $i:lower>]() {
                    let zero = <$i>::try_from(0i8).unwrap();
                    let minus_one = <$i>::try_from(-1i8).unwrap();
                    assert_eq!(zero.cmp(&minus_one), Ordering::Greater);
                }
            )*
        }
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! test_from_all_types_builtin_safe {
    ($i:ty, ($($from:ty),*)) => {
        paste!{
        $(
            #[test]
            fn [<test_from_builtin_ $i:lower _from_safe_ $from:lower>]() {
                let a: $i = <$i>::from(<$from>::try_from(112u8).unwrap());
                let expect: $i = <$i>::try_from(112u8).unwrap();
                assert_eq!(a, expect);
            }
        )*
        }
    };
}

#[macro_export]
macro_rules! test_from_all_types_safe_builtin {
    ($i:ty, ($($from:ty),*)) => {
        paste!{
        $(
            #[test]
            fn [<test_from_safe_ $i:lower _from _builtin_ $from:lower>]() {
                let a: $i = <$i>::from(<$from>::try_from(112u8).unwrap());
                let expect: $i = <$i>::try_from(112u8).unwrap();
                assert_eq!(a, expect);
            }
        )*
        }
    };
}

#[macro_export]
macro_rules! test_from_all_types_safe_safe {
    ($i:ty, ($($from:ty),*)) => {
        paste!{
        $(
            #[test]
            fn [<test_from_safe_ $i:lower _from_safe_ $from:lower>]() {
                let a: $i = <$i>::try_from(
                        <$from>::try_from(112u8).unwrap()
                    ).unwrap();
                let expect: $i = <$i>::try_from(112u8).unwrap();
                assert_eq!(a, expect);
            }
        )*
        }
    };
}
