use serde::{Serialize, Serializer};

/// This trait is used where context is required to correctly serialize a value.
///
/// Typically, this is due to needing to know the current network to display addresses.
/// Other forms of Context are also possible.
///
/// The `Context` used should typically just be a wrapper type around references, and so
/// be a small, cheap, ephemeral value on the stack (if it's not just optimized away entirely).
/// It is therefore recommended that the `Context` implement `Copy`,
/// to make it very easy to pass around and re-use.
///
pub trait ContextualSerialize<Context> {
    /// Serializes the value to the given `serde::Serializer`, making use of the provided context.
    /// See also [`serialize`], which is typically easier to use, as it takes an `Into<Context>`
    /// instead of a `&Context`.
    ///
    /// Any custom errors during serialization will need mapping into a custom serde error,
    /// which basically wraps a String, via: `serde::ser::Error::custom(error: Displayable)`.
    ///
    /// [`serialize`]: #method.serialize
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &Context,
    ) -> Result<S::Ok, S::Error>;

    /// Serializes the value to the given `serde::Serializer`, making use of the provided context.
    /// See also [`contextual_serialize`], which takes a `&Context` instead of an `Into<Context>`.
    ///
    /// Alternatively, the [`serializable`] method can be used to create an object that
    /// directly implements `serde::Serialize`, for passing to `serde` functions.
    ///
    /// [`contextual_serialize`]: #method.contextual_serialize
    /// [`serializable`]: #method.serializable
    fn serialize<S: Serializer, TContext: Into<Context>>(
        &self,
        serializer: S,
        context: TContext,
    ) -> Result<S::Ok, S::Error> {
        self.contextual_serialize(serializer, &context.into())
    }

    /// Returns an object implementing `serde::Serialize`, which can be passed to `serde` functions.
    fn serializable<'a, 'b, TContext: Into<Context>>(
        &'a self,
        context: TContext,
    ) -> ContextSerializable<'a, Self, Context> {
        ContextSerializable {
            value: self,
            context: context.into(),
        }
    }
}

pub struct ContextSerializable<'a, TValue, TContext>
where
    TValue: ContextualSerialize<TContext> + ?Sized,
{
    value: &'a TValue,
    context: TContext,
}

impl<'a, TValue: ContextualSerialize<TContext> + ?Sized, TContext> Serialize
    for ContextSerializable<'a, TValue, TContext>
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.contextual_serialize(serializer, &self.context)
    }
}
