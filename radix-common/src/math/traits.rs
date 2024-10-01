pub trait CheckedAdd<Rhs = Self> {
    type Output;

    fn checked_add(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait SaturatingAdd<Rhs = Self> {
    type Output;

    fn saturating_add(self, other: Rhs) -> Self::Output
    where
        Self: Sized;
}

pub trait CheckedSub<Rhs = Self> {
    type Output;

    fn checked_sub(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait CheckedMul<Rhs = Self> {
    type Output;

    fn checked_mul(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait CheckedDiv<Rhs = Self> {
    type Output;

    fn checked_div(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait CheckedNeg<Rhs = Self> {
    type Output;

    fn checked_neg(self) -> Option<Self::Output>
    where
        Self: Sized;
}
