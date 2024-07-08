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

/* origin: FreeBSD /usr/src/lib/msun/src/e_pow.c */
/*
 * ====================================================
 * Copyright (C) 2004 by Sun Microsystems, Inc. All rights reserved.
 *
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

// pow(x,y) return x**y
//
//                    n
// Method:  Let x =  2   * (1+f)
//      1. Compute and return log2(x) in two pieces:
//         (does not apply - due to integer math) log2(x) = w1 + w2, where w1
//         has 53-24 = 29 bit trailing zeros.
//      2. Perform y*log2(x) = n+y' by simulating muti-precision
//         (does not apply - due to integer math) arithmetic, where |y'|<=0.5.
//      3. Return x**y = 2**n*exp(y'*log2)
//         (x**y = exp(ln(x) * y))
//
// Special cases:
//      1. (anything) ** 0  is 1
//      2. 1 ** (anything)  is 1
//      3. (anything except 1) ** NAN is NAN
//         (does not apply - no NAN)
//      4. NAN ** (anything except 0) is NAN
//         (does not apply - no NAN)
//      5. +-(|x| > 1) **  +INF is +INF
//         (does not apply - no INF)
//      6. +-(|x| > 1) **  -INF is +0
//         (does not apply - no INF)
//      7. +-(|x| < 1) **  +INF is +0
//         (does not apply - no INF)
//      8. +-(|x| < 1) **  -INF is +INF
//         (does not apply - no INF)
//      9. -1          ** +-INF is 1
//         (does not apply - no INF)
//      10. +0 ** (+anything except 0, NAN)               is +0
//      11. -0 ** (+anything except 0, NAN, odd integer)  is +0
//          (does not apply - only positive zero)
//      12. +0 ** (-anything except 0, NAN)               is +INF, raise
//          divbyzero
//      13. -0 ** (-anything except 0, NAN, odd integer)  is +INF, raise
//          divbyzero  (does not apply - only positive zero)
//      14. -0 ** (+odd integer) is -0
//          (does not apply - only positive zero)
//      15. -0 ** (-odd integer) is -INF, raise divbyzero
//          (does not apply - only positive zero)
//      16. +INF ** (+anything except 0,NAN) is +INF
//          (does not apply - no INF)
//      17. +INF ** (-anything except 0,NAN) is +0
//          (does not apply - no INF)
//      18. -INF ** (+odd integer) is -INF
//          (does not apply - no INF)
//      19. -INF ** (anything) = -0 ** (-anything), (anything except odd
//          integer)   (does not apply - no INF)
//      20. (anything) ** 1 is (anything)
//      21. (anything) ** -1 is 1/(anything)
//      22. (-anything) ** (integer) is (-1)**(integer)*(+anything**integer)
//      23. (-anything except 0 and inf) ** (non-integer) is NAN
//
// Accuracy:
//      pow(x,y) returns x**y nearly rounded. In particular
//                      pow(integer,integer)
//      always returns the correct integer provided it is
//      representable.
//

use crate::exponential::ExponentialPreciseDecimal;
use crate::logarithm::LogarithmPreciseDecimal;
use num_traits::ToPrimitive;
use radix_common::math::{CheckedMul, Decimal, PreciseDecimal};
use radix_engine_interface::prelude::pdec;

pub trait PowerDecimal {
    fn pow(&self, exp: Decimal) -> Option<Decimal>;
}

pub trait PowerPreciseDecimal {
    fn pow(&self, exp: PreciseDecimal) -> Option<PreciseDecimal>;
}

impl PowerDecimal for Decimal {
    /// Calculates the power of a Decimal
    /// Using the natural logarithm of PreciseDecimal internally
    fn pow(&self, exp: Decimal) -> Option<Decimal> {
        let exp = PreciseDecimal::try_from(exp).ok()?;
        PreciseDecimal::try_from(*self)
            .ok()?
            .pow(exp)
            .and_then(|e| e.try_into().ok())
    }
}

impl PowerPreciseDecimal for PreciseDecimal {
    /// Calculates the power of a PreciseDecimal
    fn pow(&self, exp: PreciseDecimal) -> Option<PreciseDecimal> {
        // based on https://github.com/rust-lang/libm/blob/master/src/math/pow.rs
        if exp == PreciseDecimal::ZERO {
            // special case (1)
            return Some(PreciseDecimal::ONE);
        }
        if *self == PreciseDecimal::ONE {
            // special case (2)
            return Some(PreciseDecimal::ONE);
        }
        if *self == PreciseDecimal::ZERO && exp.is_positive() {
            // special case (10)
            return Some(PreciseDecimal::ZERO);
        }
        if *self == PreciseDecimal::ZERO && exp.is_negative() {
            // special case (12)
            return None;
        }
        if exp == PreciseDecimal::ONE {
            // special case (20)
            return Some(self.clone());
        }
        if exp == pdec!(-1) {
            // special case (21)
            return Some(PreciseDecimal::ONE / *self);
        }

        if self.is_negative() {
            let exp_is_integer = PreciseDecimal(
                exp.0 / PreciseDecimal::ONE.0 * PreciseDecimal::ONE.0,
            ) == exp;
            if !exp_is_integer {
                // special case (23)
                return None;
            }
            // special case (22)
            let is_even = (exp.0 / PreciseDecimal::ONE.0).to_i32()? % 2 == 0;
            let pow = (self.checked_abs()?.ln()? * exp).exp();
            if is_even {
                return pow;
            }
            return Some(pdec!(-1) * pow?);
        }

        Some((self.ln()?.checked_mul(exp))?.exp()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use radix_engine_interface::prelude::dec;

    #[test]
    fn test_pow_exp_zero() {
        assert_eq!(dec!(-2).pow(dec!(0)), Some(dec!(1)));
        assert_eq!(dec!(-1).pow(dec!(0)), Some(dec!(1)));
        assert_eq!(dec!(0).pow(dec!(0)), Some(dec!(1)));
        assert_eq!(dec!(1).pow(dec!(0)), Some(dec!(1)));
        assert_eq!(dec!(2).pow(dec!(0)), Some(dec!(1)));
    }

    #[test]
    fn test_pow_base_one() {
        assert_eq!(dec!(1).pow(dec!(2)), Some(dec!(1)));
        assert_eq!(dec!(1).pow(dec!(-2)), Some(dec!(1)));
    }

    #[test]
    fn test_pow_base_zero() {
        assert_eq!(dec!(0).pow(dec!(-2)), None);
        assert_eq!(dec!(0).pow(dec!(-1)), None);
        assert_eq!(dec!(0).pow(dec!(0)), Some(dec!(1)));
        assert_eq!(dec!(0).pow(dec!(1)), Some(dec!(0)));
        assert_eq!(dec!(0).pow(dec!(2)), Some(dec!(0)));
    }

    #[test]
    fn test_pow_exp_one() {
        assert_eq!(dec!(2).pow(dec!(1)), Some(dec!(2)));
        assert_eq!(dec!(-2).pow(dec!(1)), Some(dec!(-2)));
    }

    #[test]
    fn test_pow_exp_minus_one() {
        assert_eq!(dec!(2).pow(dec!(-1)), Some(dec!("0.5")));
        assert_eq!(dec!(-2).pow(dec!(-1)), Some(dec!("-0.5")));
    }

    #[test]
    fn test_pow_base_negative_exp_integer() {
        assert_eq!(dec!(2).pow(dec!(-2)), Some(dec!("0.25")));
        assert_eq!(dec!(-2).pow(dec!(2)), Some(dec!("4")));
        assert_eq!(dec!(-2).pow(dec!(-2)), Some(dec!("0.25")));
        assert_eq!(dec!(5).pow(dec!(-5)), Some(dec!("0.00032")));
        assert_eq!(
            dec!(-5).pow(dec!(5)),
            Some(dec!("-3125") + dec!("0.000000000000001660"))
        );
        assert_eq!(dec!(-5).pow(dec!(-5)), Some(dec!("-0.00032")));
    }

    #[test]
    fn test_pow_base_negative_exp_non_integer() {
        assert_eq!(dec!("-1.1").pow(dec!("0.00000000000000001")), None);
        assert_eq!(dec!("-3.4").pow(dec!("15.43")), None);
        assert_eq!(dec!("-3.4").pow(dec!("-15.43")), None);
    }

    #[test]
    fn test_pow_base_maximum_exp_non_integer() {
        assert_eq!(dec!("-1.1").pow(dec!("0.00000000000000001")), None);
        assert_eq!(dec!("-3.4").pow(dec!("15.43")), None);
        assert_eq!(dec!("-3.4").pow(dec!("-15.43")), None);
    }

    #[test]
    fn test_pow_smallest_value() {
        assert_eq!(
            dec!("3.4").pow(dec!("-33.43")),
            Some(dec!("0.000000000000000001"))
        );
    }

    #[test]
    fn test_pow_largest_value() {
        assert_eq!(
            dec!("3.4").pow(dec!("71.43")),
            Some(
                dec!(
                    "91947313437872693600354888137039353441.244419982586019069"
                ) - dec!("187832408272640032348.012171022248677284")
            )
        );
    }

    #[test]
    fn test_pow_base_minimum() {
        assert_eq!(Decimal::MIN.pow(dec!(3)), None);
        assert_eq!(Decimal::MIN.pow(Decimal::MIN), None);
        assert_eq!(Decimal::MIN.pow(Decimal::MAX), None);
    }

    #[test]
    fn test_pow_base_maximum() {
        assert_eq!(Decimal::MAX.pow(dec!(3)), None);
        assert_eq!(Decimal::MAX.pow(Decimal::MIN), None);
        assert_eq!(Decimal::MAX.pow(Decimal::MAX), None);
    }

    #[test]
    fn test_pow_base_positive_normal() {
        assert_eq!(dec!(2).pow(dec!(2)), Some(dec!(4)));
        assert_eq!(
            dec!("3.4").pow(dec!("15.43")),
            Some(
                dec!("158752177.142935864260984228")
                    - dec!("0.000000000094162353")
            )
        );
        assert_eq!(
            dec!("3.4").pow(dec!("-15.43")),
            Some(dec!("0.000000006299126210"))
        );
    }
}
