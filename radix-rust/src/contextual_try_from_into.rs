pub trait ContextualTryFrom<T>
where
    Self: Sized,
{
    type Context;
    type Error;

    fn contextual_try_from(value: T, context: &Self::Context) -> Result<Self, Self::Error>;
}

pub trait ContextualTryInto<T> {
    type Context;
    type Error;

    fn contextual_try_into(self, context: &Self::Context) -> Result<T, Self::Error>;
}

impl<T, U> ContextualTryInto<U> for T
where
    U: ContextualTryFrom<T>,
{
    type Error = U::Error;
    type Context = U::Context;

    #[inline]
    fn contextual_try_into(self, context: &U::Context) -> Result<U, U::Error> {
        U::contextual_try_from(self, context)
    }
}
