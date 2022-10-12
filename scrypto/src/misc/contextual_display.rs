use sbor::rust::fmt;

pub trait ContextualDisplay<Context>: Sized {
    type Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &Context,
    ) -> Result<(), Self::Error>;

    fn format<F: fmt::Write, TContext: Into<Context>>(
        &self,
        f: &mut F,
        context: TContext,
    ) -> Result<(), Self::Error> {
        self.contextual_format(f, &context.into())
    }

    fn display<'a, 'b, TContext: Into<Context>>(
        &'a self,
        context: TContext,
    ) -> ContextDisplayable<'a, Self, Context> {
        ContextDisplayable {
            value: self,
            context: context.into(),
        }
    }
}

pub struct ContextDisplayable<'a, TValue, TContext>
where
    TValue: ContextualDisplay<TContext> + Sized,
{
    value: &'a TValue,
    context: TContext,
}

impl<'a, 'b, TValue, TContext> fmt::Display for ContextDisplayable<'a, TValue, TContext>
where
    TValue: ContextualDisplay<TContext> + Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.value
            .contextual_format(f, &self.context)
            .map_err(|_| fmt::Error) // We eat any errors into fmt::Error
    }
}
