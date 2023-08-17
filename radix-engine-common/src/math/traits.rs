pub trait SafeAdd<Rhs = Self> {
    type Output;

    fn safe_add(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait SafeSub<Rhs = Self> {
    type Output;

    fn safe_sub(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait SafeMul<Rhs = Self> {
    type Output;

    fn safe_mul(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait SafeDiv<Rhs = Self> {
    type Output;

    fn safe_div(self, other: Rhs) -> Option<Self::Output>
    where
        Self: Sized;
}

pub trait SafeNeg<Rhs = Self> {
    type Output;

    fn safe_neg(self) -> Option<Self::Output>
    where
        Self: Sized;
}
