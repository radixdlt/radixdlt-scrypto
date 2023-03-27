use crate::rust::fmt;
use crate::rust::prelude::*;

/// This trait is used where context is required to correctly display a value.
///
/// Typically, this is due to needing to know the current network to display addresses.
/// Other forms of Context are also possible. See `ComponentAddress`
/// or `TransactionReceipt` in the `radix-engine` crate for example implementations.
///
/// The `Context` used should typically just be a wrapper type around references, and so
/// be a small, cheap, ephemeral value on the stack (if it's not just optimized away entirely).
/// It is therefore recommended that the `Context` implement `Copy`,
/// to make it very easy to pass around and re-use.
///
pub trait ContextualDisplay<Context> {
    type Error;

    /// Formats the value to the given `fmt::Write` buffer, making use of the provided context.
    /// See also [`format`], which is typically easier to use, as it takes an `Into<Context>`
    /// instead of a `&Context`.
    ///
    /// [`format`]: #method.format
    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &Context,
    ) -> Result<(), Self::Error>;

    /// Formats the value to the given `fmt::Write` buffer, making use of the provided context.
    /// See also [`contextual_format`], which takes a `&Context` instead of an `Into<Context>`.
    ///
    /// Alternatively, the [`display`] method can be used to create an object that can be used
    /// directly in a `format!` style macro.
    ///
    /// [`contextual_format`]: #method.contextual_format
    /// [`display`]: #method.display
    fn format<F: fmt::Write, TContext: Into<Context>>(
        &self,
        f: &mut F,
        context: TContext,
    ) -> Result<(), Self::Error> {
        self.contextual_format(f, &context.into())
    }

    /// Returns an object implementing `fmt::Display`, which can be used in a `format!` style macro.
    ///
    /// Whilst this is syntactically nicer, beware that the use of `format!` absorbs any errors during
    /// formatting, replacing them with `fmt::Error`.
    /// If you'd like to preserve errors, use the [`format`] method instead. This may require manually
    /// splitting up your `format!` style macro. For example:
    ///
    /// ```rust,ignore
    /// // Syntactically nice, but the AddressError is swallowed into fmt::Error
    /// write!(f, "ComponentAddress(\"{}\")", address.display(context))?;
    ///
    /// // Less nice, but the AddressError is correctly returned
    /// f.write_str("ComponentAddress(\"")?;
    /// address.format(f, context)?;
    /// f.write_str("\")")?;
    /// ```
    ///
    /// [`format`]: #method.format
    fn display<'a, 'b, TContext: Into<Context>>(
        &'a self,
        context: TContext,
    ) -> ContextDisplayable<'a, Self, Context> {
        ContextDisplayable {
            value: self,
            context: context.into(),
        }
    }

    fn to_string<'a, 'b, TContext: Into<Context>>(&'a self, context: TContext) -> String {
        self.display(context).to_string()
    }
}

pub struct ContextDisplayable<'a, TValue, TContext>
where
    TValue: ContextualDisplay<TContext> + ?Sized,
{
    value: &'a TValue,
    context: TContext,
}

impl<'a, 'b, TValue, TContext> fmt::Display for ContextDisplayable<'a, TValue, TContext>
where
    TValue: ContextualDisplay<TContext> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.value
            .contextual_format(f, &self.context)
            .map_err(|_| fmt::Error) // We eat any errors into fmt::Error
    }
}
