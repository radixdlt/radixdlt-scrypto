use crate::prelude::*;

/// This trait is intended to be used as an `impl` argument in helper methods, to accept
/// a wider range of arguments.
///
/// It should only be used where it is safe to panic if the wrong argument is provided,
/// and where performance isn't a primary concern.
///
/// It's not expected for other types to implement this trait directly - instead they
/// should implement [`TryFrom`] to convert between the types.
///
/// If resolution needs to be keyed against an external resolver (e.g. a look-up to translate
/// string names into values), then [`LabelledResolve`] should be used instead.
///
/// ## Implementers
/// * You should prefer to implement [`ResolveFrom`] as it is easier to implement
/// due to trait coherence rules. Sometimes you can only implement [`Resolve`]
/// however.
/// * If requiring a labelled resolution in your bounds, prefer [`Resolve`]
/// because slightly more types can implement it.
pub trait Resolve<X: Resolvable> {
    fn resolve(self) -> X;
}

/// The inverse trait of [`Resolve`].
///
/// This should be implemented instead of [`Resolve`] where possible, but
/// [`Resolve`] should be used as bounds in arguments.
pub trait ResolveFrom<X>: Resolvable {
    fn resolve_from(value: X) -> Self;
}

impl<X, Y: ResolveFrom<X>> Resolve<Y> for X {
    fn resolve(self) -> Y {
        Y::resolve_from(self)
    }
}

/// `Resolvable` is a marker trait, mainly to make resolution opt-in and to avoid
/// polluting every type with a resolve method.
///
/// You might want to use [`resolvable_with_identity_impl`] or [`resolvable_with_try_into_impls`]
/// to implement this trait and a reflexive or blanket impl.
pub trait Resolvable {}

#[macro_export]
macro_rules! resolvable_with_identity_impl {
    ($ty:ty$(,)?) => {
        impl Resolvable for $ty {}

        impl ResolveFrom<$ty> for $ty {
            fn resolve_from(value: $ty) -> $ty {
                value
            }
        }
    };
}

#[macro_export]
macro_rules! resolvable_with_try_into_impls {
    ($ty:ty$(,)?) => {
        impl Resolvable for $ty {}

        impl<T: TryInto<$ty, Error = E>, E: Debug> ResolveFrom<T> for $ty {
            fn resolve_from(value: T) -> $ty {
                value.try_into().unwrap_or_else(|err| {
                    panic!(
                        "The provided argument could not be resolved into a {}: {err:?}",
                        core::any::type_name::<$ty>()
                    )
                })
            }
        }
    };
}

impl<'a, X: ResolveFrom<X> + Clone> ResolveFrom<&'a X> for X {
    fn resolve_from(value: &'a X) -> X {
        value.clone()
    }
}

/// This trait is intended to be used as an `impl` argument in helper methods, to accept
/// a wider range of arguments.
///
/// It should only be used where it is safe to panic if the wrong argument is provided,
/// and where performance isn't a primary concern.
///
/// Compared to [`Resolve`], [`LabelledResolve`] also accepts an optional resolver,
/// which can be used to convert label/s either directly into `Self`, or into values which
/// can be used to build up self.
///
/// However, unlike [`Resolve`], a reflexive [`LabelledResolve`] is only implemented for
/// `Self`, `&Self` and various string labels. It doesn't build on top of [`TryInto`]
/// because that causes implementation collisions with labels for types which could implement
/// `TryFrom<&str>`.
///
/// ## Implementers
/// * You should prefer to implement [`LabelledResolveFrom`] as it is easier to implement
/// due to trait coherence rules. Sometimes you can only implement [`LabelledResolve`]
/// however.
/// * If requiring a labelled resolution in your bounds, prefer [`LabelledResolve`]
/// because slightly more types can implement it.
pub trait LabelledResolve<Y: LabelledResolvable> {
    fn labelled_resolve(self, resolver: &impl LabelResolver<Y::ResolverOutput>) -> Y;
}

/// The inverse trait of [`LabelledResolve`].
///
/// This should be implemented instead of [`LabelledResolve`] where possible, but
/// [`LabelledResolve`] should be used as bounds in arguments.
pub trait LabelledResolveFrom<X>: LabelledResolvable {
    fn labelled_resolve_from(value: X, resolver: &impl LabelResolver<Self::ResolverOutput>)
        -> Self;
}

impl<X, Y: LabelledResolveFrom<X>> LabelledResolve<Y> for X {
    fn labelled_resolve(
        self,
        resolver: &impl LabelResolver<<Y as LabelledResolvable>::ResolverOutput>,
    ) -> Y {
        Y::labelled_resolve_from(self, resolver)
    }
}

/// `LabelledResolvable` is a marker trait, serving a few purposes:
/// * It avoids polluting every type with a resolve method
/// * It avoids trait definition collisions, by ensuring key types (e.g. &str) don't implement it.
/// * It allows providing [`ResolverOutput`] to establish what kind of resolver it works with.
///   This allows distinguishing "leaf" nodes which can be directly resolved from a resolver,
///   and have [`ResolverOutput`] equal to `Self`, from container types (e.g. `Option` and `Vec`
///   which don't have that bound).
///
/// If implementing this with [`ResolverOutput`] = `Self`, you will likely want to
/// use [`labelled_resolvable_with_identity_impl`] or [`labelled_resolvable_with_try_into_impls`]
/// to implement this trait and a reflexive or blanket impl using `try_into`.
///
/// [`ResolverOutput`]: LabelledResolvable::ResolverOutput
pub trait LabelledResolvable {
    /// You'll be passed a resolver, what will the resolver output?
    /// Often this will be `Self`, but sometimes it will be another type which you will
    /// need to map into `Self`.
    type ResolverOutput;
}

pub trait LabelResolver<X> {
    fn resolve_label_into(&self, label: &str) -> X;
}

#[macro_export]
macro_rules! labelled_resolvable_with_identity_impl {
    ($ty:ty, resolver_output: $resolver_output:ty$(,)?) => {
        impl LabelledResolvable for $ty {
            type ResolverOutput = $resolver_output;
        }

        impl LabelledResolveFrom<$ty> for $ty {
            fn labelled_resolve_from(
                value: Self,
                _resolver: &impl LabelResolver<$resolver_output>,
            ) -> Self {
                value
            }
        }

        // In future, could likely add an implementation from &$ty if $ty is Clone;
        // if we can get around the "trivially true/false" bound.
    };
}

#[macro_export]
macro_rules! labelled_resolvable_using_resolvable_impl {
    ($ty:ty, resolver_output: $resolver_output:ty$(,)?) => {
        impl LabelledResolvable for $ty {
            type ResolverOutput = $resolver_output;
        }

        impl<T: Resolve<Self>> LabelledResolveFrom<T> for $ty {
            fn labelled_resolve_from(
                value: T,
                _resolver: &impl LabelResolver<$resolver_output>,
            ) -> Self {
                value.resolve()
            }
        }
    };
}

//==============================================================
// If a type `X` has `ResolverOutput = Self` then it's a "leaf" - i.e. the thing
// that's ultimately being resolved.
// * We leave an identity resolver or try_into resolver for the macros `labelled_resolvable_with_identity_impl`
//   or `labelled_resolvable_with_try_into_impls` or `labelled_resolvable_using_resolvable_impl`
// * Implement resolves form string-based labels
//==============================================================

// Ideally we'd be able to allow `ResolverOutput = TryInfo<X>`, but the
// compiler disallows this, due to clashes with other blanket implementations.
// For example, it might be possible in future for e.g. &'a str to implement
// `IntoIterator<Item = A>` (e.g. A = &'a char) and for `A` to implement
// `Resolve<X>` and so give clashing implementations of
// LabelledResolveFrom<&'a str> for Vec<char>.

impl<'a, X: LabelledResolvable<ResolverOutput = X>> LabelledResolveFrom<&'a str> for X {
    fn labelled_resolve_from(value: &'a str, resolver: &impl LabelResolver<X>) -> X {
        resolver.resolve_label_into(value)
    }
}

impl<'a, X: LabelledResolvable<ResolverOutput = X>> LabelledResolveFrom<&'a String> for X {
    fn labelled_resolve_from(value: &'a String, resolver: &impl LabelResolver<X>) -> X {
        resolver.resolve_label_into(value.as_str())
    }
}

impl<X: LabelledResolvable<ResolverOutput = X>> LabelledResolveFrom<String> for X {
    fn labelled_resolve_from(value: String, resolver: &impl LabelResolver<X>) -> X {
        resolver.resolve_label_into(value.as_str())
    }
}

//==============================================================
// Handle Option<X>
//==============================================================
// - None and Some(X) are handled by the identity above
// - We then handle label -> Some(X) below
//==============================================================

impl<X: LabelledResolvable> LabelledResolvable for Option<X> {
    type ResolverOutput = X;
}

impl<X: LabelledResolvable> LabelledResolveFrom<Option<X>> for Option<X> {
    fn labelled_resolve_from(value: Option<X>, _resolver: &impl LabelResolver<X>) -> Option<X> {
        value
    }
}

impl<X: LabelledResolvable> LabelledResolveFrom<X> for Option<X> {
    fn labelled_resolve_from(value: X, _resolver: &impl LabelResolver<X>) -> Option<X> {
        Some(value)
    }
}

impl<'a, X: LabelledResolvable + Clone> LabelledResolveFrom<&'a X> for Option<X> {
    fn labelled_resolve_from(value: &'a X, _resolver: &impl LabelResolver<X>) -> Option<X> {
        Some(value.clone())
    }
}

impl<'a, X: LabelledResolvable + Clone> LabelledResolveFrom<&'a Option<X>> for Option<X> {
    fn labelled_resolve_from(value: &'a Option<X>, _resolver: &impl LabelResolver<X>) -> Option<X> {
        value.clone()
    }
}

impl<'a, X: LabelledResolvable> LabelledResolveFrom<&'a str> for Option<X> {
    fn labelled_resolve_from(value: &'a str, resolver: &impl LabelResolver<X>) -> Option<X> {
        Some(resolver.resolve_label_into(value))
    }
}

impl<'a, X: LabelledResolvable> LabelledResolveFrom<&'a String> for Option<X> {
    fn labelled_resolve_from(value: &'a String, resolver: &impl LabelResolver<X>) -> Option<X> {
        Some(resolver.resolve_label_into(value.as_str()))
    }
}

impl<'a, X: LabelledResolvable> LabelledResolveFrom<String> for Option<X> {
    fn labelled_resolve_from(value: String, resolver: &impl LabelResolver<X>) -> Option<X> {
        Some(resolver.resolve_label_into(value.as_str()))
    }
}

//==============================================================
// Handle collections
//==============================================================
// - An iterator over something that resolves to X, resolves to
//   the given collection/s of X.
// Feel free to add more collections here as needed.
//==============================================================

impl<X: LabelledResolvable> LabelledResolvable for Vec<X> {
    type ResolverOutput = X;
}

impl<T, X> LabelledResolveFrom<T> for Vec<X>
where
    T: IntoIterator,
    T::Item: LabelledResolve<X>,
    X: LabelledResolvable<ResolverOutput = X>,
{
    fn labelled_resolve_from(value: T, resolver: &impl LabelResolver<X>) -> Vec<X> {
        value
            .into_iter()
            .map(|item| LabelledResolve::<X>::labelled_resolve(item, resolver))
            .collect()
    }
}

impl<X: LabelledResolvable> LabelledResolvable for IndexSet<X> {
    type ResolverOutput = X;
}

impl<T, X> LabelledResolveFrom<T> for IndexSet<X>
where
    T: IntoIterator,
    T::Item: LabelledResolve<X>,
    X: LabelledResolvable<ResolverOutput = X> + core::hash::Hash + core::cmp::Eq,
{
    fn labelled_resolve_from(value: T, resolver: &impl LabelResolver<X>) -> IndexSet<X> {
        value
            .into_iter()
            .map(|item| LabelledResolve::<X>::labelled_resolve(item, resolver))
            .collect()
    }
}
