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
                    test_ops_output_type_fn!($i, $i_bits, $ops, $t, $t_bits);
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
            let my_bits: usize = $i_bits;
            let out_bits: usize = my_bits;
            let out_type_name = $i;
            let a: [<$i $i_bits>] = [<$i $i_bits>]::from_str("2").unwrap();
            let b: [<$t $t_bits>] = [<$t $t_bits>]::from_str("1").unwrap();
            assert_eq!(core::any::type_name_of_val(&a.$ops(b)), format!("scrypto::math::integer::{}{}", out_type_name, out_bits));
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
