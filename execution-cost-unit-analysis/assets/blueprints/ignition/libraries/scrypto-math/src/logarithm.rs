// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

/* origin: FreeBSD /usr/src/lib/msun/src/e_log.c */
/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */
/* log(x)
 * Return the logarithm of x
 *
 * Method :
 *   1. Argument Reduction: find k and f such that x = 2^k * (1+f), where
 *      sqrt(2)/2 < 1+f < sqrt(2) .
 *
 *   2. Approximation of log(1+f). Let s = f/(2+f) ; based on log(1+f) =
 *      log(1+s) - log(1-s) = 2s + 2/3 s**3 + 2/5 s**5 + ....., = 2s + s*R We
 *      use a special Remez algorithm on [0,0.1716] to generate a polynomial
 *      of degree 14 to approximate R The maximum error of this polynomial
 *      approximation is bounded by 2**-58.45. In other words, 2      4
 *      6      8      10      12      14 R(z) ~ Lg1*s +Lg2*s +Lg3*s +Lg4*s
 *      +Lg5*s  +Lg6*s  +Lg7*s (the values of Lg1 to Lg7 are listed in the
 *      program) and |      2          14          |     -58.45 | Lg1*s
 *      +...+Lg7*s    -  R(z) | <= 2 |                             | Note
 *      that 2s = f - s*f = f - hfsq + s*hfsq, where hfsq = f*f/2. In order
 *      to guarantee error in log below 1ulp, we compute log by log(1+f) = f
 *      - s*(f - R)        (if f is not too large) log(1+f) = f - (hfsq -
 *      s*(hfsq+R)).     (better accuracy)
 *
 *   3. Finally,  log(x) = k*ln2 + log(1+f). =
 *      k*ln2_hi+(f-(hfsq-(s*(hfsq+R)+k*ln2_lo))) Here ln2 is split into two
 *      floating point number: ln2_hi + ln2_lo, where n*ln2_hi is always
 *      exact for |n| < 2000.
 *
 * Special cases:
 *      log(x) is NaN with signal if x < 0 (including -INF) ;
 *      log(+INF) is +INF; log(0) is -INF with signal;
 *      log(NaN) is that NaN with no signal.
 *
 * Accuracy:
 *      according to an error analysis, the error is always less than
 *      1 ulp (unit in the last place).
 *
 * Misc:
 *      hi and lo separation is not needed for Decimal and PreciseDecimal in
 * Scrypto      due to exact integer calculation.
 */

use num_traits::Zero;
use radix_common::math::{Decimal, PreciseDecimal};
use radix_engine_interface::prelude::pdec;

const LN2: PreciseDecimal = pdec!("0.693147180559945309417232121458176568");
const LN10: PreciseDecimal = pdec!("2.302585092994045684017991454684364207");
const SQRT: PreciseDecimal = pdec!("1.414213562373095048801688724209698078");
const SQRT_HALF: PreciseDecimal =
    pdec!("0.707106781186547524400844362104849039");
const LG1: PreciseDecimal = pdec!("0.6666666666666735130"); // 6.666666666666735130e-01
const LG2: PreciseDecimal = pdec!("0.3999999999940941908"); // 3.999999999940941908e-01
const LG3: PreciseDecimal = pdec!("0.2857142874366239149"); // 2.857142874366239149e-01
const LG4: PreciseDecimal = pdec!("0.2222219843214978396"); // 2.222219843214978396e-01
const LG5: PreciseDecimal = pdec!("0.1818357216161805012"); // 1.818357216161805012e-01
const LG6: PreciseDecimal = pdec!("0.1531383769920937332"); // 1.531383769920937332e-01
const LG7: PreciseDecimal = pdec!("0.1479819860511658591"); // 1.479819860511658591e-01

pub trait LogarithmDecimal {
    fn ln(&self) -> Option<Decimal>;
    fn log2(&self) -> Option<Decimal>;
    fn log10(&self) -> Option<Decimal>;
    fn log_base(&self, base: Decimal) -> Option<Decimal>;
}

pub trait LogarithmPreciseDecimal {
    fn ln(&self) -> Option<PreciseDecimal>;
    fn log2(&self) -> Option<PreciseDecimal>;
    fn log10(&self) -> Option<PreciseDecimal>;
    fn log_base(&self, base: PreciseDecimal) -> Option<PreciseDecimal>;
}

/// Reduces the argument x by finding k and f such that
/// x = 2^k * (1+f)    where  sqrt(2)/2 < 1+f < sqrt(2)
fn log_reduce_argument(number: PreciseDecimal) -> (i32, PreciseDecimal) {
    let full_integer = number.0 / PreciseDecimal::ONE.0;

    if full_integer.is_zero() {
        if number >= SQRT_HALF {
            return (0, number);
        }

        // uses leading zeros of the full big integer to derive k
        let k = number.0.leading_zeros() as i32
            - SQRT_HALF.0.leading_zeros() as i32;
        let r = number * PreciseDecimal(PreciseDecimal::ONE.0 << k as u32);

        if r >= SQRT_HALF {
            return (-k, r);
        }

        // r can be smaller than SQRT_HALF but still having the same amount of
        // leading zeros
        return (-k - 1, r * pdec!(2));
    }

    // uses leading zeros of the full big integer to derive k
    // 255 bits only because the first bit is the sign bit
    let k = 255 - full_integer.leading_zeros() as i32; // index highest integer bit
    let r = number / PreciseDecimal(PreciseDecimal::ONE.0 << k as u32);

    if r <= SQRT {
        return (k, r);
    }

    // r can be larger than SQRT but still having the same amount of leading
    // zeros
    return (k + 1, r / pdec!(2));
}

impl LogarithmDecimal for Decimal {
    /// Calculates the natural logarithm of a Decimal
    /// Using the natural logarithm of PreciseDecimal internally
    fn ln(&self) -> Option<Decimal> {
        PreciseDecimal::try_from(*self)
            .ok()?
            .ln()
            .and_then(|log| log.try_into().ok())
    }

    /// Calculates the logarithm to base 2 of a Decimal
    /// Using the logarithm to base 2 of PreciseDecimal internally
    fn log2(&self) -> Option<Decimal> {
        PreciseDecimal::try_from(*self)
            .ok()?
            .log2()
            .and_then(|log| log.try_into().ok())
    }

    /// Calculates the logarithm to base 10 of a Decimal
    /// Using the logarithm to base 10 of PreciseDecimal internally
    fn log10(&self) -> Option<Decimal> {
        PreciseDecimal::try_from(*self)
            .ok()?
            .log10()
            .and_then(|log| log.try_into().ok())
    }

    /// Calculates the logarithm to an arbitrary base of a Decimal
    /// Using the logarithm to an abitrary base of PreciseDecimal internally
    fn log_base(&self, base: Decimal) -> Option<Decimal> {
        let base = PreciseDecimal::try_from(base).ok()?;
        PreciseDecimal::try_from(*self)
            .ok()?
            .log_base(base)
            .and_then(|log| log.try_into().ok())
    }
}

impl LogarithmPreciseDecimal for PreciseDecimal {
    /// Calculates the natural logarithm of a PreciseDecimal
    fn ln(&self) -> Option<PreciseDecimal> {
        // based on https://github.com/rust-lang/libm/blob/master/src/math/log.rs
        if !self.is_positive() {
            return None;
        }

        // (1) Argument Reduction
        let (k, r) = log_reduce_argument(*self);
        // println!("k = {:?}, r = {:?}", k, r);
        // println!("x_n = {:?}", pdec!(2).checked_powi(k.into())? * r);
        // println!("x_o = {:?}", self);

        // (2) Approximation of log(1+f)
        let f = r - PreciseDecimal::ONE;
        let s = f / (pdec!(2) + f);
        let z = s * s;
        let w = z * z;
        let remez = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)))
            + w * (LG2 + w * (LG4 + w * LG6));
        let log_1_f = f - s * (f - remez);

        // (3) log(x) = k*ln2 + log(1+f)
        Some(LN2 * k + log_1_f)
    }

    /// Calculates the logarithm to base 2 of a PreciseDecimal
    fn log2(&self) -> Option<PreciseDecimal> {
        Some(self.ln()? / LN2)
    }

    /// Calculates the logarithm to base 10 of a PreciseDecimal
    fn log10(&self) -> Option<PreciseDecimal> {
        Some(self.ln()? / LN10)
    }

    /// Calculates the logarithm to an arbitrary base of a PreciseDecimal
    fn log_base(&self, base: PreciseDecimal) -> Option<PreciseDecimal> {
        let base_ln = base.ln()?;
        Some(self.ln()? / base_ln)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use radix_engine_interface::prelude::dec;

    #[test]
    fn test_constants() {
        assert_eq!(LN2, pdec!("0.693147180559945309417232121458176568"));
        assert_eq!(LN10, pdec!("2.302585092994045684017991454684364207"));
        assert_eq!(SQRT, pdec!("1.414213562373095048801688724209698078"));
        assert_eq!(SQRT_HALF, pdec!("0.707106781186547524400844362104849039"));
        assert_eq!(LG1, pdec!("0.6666666666666735130"));
        assert_eq!(LG2, pdec!("0.3999999999940941908"));
        assert_eq!(LG3, pdec!("0.2857142874366239149"));
        assert_eq!(LG4, pdec!("0.2222219843214978396"));
        assert_eq!(LG5, pdec!("0.1818357216161805012"));
        assert_eq!(LG6, pdec!("0.1531383769920937332"));
        assert_eq!(LG7, pdec!("0.1479819860511658591"));
    }

    #[test]
    fn test_ln_positive_number() {
        assert_eq!(
            dec!(10).ln(),
            Some(dec!("2.302585092994045684") - dec!("0.000000000000000001"))
        );
        assert_eq!(
            pdec!(10).ln(),
            Some(
                pdec!("2.302585092994045684017991454684364207")
                    - pdec!("0.000000000000000000093151926716540583")
            )
        );
    }

    #[test]
    fn test_ln_e() {
        assert_eq!(
            dec!("2.718281828459045235").ln(),
            Some(dec!(1) - dec!("0.000000000000000001"))
        );
        assert_eq!(
            pdec!("2.718281828459045235360287471352662497").ln(),
            Some(pdec!(1) - pdec!("0.000000000000000000072256402139088204"))
        );
    }

    #[test]
    fn test_ln_one() {
        assert_eq!(dec!(1).ln(), Some(dec!(0)));
        assert_eq!(pdec!(1).ln(), Some(pdec!(0)));
    }

    #[test]
    fn test_ln_zero() {
        assert_eq!(dec!(0).ln(), None);
        assert_eq!(pdec!(0).ln(), None);
    }

    #[test]
    fn test_ln_negative_number() {
        assert_eq!(dec!(-1).ln(), None);
        assert_eq!(pdec!(-1).ln(), None);
    }

    #[test]
    fn test_ln_lesser_sqrt_half() {
        assert_eq!(
            (SQRT_HALF - pdec!("0.000000000000000000000000000000000001")).ln(),
            Some(
                pdec!("-0.346573590279972654708616060729088286")
                    - pdec!("0.000000000000000000349708283169683682")
            )
        ); // * 2
        assert_eq!(
            dec!("0.664613997892457936").ln(),
            Some(dec!("-0.408548861152152805") + dec!("0.000000000000000001"))
        ); // * 2; equal leading zeros of sqrt_half and number (~ 2 ** 119 =
           //   1000...)
        assert_eq!(dec!("0.5").ln(), Some(dec!("-0.693147180559945309"))); // * 2
        assert_eq!(dec!("0.25").ln(), Some(dec!("-1.386294361119890618"))); // * 2^2
        assert_eq!(dec!("0.125").ln(), Some(dec!("-2.079441541679835928"))); // * 2^3
    }

    #[test]
    fn test_ln_equal_sqrt_half() {
        assert_eq!(
            SQRT_HALF.ln(),
            Some(
                pdec!("-0.346573590279972654708616060729088284")
                    + pdec!("0.000000000000000000349708283169683683")
            )
        );
    }

    #[test]
    fn test_ln_between_sqrt_half_and_sqrt() {
        assert_eq!(
            (SQRT_HALF + pdec!("0.000000000000000000000000000000000001")).ln(),
            Some(
                pdec!("-0.346573590279972654708616060729088284")
                    + pdec!("0.000000000000000000349708283169683685")
            )
        );
        assert_eq!(dec!("0.8").ln(), Some(dec!("-0.223143551314209755")));
        assert_eq!(
            dec!("1.329227995784915872").ln(),
            Some(dec!("0.284598319407792504") + dec!("0.000000000000000001"))
        ); // equal leading zeros of sqrt_half and number (~ 2 ** 120 - 1 =
           // 1111...)
        assert_eq!(
            dec!("1.329227995784915873").ln(),
            Some(dec!("0.284598319407792505"))
        ); // equal leading zeros of sqrt and number (~ 2 ** 120 = 1000...)
        assert_eq!(dec!("1.2").ln(), Some(dec!("0.182321556793954626")));
        assert_eq!(
            (SQRT - pdec!("0.000000000000000000000000000000000001")).ln(),
            Some(
                pdec!("0.346573590279972654708616060729088282")
                    - pdec!("0.000000000000000000349708283169683681")
            )
        );
    }

    #[test]
    fn test_ln_equal_sqrt() {
        assert_eq!(
            SQRT.ln(),
            Some(
                pdec!("0.346573590279972654708616060729088284")
                    - pdec!("0.000000000000000000349708283169683683")
            )
        );
    }

    #[test]
    fn test_ln_greater_sqrt() {
        assert_eq!(
            (SQRT + pdec!("0.000000000000000000000000000000000001")).ln(),
            Some(
                pdec!("0.346573590279972654708616060729088284")
                    + pdec!("0.000000000000000000349708283169683683")
            )
        ); // * 2
        assert_eq!(
            dec!("2.658455991569831745").ln(),
            Some(dec!("0.977745499967737814"))
        ); // equal leading zeros for sqrt and number (~ 2**121 - 1 = 1111...)
        assert_eq!(dec!("2").ln(), Some(dec!("0.693147180559945309"))); // / 2
        assert_eq!(dec!("4").ln(), Some(dec!("1.386294361119890618"))); // / 2^2
        assert_eq!(dec!("8").ln(), Some(dec!("2.079441541679835928"))); // / 2^3
    }

    #[test]
    fn test_ln_decimal_precision() {
        assert_eq!(
            dec!("1000000000000000000000000000000").ln(),
            Some(dec!("69.077552789821370520"))
        );
        assert_eq!(
            pdec!("1000000000000000000000000000000").ln(),
            Some(
                pdec!("69.077552789821370520539743640530926228")
                    + pdec!("0.000000000000000000274111421367606147")
            )
        );
    }

    #[test]
    fn test_ln_smallest_positive() {
        assert_eq!(
            dec!("0.000000000000000001").ln(),
            Some(dec!("-41.446531673892822312"))
        );
        assert_eq!(
            pdec!("0.000000000000000000000000000000000001").ln(),
            Some(
                pdec!("-82.893063347785644624647692368637111474")
                    + pdec!("0.000000000000000000345534790097727621")
            )
        );
    }

    #[test]
    fn test_ln_maximum_possible() {
        assert_eq!(Decimal::MAX.ln(), Some(dec!("90.944579813056731786")));
        assert_eq!(
            PreciseDecimal::MAX.ln(),
            Some(
                pdec!("93.859467695000409276746498603197913385")
                    + pdec!("0.000000000000000000345534790097727602")
            )
        );
    }

    #[test]
    fn test_log_2() {
        assert_eq!(dec!(-1).log2(), None);
        assert_eq!(dec!(0).log2(), None);
        assert_eq!(dec!(1).log2(), Some(dec!(0)));
        assert_eq!(
            dec!("1.5").log2(),
            Some(dec!("0.584962500721156181") - dec!("0.000000000000000001"))
        );
        assert_eq!(dec!(2).log2(), Some(dec!(1)));
        assert_eq!(dec!(10).log2(), Some(dec!("3.321928094887362347")));
    }

    #[test]
    fn test_log_10() {
        assert_eq!(dec!(-1).log10(), None);
        assert_eq!(dec!(0).log10(), None);
        assert_eq!(dec!(1).log10(), Some(dec!(0)));
        assert_eq!(dec!(5).log10(), Some(dec!("0.698970004336018804")));
        assert_eq!(
            dec!(10).log10(),
            Some(dec!(1) - dec!("0.000000000000000001"))
        );
        assert_eq!(dec!(20).log10(), Some(dec!("1.301029995663981195")));
    }

    #[test]
    fn test_log_base() {
        assert_eq!(dec!(-1).log_base(dec!(8)), None);
        assert_eq!(dec!(0).log_base(dec!(8)), None);
        assert_eq!(dec!(1).log_base(dec!(8)), Some(dec!(0)));
        assert_eq!(
            dec!(5).log_base(dec!(8)),
            Some(dec!("0.773976031629120782"))
        );
        assert_eq!(dec!(8).log_base(dec!(8)), Some(dec!(1)));
        assert_eq!(
            dec!(10).log_base(dec!(8)),
            Some(dec!("1.107309364962454115"))
        );
        assert_eq!(
            dec!(20).log_base(dec!(8)),
            Some(dec!("1.440642698295787449"))
        );
        assert_eq!(
            Decimal::MAX.log_base(dec!(8)),
            Some(dec!("43.735098097342492579"))
        );
    }
}
