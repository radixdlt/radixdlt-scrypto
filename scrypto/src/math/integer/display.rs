use super::*;
use core::ptr;
use core::slice;
use core::str;
use maybe_uninit::MaybeUninit;

mod maybe_uninit;

// BEGIN: Taken from core::num::fmt with no changes
// as this is an unstable feature, but can not hurt Scrypto if we use it
// anyway.
/// Formatted parts.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Part<'a> {
    /// Given number of zero digits.
    Zero(usize),
    /// A literal number up to 5 digits.
    Num(u16),
    /// A verbatim copy of given bytes.
    Copy(&'a [u8]),
}

impl<'a> Part<'a> {
    /// Returns the exact byte length of given part.
    pub fn len(&self) -> usize {
        match *self {
            Part::Zero(nzeroes) => nzeroes,
            Part::Num(v) => {
                if v < 1_000 {
                    if v < 10 {
                        1
                    } else if v < 100 {
                        2
                    } else {
                        3
                    }
                } else {
                    if v < 10_000 { 4 } else { 5 }
                }
            }
            Part::Copy(buf) => buf.len(),
        }
    }

    /// Writes a part into the supplied buffer.
    /// Returns the number of written bytes, or `None` if the buffer is not enough.
    /// (It may still leave partially written bytes in the buffer; do not rely on that.)
    pub fn write(&self, out: &mut [u8]) -> Option<usize> {
        let len = self.len();
        if out.len() >= len {
            match *self {
                Part::Zero(nzeroes) => {
                    for c in &mut out[..nzeroes] {
                        *c = b'0';
                    }
                }
                Part::Num(mut v) => {
                    for c in out[..len].iter_mut().rev() {
                        *c = b'0' + (v % 10) as u8;
                        v /= 10;
                    }
                }
                Part::Copy(buf) => {
                    out[..buf.len()].copy_from_slice(buf);
                }
            }
            Some(len)
        } else {
            None
        }
    }
}

/// Formatted result containing one or more parts.
/// This can be written to the byte buffer or converted to the allocated string.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Formatted<'a> {
    /// A byte slice representing a sign, either `""`, `"-"` or `"+"`.
    pub sign: &'static str,
    /// Formatted parts to be rendered after a sign and optional zero padding.
    pub parts: &'a [Part<'a>],
}

impl<'a> Formatted<'a> {
    /// Returns the exact byte length of combined formatted result.
    pub fn len(&self) -> usize {
        let mut len = self.sign.len();
        for part in self.parts {
            len += part.len();
        }
        len
    }

    /// Writes all formatted parts into the supplied buffer.
    /// Returns the number of written bytes, or `None` if the buffer is not enough.
    /// (It may still leave partially written bytes in the buffer; do not rely on that.)
    pub fn write(&self, out: &mut [u8]) -> Option<usize> {
        if out.len() < self.sign.len() {
            return None;
        }
        out[..self.sign.len()].copy_from_slice(self.sign.as_bytes());

        let mut written = self.sign.len();
        for part in self.parts {
            let len = part.write(&mut out[written..])?;
            written += len;
        }
        Some(written)
    }
}
// END: Taken from core::num::fmt with no changes
// as this is an unstable feature, but can not hurt Scrypto if we use it
// anyway.

// 2 digit decimal look up table
static DEC_DIGITS_LUT: &[u8; 200] = b"0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

// Code below is taken from 
// https://doc.rus t-lang.org/src/core/fmt/num.rs.html#209
// Only necessary changes were made to reflect differences in data size and syntax
macro_rules! impl_Display {
    ($($t:ident),* as $u:ident via $conv_fn:ident named $name:ident) => {
        fn $name(mut n: $u, is_nonnegative: bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // 2^512 is about 1.34*10^154, so 155 gives an extra byte of space
            let mut buf = [MaybeUninit::<u8>::uninit(); 155];
            let mut curr = buf.len() as isize;
            let buf_ptr = MaybeUninit::slice_as_mut_ptr(&mut buf);
            let lut_ptr = DEC_DIGITS_LUT.as_ptr();

            // SAFETY: Since `d1` and `d2` are always less than or equal to `198`, we
            // can copy from `lut_ptr[d1..d1 + 1]` and `lut_ptr[d2..d2 + 1]`. To show
            // that it's OK to copy into `buf_ptr`, notice that at the beginning
            // `curr == buf.len() == 155 > log(n)` since `n < 2^512 < 10^155`, and at
            // each step this is kept the same as `n` is divided. Since `n` is always
            // non-negative, this means that `curr > 0` so `buf_ptr[curr..curr + 1]`
            // is safe to access.
            unsafe {
                // need at least 16 bits for the 4-characters-at-a-time to work.
                assert!(core::mem::size_of::<$u>() >= 2);

                // eagerly decode 4 characters at a time
                let u_10000 = <$u>::try_from(10000u64).unwrap();
                while n >= u_10000 {
                    let rem = (n % u_10000).to_isize().unwrap();
                    n /= u_10000;

                    let d1 = (rem / 100) << 1;
                    let d2 = (rem % 100) << 1;
                    curr -= 4;

                    // We are allowed to copy to `buf_ptr[curr..curr + 3]` here since
                    // otherwise `curr < 0`. But then `n` was originally at least `10000^10`
                    // which is `10^155 > 2^512 > n`.
                    ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                    ptr::copy_nonoverlapping(lut_ptr.offset(d2), buf_ptr.offset(curr + 2), 2);
                }

                // if we reach here numbers are <= 9999, so at most 4 chars long
//              let mut n = n as isize; // possibly reduce 64bit math

                // decode 2 more chars, if > 2 chars
                let hundred: $u = <$u>::try_from(100u8).unwrap();
                if n >= hundred {
                    let d1 = (n % hundred) << <$u>::one();
                    n /= hundred;
                    curr -= 2;
                    ptr::copy_nonoverlapping(lut_ptr.offset(d1.to_isize().unwrap()), buf_ptr.offset(curr), 2);
                }

                // decode last 1 or 2 chars
                if n < <$u>::try_from(10u8).unwrap() {
                    curr -= 1;
                    *buf_ptr.offset(curr) = (n.to_u8().unwrap()) + b'0';
                } else {
                    let d1 = n << <$u>::one();
                    curr -= 2;
                    ptr::copy_nonoverlapping(lut_ptr.offset(d1.to_isize().unwrap()), buf_ptr.offset(curr), 2);
                }
            }

            // SAFETY: `curr` > 0 (since we made `buf` large enough), and all the chars are valid
            // UTF-8 since `DEC_DIGITS_LUT` is
            let buf_slice = unsafe {
                str::from_utf8_unchecked(
                    slice::from_raw_parts(buf_ptr.offset(curr), buf.len() - curr as usize))
            };
            f.pad_integral(is_nonnegative, "", buf_slice)
        }

//      #[stable(feature = "rust1", since = "1.0.0")]
        $(
        impl fmt::Display for $t {
            #[allow(unused_comparisons)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let is_nonnegative = *self >= <$t>::zero();
                    let n: $u = if is_nonnegative {
                        let a: $u = (*self).try_into().unwrap();
                        a
                    } else {
                        let neg: $u = (*self).try_into().unwrap();
                        <$u>::zero() - neg 
                    };
                $name(n, is_nonnegative, f)
            }
        })*
    };
}

macro_rules! impl_Exp {
    ($($t:ident),* as $u:ident via $conv_fn:ident named $name:ident) => {
        fn $name(
            mut n: $u,
            is_nonnegative: bool,
            upper: bool,
            f: &mut fmt::Formatter<'_>
        ) -> fmt::Result {
            let (mut n, mut exponent, trailing_zeros, added_precision) = {
                let ten: $u = <$u>::try_from(10u8).unwrap();
                let mut exponent = 0;
                // count and remove trailing decimal zeroes
                while n % ten == <$u>::zero() && n >= ten {
                    n /= ten;
                    exponent += 1;
                }

                let (added_precision, subtracted_precision) = match f.precision() {
                    Some(fmt_prec) => {
                        // number of decimal digits minus 1
                        let mut tmp = n;
                        let mut prec = 0;
                        while tmp >= ten {
                            tmp /= ten;
                            prec += 1;
                        }
                        (fmt_prec.saturating_sub(prec), prec.saturating_sub(fmt_prec))
                    }
                    None => (0, 0)
                };
                for _ in 1..subtracted_precision {
                    n /= ten;
                    exponent += 1;
                }
                let five: $u = <$u>::try_from(5).unwrap();
                if subtracted_precision != 0 {
                    let rem = n % ten;
                    n /= ten;
                    exponent += 1;
                    // round up last digit
                    if rem >= five {
                        n += <$u>::one();
                    }
                }
                (n, exponent, exponent, added_precision)
            };

            // 155 digits (worst case u128) + . = 156
            // Since `curr` always decreases by the number of digits copied, this means
            // that `curr >= 0`.
            let mut buf = [MaybeUninit::<u8>::uninit(); 156];
            let mut curr = buf.len() as isize; //index for buf
            let buf_ptr = MaybeUninit::slice_as_mut_ptr(&mut buf);
            let lut_ptr = DEC_DIGITS_LUT.as_ptr();

            // decode 2 chars at a time
            let hundred: $u = <$u>::try_from(100u8).unwrap();
            while n >= hundred {
                let d1 = ((n % hundred).to_isize().unwrap()) << 1;
                curr -= 2;
                // SAFETY: `d1 <= 198`, so we can copy from `lut_ptr[d1..d1 + 2]` since
                // `DEC_DIGITS_LUT` has a length of 200.
                unsafe {
                    ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                }
                n /= hundred;
                exponent += 2;
            }
            // n is <= 99, so at most 2 chars long
//          let mut n = n as isize; // possibly reduce 64bit math
            // decode second-to-last character
            let ten: $u = <$u>::try_from(10u8).unwrap();
            if n >= ten {
                curr -= 1;
                // SAFETY: Safe since `156 > curr >= 0` (see comment)
                unsafe {
                    *buf_ptr.offset(curr) = (n.to_u8().unwrap() % 10_u8) + b'0';
                }
                n /= ten;
                exponent += 1;
            }
            // add decimal point iff >1 mantissa digit will be printed
            if exponent != trailing_zeros || added_precision != 0 {
                curr -= 1;
                // SAFETY: Safe since `156 > curr >= 0`
                unsafe {
                    *buf_ptr.offset(curr) = b'.';
                }
            }

            // SAFETY: Safe since `156 > curr >= 0`
            let buf_slice = unsafe {
                // decode last character
                curr -= 1;
                *buf_ptr.offset(curr) = (n.to_u8().unwrap()) + b'0';

                let len = buf.len() - curr as usize;
                slice::from_raw_parts(buf_ptr.offset(curr), len)
            };

            // stores 'e' (or 'E') and the up to 2-digit exponent
            let mut exp_buf = [MaybeUninit::<u8>::uninit(); 3];
            let exp_ptr = MaybeUninit::slice_as_mut_ptr(&mut exp_buf);
            // SAFETY: In either case, `exp_buf` is written within bounds and `exp_ptr[..len]`
            // is contained within `exp_buf` since `len <= 3`.
            let exp_slice = unsafe {
                *exp_ptr.offset(0) = if upper { b'E' } else { b'e' };
                let len = if exponent < 10 {
                    *exp_ptr.offset(1) = (exponent as u8) + b'0';
                    2
                } else {
                    let off = exponent << 1;
                    ptr::copy_nonoverlapping(lut_ptr.offset(off), exp_ptr.offset(1), 2);
                    3
                };
                slice::from_raw_parts(exp_ptr, len)
            };

            let parts = &[
                Part::Copy(buf_slice),
                Part::Zero(added_precision),
                Part::Copy(exp_slice)
            ];
            let sign = if !is_nonnegative {
                "-"
            } else if f.sign_plus() {
                "+"
            } else {
                ""
            };
            let formatted = Formatted{sign, parts};
            f.pad_formatted_parts(&formatted)
        }

        $(
//          #[stable(feature = "integer_exp_format", since = "1.42.0")]
            impl fmt::LowerExp for $t {
                #[allow(unused_comparisons)]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let is_nonnegative = *self >= <$t>::zero();
                    let n: $u = if is_nonnegative {
                        let positive: $u = (*self).try_into().unwrap();
                        positive
                    } else {
                        // convert the negative num to positive by summing 1 to it's 2 complement
                        let neg: $u = (*self).try_into().unwrap();
                        <$u>::zero() - neg
                    };
                    $name(n, is_nonnegative, false, f)
                }
            })*
        $(
//          #[stable(feature = "integer_exp_format", since = "1.42.0")]
            impl fmt::UpperExp for $t {
                #[allow(unused_comparisons)]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let is_nonnegative = *self >= <$t>::zero();
                    let n: $u = if is_nonnegative {
                        let a: $u = (*self).try_into().unwrap();
                        a
                    } else {
                        let neg: $u = (*self).try_into().unwrap();
                        <$u>::zero() - neg 
                    };
                    $name(n, is_nonnegative, true, f)
                }
            })*
    };
}

// Include wasm32 in here since it doesn't reflect the native pointer size, and
// often cares strongly about getting a smaller code size.
#[cfg(any(target_pointer_width = "64", target_arch = "wasm32"))]
mod imp {
    use super::*;
    impl_Display!(
        I8, U8, I16, U16, I32, U32, I64, U64
            as U64 via to_u64 named fmt_u64
    );
    impl_Exp!(
        I8, U8, I16, U16, I32, U32, I64, U64
            as U64 via to_u64 named exp_u64
    );
}


/// Helper function for writing a u64 into `buf` going from last to first, with `curr`.
fn parse_u64_into<const N: usize>(mut n: u64, buf: &mut [MaybeUninit<u8>; N], curr: &mut isize) {
    let buf_ptr = MaybeUninit::slice_as_mut_ptr(buf);
    let lut_ptr = DEC_DIGITS_LUT.as_ptr();
    assert!(*curr > 19);

    // SAFETY:
    // Writes at most 19 characters into the buffer. Guaranteed that any ptr into LUT is at most
    // 198, so will never OOB. There is a check above that there are at least 19 characters
    // remaining.
    unsafe {
        if n >= 1e16 as u64 {
            let to_parse = n % 1e16 as u64;
            n /= 1e16 as u64;

            // Some of these are nops but it looks more elegant this way.
            let d1 = ((to_parse / 1e14 as u64) % 100) << 1;
            let d2 = ((to_parse / 1e12 as u64) % 100) << 1;
            let d3 = ((to_parse / 1e10 as u64) % 100) << 1;
            let d4 = ((to_parse / 1e8 as u64) % 100) << 1;
            let d5 = ((to_parse / 1e6 as u64) % 100) << 1;
            let d6 = ((to_parse / 1e4 as u64) % 100) << 1;
            let d7 = ((to_parse / 1e2 as u64) % 100) << 1;
            let d8 = ((to_parse / 1e0 as u64) % 100) << 1;

            *curr -= 16;

            ptr::copy_nonoverlapping(lut_ptr.offset(d1 as isize), buf_ptr.offset(*curr + 0), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d2 as isize), buf_ptr.offset(*curr + 2), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d3 as isize), buf_ptr.offset(*curr + 4), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d4 as isize), buf_ptr.offset(*curr + 6), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d5 as isize), buf_ptr.offset(*curr + 8), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d6 as isize), buf_ptr.offset(*curr + 10), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d7 as isize), buf_ptr.offset(*curr + 12), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d8 as isize), buf_ptr.offset(*curr + 14), 2);
        }
        if n >= 1e8 as u64 {
            let to_parse = n % 1e8 as u64;
            n /= 1e8 as u64;

            // Some of these are nops but it looks more elegant this way.
            let d1 = ((to_parse / 1e6 as u64) % 100) << 1;
            let d2 = ((to_parse / 1e4 as u64) % 100) << 1;
            let d3 = ((to_parse / 1e2 as u64) % 100) << 1;
            let d4 = ((to_parse / 1e0 as u64) % 100) << 1;
            *curr -= 8;

            ptr::copy_nonoverlapping(lut_ptr.offset(d1 as isize), buf_ptr.offset(*curr + 0), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d2 as isize), buf_ptr.offset(*curr + 2), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d3 as isize), buf_ptr.offset(*curr + 4), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d4 as isize), buf_ptr.offset(*curr + 6), 2);
        }
        // `n` < 1e8 < (1 << 32)
        let mut n = n as u32;
        if n >= 1e4 as u32 {
            let to_parse = n % 1e4 as u32;
            n /= 1e4 as u32;

            let d1 = (to_parse / 100) << 1;
            let d2 = (to_parse % 100) << 1;
            *curr -= 4;

            ptr::copy_nonoverlapping(lut_ptr.offset(d1 as isize), buf_ptr.offset(*curr + 0), 2);
            ptr::copy_nonoverlapping(lut_ptr.offset(d2 as isize), buf_ptr.offset(*curr + 2), 2);
        }

        // `n` < 1e4 < (1 << 16)
        let mut n = n as u16;
        if n >= 100 {
            let d1 = (n % 100) << 1;
            n /= 100;
            *curr -= 2;
            ptr::copy_nonoverlapping(lut_ptr.offset(d1 as isize), buf_ptr.offset(*curr), 2);
        }

        // decode last 1 or 2 chars
        if n < 10 {
            *curr -= 1;
            *buf_ptr.offset(*curr) = (n as u8) + b'0';
        } else {
            let d1 = n << 1;
            *curr -= 2;
            ptr::copy_nonoverlapping(lut_ptr.offset(d1 as isize), buf_ptr.offset(*curr), 2);
        }
    }
}

/// Specialized optimization for u128. Instead of taking two items at a time, it splits
/// into at most 2 u64s, and then chunks by 10e16, 10e8, 10e4, 10e2, and then 10e1.
/// It also has to handle 1 last item, as 10^40 > 2^128 > 10^39, whereas
/// 10^20 > 2^64 > 10^19.
fn fmt_u128(n: u128, is_nonnegative: bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // 2^128 is about 3*10^38, so 39 gives an extra byte of space
    let mut buf = [MaybeUninit::<u8>::uninit(); 39];
    let mut curr = buf.len() as isize;

    let (n, rem) = udiv_1e19(n);
    parse_u64_into(rem, &mut buf, &mut curr);

    if n != 0 {
        // 0 pad up to point
        let target = (buf.len() - 19) as isize;
        // SAFETY: Guaranteed that we wrote at most 19 bytes, and there must be space
        // remaining since it has length 39
        unsafe {
            ptr::write_bytes(
                MaybeUninit::slice_as_mut_ptr(&mut buf).offset(target),
                b'0',
                (curr - target) as usize,
            );
        }
        curr = target;

        let (n, rem) = udiv_1e19(n);
        parse_u64_into(rem, &mut buf, &mut curr);
        // Should this following branch be annotated with unlikely?
        if n != 0 {
            let target = (buf.len() - 38) as isize;
            // The raw `buf_ptr` pointer is only valid until `buf` is used the next time,
            // buf `buf` is not used in this scope so we are good.
            let buf_ptr = MaybeUninit::slice_as_mut_ptr(&mut buf);
            // SAFETY: At this point we wrote at most 38 bytes, pad up to that point,
            // There can only be at most 1 digit remaining.
            unsafe {
                ptr::write_bytes(buf_ptr.offset(target), b'0', (curr - target) as usize);
                curr = target - 1;
                *buf_ptr.offset(curr) = (n as u8) + b'0';
            }
        }
    }

    // SAFETY: `curr` > 0 (since we made `buf` large enough), and all the chars are valid
    // UTF-8 since `DEC_DIGITS_LUT` is
    let buf_slice = unsafe {
        str::from_utf8_unchecked(slice::from_raw_parts(
            MaybeUninit::slice_as_mut_ptr(&mut buf).offset(curr),
            buf.len() - curr as usize,
        ))
    };
    f.pad_integral(is_nonnegative, "", buf_slice)
}
