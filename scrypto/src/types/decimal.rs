use core::ops::*;

use num_bigint::BigInt;
use num_traits::sign::Signed;

use crate::rust::fmt;
use crate::rust::format;

const PRECISION: u128 = 10u128.pow(18);

/// Represented a **signed** fixed-point decimal, where the precision is 10^-18.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(BigInt);

impl Decimal {
    pub fn new<T: Into<BigInt>>(value: T, decimals: u8) -> Self {
        assert!(decimals <= 18);

        Self(value.into() * 10u128.pow((18 - decimals).into()))
    }
}

impl<T: Into<BigInt>> From<T> for Decimal {
    fn from(v: T) -> Self {
        Self(v.into() * PRECISION)
    }
}

impl<T: Into<Decimal>> Add<T> for Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        Self(self.0 + other.into().0)
    }
}

impl<T: Into<Decimal>> Sub<T> for Decimal {
    type Output = Decimal;

    fn sub(self, other: T) -> Self::Output {
        Self(self.0 - other.into().0)
    }
}

impl<T: Into<Decimal>> Mul<T> for Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        Self(self.0 * other.into().0 / PRECISION)
    }
}

impl<T: Into<Decimal>> Div<T> for Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        Self(self.0 * PRECISION / other.into().0)
    }
}
impl Shl<usize> for Decimal {
    type Output = Decimal;

    fn shl(self, shift: usize) -> Self::Output {
        Self(self.0.shl(shift))
    }
}

impl Shr<usize> for Decimal {
    type Output = Decimal;

    fn shr(self, shift: usize) -> Self::Output {
        Self(self.0.shr(shift))
    }
}

impl<T: Into<Decimal>> AddAssign<T> for Decimal {
    fn add_assign(&mut self, other: T) {
        self.0 += other.into().0;
    }
}

impl<T: Into<Decimal>> SubAssign<T> for Decimal {
    fn sub_assign(&mut self, other: T) {
        self.0 -= other.into().0;
    }
}

impl<T: Into<Decimal>> MulAssign<T> for Decimal {
    fn mul_assign(&mut self, other: T) {
        self.0 = self.0.clone() * other.into().0 / PRECISION;
    }
}

impl<T: Into<Decimal>> DivAssign<T> for Decimal {
    fn div_assign(&mut self, other: T) {
        self.0 = self.0.clone() * PRECISION / other.into().0;
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw = self.0.abs().to_str_radix(10);
        // add radix point
        let scaled = if raw.len() <= 18 {
            format!("0.{}{}", "0".repeat(18 - raw.len()), raw)
        } else {
            format!("{}.{}", &raw[..raw.len() - 18], &raw[raw.len() - 18..])
        };

        // strip trailing zeros
        let mut res = scaled.as_str();
        while res.ends_with('0') {
            res = &res[..res.len() - 1];
        }
        if res.ends_with('.') {
            res = &res[..res.len() - 1];
        }

        write!(f, "{}{}", if self.0.is_positive() { "" } else { "-" }, res)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_format() {
        assert_eq!(Decimal(1u128.into()).to_string(), "0.000000000000000001");
        assert_eq!(
            Decimal(123456789123456789u128.into()).to_string(),
            "0.123456789123456789"
        );
        assert_eq!(Decimal(1000000000000000000u128.into()).to_string(), "1");
        assert_eq!(Decimal(123000000000000000000u128.into()).to_string(), "123");
    }

    #[test]
    fn test_add() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a + b).to_string(), "12");
    }

    #[test]
    fn test_sub() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a.clone() - b.clone()).to_string(), "-2");
        assert_eq!((b - a).to_string(), "2");
    }

    #[test]
    fn test_mul() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a * b).to_string(), "35");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(0u32);
        assert_eq!((a / b).to_string(), "0");
    }

    #[test]
    fn test_div() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a / b).to_string(), "0.714285714285714285");
    }
}
