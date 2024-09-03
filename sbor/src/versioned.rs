use crate::internal_prelude::*;

/// A trait implemented by versioned types created via [`define_versioned`] and [`define_single_versioned`].
///
/// A versioned type is a type wrapping an enum, this enum is the associated type [`Versioned::Versions`],
/// and contains a variant for each supported version.
///
/// This [`Versioned`] type itself is a struct wrapper around this enum, which allows for fully updating
/// the contained version to [`Versioned::LatestVersion`]. This wrapper is required so that the wrapper
/// can take ownership of old versions as part of the upgrade process, in order to incrementally update
/// them using the [`From`] trait.
pub trait Versioned: AsRef<Self::Versions> + AsMut<Self::Versions> + From<Self::Versions> {
    /// The type for the enum of versions.
    type Versions: From<Self>;

    /// The type for the latest content.
    type LatestVersion;

    /// Returns true if at the latest version.
    fn is_fully_updated(&self) -> bool;

    /// Updates the latest version in place, and returns a `&mut` to the latest content
    fn in_place_fully_update_and_as_latest_version_mut(&mut self) -> &mut Self::LatestVersion {
        self.in_place_fully_update();
        self.as_latest_version_mut().unwrap()
    }

    /// Updates to the latest version in place.
    fn in_place_fully_update(&mut self) -> &mut Self;

    /// Consumes self, updates to the latest version and returns itself.
    fn fully_update(mut self) -> Self {
        self.in_place_fully_update();
        self
    }

    /// Updates itself to the latest version, then returns the latest content
    fn fully_update_and_into_latest_version(self) -> Self::LatestVersion;

    /// Constructs a versioned wrapper around the latest content
    fn from_latest_version(latest: Self::LatestVersion) -> Self;

    /// If the versioned wrapper is at the latest version, it returns
    /// an immutable reference to the latest content, otherwise it returns `None`.
    ///
    /// If you require the latest version unconditionally, consider using
    /// [`in_place_fully_update_and_as_latest_version_mut`] to update to the latest version first - or, if
    /// there is only a single version, use [`as_unique_version`].
    fn as_latest_version(&self) -> Option<&Self::LatestVersion>;

    /// If the versioned wrapper is at the latest version, it returns
    /// a mutable reference to the latest content, otherwise it returns `None`.
    ///
    /// If you require the latest version unconditionally, consider using
    /// [`in_place_fully_update_and_as_latest_version_mut`] to update to the latest version first  - or, if
    /// there is only a single version, use [`as_unique_version_mut`].
    fn as_latest_version_mut(&mut self) -> Option<&mut Self::LatestVersion>;

    /// Gets a reference the inner versions enum, for e.g. matching on the enum.
    ///
    /// This is essentially a clearer alias for `as_ref`.
    fn as_versions(&self) -> &Self::Versions;

    /// Gets a mutable reference the inner versions enum, for e.g. matching on the enum.
    ///
    /// This is essentially a clearer alias for `as_mut`.
    fn as_versions_mut(&mut self) -> &mut Self::Versions;

    /// Removes the upgradable wrapper to get at the inner versions enum, for e.g. matching on the enum.
    fn into_versions(self) -> Self::Versions;

    /// Creates a new Versioned wrapper from a given specific version.
    fn from_versions(version: Self::Versions) -> Self;
}

/// A trait for Versioned types which only have a single version.
///
/// This enables a number of special-cased methods to be implemented which are only possible when there
/// is only one version.
pub trait UniqueVersioned: Versioned {
    /// Returns an immutable reference to (currently) the only possible version of the inner content.
    fn as_unique_version(&self) -> &Self::LatestVersion;

    /// Returns a mutable reference to (currently) the only possible version of the inner content.
    ///
    /// This is somewhat equivalent to `in_place_fully_update_and_as_latest_version_mut`, but doesn't need to do
    /// any updating, so can be used where logical correctness requires there to be a unique version,
    /// requires no updating, or simply for slightly better performance.
    fn as_unique_version_mut(&mut self) -> &mut Self::LatestVersion;

    /// Returns the (currently) only possible version of the inner content.
    ///
    /// This is somewhat equivalent to `fully_update_and_into_latest_version`, but doesn't need to do
    /// any updating, so can be used where logical correctness requires there to be a unique version,
    /// requires no updating, or simply for slightly better performance.
    fn into_unique_version(self) -> Self::LatestVersion;

    /// Creates the versioned wrapper from the (currently) only possible version.
    ///
    /// This is equivalent to `from_latest_version`, but useful to use instead if your logic's correctness
    /// is dependent on there only being a single version. If another version gets added, this
    /// method will give a compile error.
    fn from_unique_version(unique_version: Self::LatestVersion) -> Self;
}

/// This macro is intended for creating a data model which supports versioning.
/// This is useful for creating an SBOR data model which can be updated in future.
///
/// In future, the type can be converted to `define_versioned`, enum variants can
/// be added, and automatically mapped to the latest version.
///
/// This macro is just a simpler wrapper around the [`define_versioned`] macro,
/// for use when there's just a single version.
///
/// ## Example usage
///
/// ```rust
/// use sbor::prelude::*;
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
/// pub struct FooV1 {
///    bar: u8,
/// }
///
/// define_single_versioned! {
///    #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
///    pub VersionedFoo(FooVersions) => Foo = FooV1
/// }
///
/// // `Foo` is created as an alias for `FooV1`
/// let a = Foo { bar: 42 }.into_versioned();
/// let a3 = VersionedFoo::from(FooVersions::V1(FooV1 { bar: 42 }));
/// let a2 = VersionedFoo::from_unique_version(Foo { bar: 42 });
///
/// assert_eq!(a, a2);
/// assert_eq!(a2, a3);
/// assert_eq!(42, a.as_unique_version().bar);
/// ```
///
/// ## Advanced attribute handling
///
/// Note that the provided attributes get applied to _both_ the outer "Versioned" type,
/// and the inner "Versions" type. To only apply to one type, you can include the
/// `outer_attributes` optional argument and/or the `inner_attributes` optional argument:
/// ```
/// # use sbor::prelude::*;
/// # #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
/// # pub struct FooV1;
/// define_single_versioned! {
///    #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
///    pub VersionedFoo(FooVersions) => Foo = FooV1,
///    outer_attributes: [
///        #[sbor(type_name = "MyVersionedFoo")]
///    ],
///    inner_attributes: [
///        #[sbor(type_name = "MyFooVersions")]
///    ],
/// }
/// ```
#[macro_export]
macro_rules! define_single_versioned {
    (
        $(#[$attributes:meta])*
        $vis:vis $versioned_name:ident(
            $versions_name:ident
        )
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
        =>
        $latest_version_alias:ty = $latest_version_type:ty
        $(, outer_attributes: [
            $(#[$outer_attributes:meta])*
        ])?
        $(, inner_attributes: [
            $(#[$inner_attributes:meta])*
        ])?
        $(,)?
    ) => {
        $crate::define_versioned!(
            $(#[$attributes])*
            $vis $versioned_name($versions_name)
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                previous_versions: [],
                latest_version: {
                    1 => $latest_version_alias = $latest_version_type
                },
            }
            $(, outer_attributes: [
                $(#[$outer_attributes])*
            ])?
            $(, inner_attributes: [
                $(#[$inner_attributes])*
            ])?
        );

        $crate::paste::paste! {
            #[allow(dead_code)]
            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            UniqueVersioned
            for $versioned_name $(< $( $lt ),+ >)?
            {
                fn as_unique_version(&self) -> &Self::LatestVersion {
                    match self.as_ref() {
                        $versions_name $(::< $( $lt ),+ >)? ::V1(content) => content,
                    }
                }

                fn as_unique_version_mut(&mut self) -> &mut Self::LatestVersion {
                    match self.as_mut() {
                        $versions_name $(::< $( $lt ),+ >)? ::V1(content) => content,
                    }
                }

                fn into_unique_version(self) -> Self::LatestVersion {
                    match $versions_name $(::< $( $lt ),+ >)? ::from(self) {
                        $versions_name $(::< $( $lt ),+ >)? ::V1(content) => content,
                    }
                }

                fn from_unique_version(content: Self::LatestVersion) -> Self {
                    $versions_name $(::< $( $lt ),+ >)? ::V1(content).into()
                }
            }
        }
    };
}

/// This macro is intended for creating a data model which supports versioning.
/// This is useful for creating an SBOR data model which can be updated in future.
/// In future, enum variants can be added, and automatically mapped to.
///
/// NOTE: A circular version update chain will be an infinite loop at runtime. Be careful.
///
/// In the future, this may become a programmatic macro to support better error handling /
/// edge case detection, and opting into more explicit SBOR handling.
///
/// ## Example usage
///
/// ```rust
/// use sbor::prelude::*;
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
/// pub struct FooV1 {
///    bar: u8,
/// }
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
/// pub struct FooV2 {
///    bar: u8,
///    baz: Option<u8>,
/// }
///
/// impl From<FooV1> for FooV2 {
///     fn from(value: FooV1) -> FooV2 {
///         FooV2 {
///             bar: value.bar,
///             // Could also use `value.bar` as sensible default during inline update
///             baz: None,
///         }
///     }
/// }
///
/// define_versioned!(
///     #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
///     VersionedFoo(FooVersions) {
///         previous_versions: [
///             1 => FooV1: { updates_to: 2 },
///         ],
///         latest_version: {
///             2 => Foo = FooV2,
///         },
///     }
/// );
///
/// let mut a = FooV1 { bar: 42 }.into_versioned();
/// let equivalent_a = VersionedFoo::from(FooVersions::V1(FooV1 { bar: 42 }));
/// assert_eq!(a, equivalent_a);
///
/// // `Foo` is created as an alias for the latest content, `FooV2`
/// let b = VersionedFoo::from(FooVersions::V2(Foo { bar: 42, baz: None }));
///
/// assert_ne!(a, b);
/// assert_eq!(&*a.in_place_fully_update_and_as_latest_version_mut(), b.as_latest_version().unwrap());
///
/// // After a call to `a.in_place_fully_update_and_as_latest_version_mut()`, `a` has now been updated:
/// assert_eq!(a, b);
/// ```
///
/// ## Advanced attribute handling
///
/// The provided attributes get applied to _both_ the outer "Versioned" type,
/// and the inner "Versions" type. To only apply to one type, you can include the
/// `outer_attributes` optional argument and/or the `inner_attributes` optional argument:
/// ```
/// # use sbor::prelude::*;
/// # #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
/// # pub struct FooV1;
/// # #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
/// # pub struct FooV2;
/// # impl From<FooV1> for FooV2 {
/// #    fn from(value: FooV1) -> FooV2 {
/// #        FooV2
/// #    }
/// # }
///
/// define_versioned! {
///     #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
///     VersionedFoo(FooVersions) {
///         previous_versions: [
///             1 => FooV1: { updates_to: 2 },
///         ],
///         latest_version: {
///             2 => Foo = FooV2,
///         },
///     }
///     outer_attributes: [
///         #[sbor(type_name = "MyVersionedFoo")]
///     ],
///     inner_attributes: [
///         #[sbor(type_name = "MyFooVersions")]
///     ],
/// }
#[macro_export]
macro_rules! define_versioned {
    (
        $(#[$attributes:meta])*
        $vis:vis $versioned_name:ident($versions_name:ident)
        // Now match the optional type parameters
        // See https://stackoverflow.com/questions/41603424/rust-macro-accepting-type-with-generic-parameters
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
        {
            $(
                previous_versions: [
                    $($version_num:expr => $version_type:ty: { updates_to: $update_to_version_num:expr }),*
                    $(,)? // Optional trailing comma
                ],
            )?
            latest_version: {
                $latest_version:expr => $latest_version_alias:ty = $latest_version_type:ty
                $(,)? // Optional trailing comma
            }
            $(,)? // Optional trailing comma
        }
        $(,)?
        $(outer_attributes: [
            $(#[$outer_attributes:meta])*
        ])?
        $(, inner_attributes: [
            $(#[$inner_attributes:meta])*
        ])?
        $(,)?
    ) => {
        $crate::eager_replace! {
            [!SET! #FullGenerics = $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?]
            [!SET! #ImplGenerics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
            [!SET! #TypeGenerics = $(< $( $lt ),+ >)?]
            [!SET! #VersionedType = $versioned_name $(< $( $lt ),+ >)?]
            [!SET! #VersionedTypePath = $versioned_name $(::< $( $lt ),+ >)?]
            [!SET! #VersionsType = $versions_name $(< $( $lt ),+ >)?]
            [!SET! #VersionsTypePath = $versions_name $(::< $( $lt ),+ >)?]
            [!SET:ident! #PermitSborAttributesAlias = $versioned_name _PermitSborAttributes]

            #[allow(dead_code)]
            $vis type $latest_version_alias = $latest_version_type;

            use $crate::PermitSborAttributes as #PermitSborAttributesAlias;

            #[derive(#PermitSborAttributesAlias)]
            $(#[$attributes])*
            $($(#[$outer_attributes])*)?
            // Needs to go below $attributes so that a #[derive(Sbor)] in the attributes can see it.
            #[sbor(as_type = [!stringify! #VersionsType])]
            /// If you wish to get access to match on the versions, use `.as_ref()` or `.as_mut()`.
            $vis struct $versioned_name #FullGenerics
            {
                inner: Option<#VersionsType>,
            }

            impl #ImplGenerics #VersionedType
            {
                pub fn new(inner: #VersionsType) -> Self {
                    Self {
                        inner: Some(inner),
                    }
                }
            }

            impl #ImplGenerics AsRef<#VersionsType> for #VersionedType
            {
                fn as_ref(&self) -> &#VersionsType {
                    self.inner.as_ref().unwrap()
                }
            }

            impl #ImplGenerics AsMut<#VersionsType> for #VersionedType
            {
                fn as_mut(&mut self) -> &mut #VersionsType {
                    self.inner.as_mut().unwrap()
                }
            }

            impl #ImplGenerics From<#VersionsType> for #VersionedType
            {
                fn from(value: #VersionsType) -> Self {
                    Self::new(value)
                }
            }

            impl #ImplGenerics From<#VersionedType> for #VersionsType
            {
                fn from(value: #VersionedType) -> Self {
                    value.inner.unwrap()
                }
            }

            impl #ImplGenerics Versioned for #VersionedType
            {
                type Versions = #VersionsType;
                type LatestVersion = $latest_version_type;

                fn is_fully_updated(&self) -> bool {
                    self.as_ref().is_fully_updated()
                }

                fn in_place_fully_update(&mut self) -> &mut Self {
                    if !self.is_fully_updated() {
                        let current = self.inner.take().unwrap();
                        self.inner = Some(current.fully_update());
                    }
                    self
                }

                fn fully_update_and_into_latest_version(self) -> Self::LatestVersion {
                    self.inner.unwrap().fully_update_and_into_latest_version()
                }

                /// Constructs the versioned enum from the latest content
                fn from_latest_version(latest: Self::LatestVersion) -> Self {
                    Self::new(latest.into())
                }

                fn as_latest_version(&self) -> Option<&Self::LatestVersion> {
                    self.as_ref().as_latest_version()
                }

                fn as_latest_version_mut(&mut self) -> Option<&mut Self::LatestVersion> {
                    self.as_mut().as_latest_version_mut()
                }

                fn as_versions(&self) -> &Self::Versions {
                    self.as_ref()
                }

                fn as_versions_mut(&mut self) -> &mut Self::Versions {
                    self.as_mut()
                }

                fn into_versions(self) -> Self::Versions {
                    self.inner.unwrap()
                }

                fn from_versions(version: Self::Versions) -> Self {
                    Self::new(version)
                }
            }

            [!SET:ident! #discriminators = $versioned_name _discriminators]
            #[allow(non_snake_case)]
            mod #discriminators {
                // The initial version of this tool used 0-indexed/off-by-one discriminators accidentally.
                // We're stuck with these now unfortunately...
                // But we make them explicit in case versions are skipped.
                $($(
                    pub const [!ident! VERSION_ $version_num]: u8 = $version_num - 1;
                )*)?
                pub const LATEST_VERSION: u8 = $latest_version - 1;
            }

            #[derive(#PermitSborAttributesAlias)]
            $(#[$attributes])*
            $($(#[$inner_attributes])*)?
            $vis enum $versions_name #FullGenerics
            {
                $($(
                    #[sbor(discriminator(#discriminators::[!ident! VERSION_ $version_num]))]
                    [!ident! V $version_num]($version_type),
                )*)?
                #[sbor(discriminator(#discriminators::LATEST_VERSION))]
                [!ident! V $latest_version]($latest_version_type),
            }

            #[allow(dead_code)]
            impl #ImplGenerics #VersionsType
            {
                /// Returns if update happened, and the updated versioned enum.
                fn attempt_single_update(self) -> (bool, Self) {
                    match self {
                    $($(
                        Self::[!ident! V $version_num](value) => (true, Self::[!ident! V $update_to_version_num](value.into())),
                    )*)?
                        this @ Self::[!ident! V $latest_version](_) => (false, this),
                    }
                }

                fn fully_update(mut self) -> Self {
                    loop {
                        let (did_update, updated) = self.attempt_single_update();
                        if did_update {
                            // We should try updating
                            self = updated;
                        } else {
                            // We're at latest - return
                            return updated;
                        }
                    }
                }

                #[allow(unreachable_patterns)]
                pub fn is_fully_updated(&self) -> bool {
                    match self {
                        Self::[!ident! V $latest_version](_) => true,
                        _ => false,
                    }
                }

                #[allow(irrefutable_let_patterns)]
                fn fully_update_and_into_latest_version(self) -> $latest_version_type {
                    let Self::[!ident! V $latest_version](latest) = self.fully_update() else {
                        panic!("Invalid resolved latest version not equal to latest type")
                    };
                    return latest;
                }

                fn from_latest_version(latest: $latest_version_type) -> Self {
                    Self::[!ident! V $latest_version](latest)
                }

                #[allow(unreachable_patterns)]
                fn as_latest_version(&self) -> Option<&$latest_version_type> {
                    match self {
                        Self::[!ident! V $latest_version](latest) => Some(latest),
                        _ => None,
                    }
                }

                #[allow(unreachable_patterns)]
                fn as_latest_version_mut(&mut self) -> Option<&mut $latest_version_type> {
                    match self {
                        Self::[!ident! V $latest_version](latest) => Some(latest),
                        _ => None,
                    }
                }

                pub fn into_versioned(self) -> #VersionedType {
                    #VersionedTypePath::new(self)
                }
            }

            $($(
                #[allow(dead_code)]
                impl #ImplGenerics From<$version_type> for #VersionsType {
                    fn from(value: $version_type) -> Self {
                        Self::[!ident! V $version_num](value)
                    }
                }

                #[allow(dead_code)]
                impl #ImplGenerics From<$version_type> for #VersionedType {
                    fn from(value: $version_type) -> Self {
                        Self::new(#VersionsTypePath::[!ident! V $version_num](value))
                    }
                }
            )*)?

            #[allow(dead_code)]
            impl #ImplGenerics From<$latest_version_type> for #VersionsType {
                fn from(value: $latest_version_type) -> Self {
                    Self::[!ident! V $latest_version](value)
                }
            }

            #[allow(dead_code)]
            impl #ImplGenerics From<$latest_version_type> for #VersionedType {
                fn from(value: $latest_version_type) -> Self {
                    Self::new($versions_name::[!ident! V $latest_version](value))
                }
            }

            // This trait is similar to `SborEnumVariantFor<X, Versioned>`, but it's nicer to use as
            // it's got a better name and can be implemented without needing a specific CustomValueKind.
            [!SET:ident! #VersionTrait = $versioned_name Version]
            #[allow(dead_code)]
            $vis trait #VersionTrait {
                // Note: We need to use an explicit associated type to capture the generics.
                type Versioned: sbor::Versioned;

                const DISCRIMINATOR: u8;
                type OwnedSborVariant;
                type BorrowedSborVariant<'a> where Self: 'a;

                /// Can be used to encode the type as a variant under the Versioned type, without
                /// needing to clone, like this: `encoder.encode(x.as_encodable_variant())`.
                fn as_encodable_variant(&self) -> Self::BorrowedSborVariant<'_>;

                /// Can be used to decode the type from an encoded variant, like this:
                /// `X::from_decoded_variant(decoder.decode()?)`.
                fn from_decoded_variant(variant: Self::OwnedSborVariant) -> Self where Self: core::marker::Sized;

                fn into_versioned(self) -> Self::Versioned;
            }

            $($(
                impl #ImplGenerics #VersionTrait for $version_type
                {
                    type Versioned = #VersionedType;

                    const DISCRIMINATOR: u8 = #discriminators::[!ident! VERSION_ $version_num];
                    type OwnedSborVariant = sbor::SborFixedEnumVariant::<{ #discriminators::[!ident! VERSION_ $version_num] }, (Self,)>;
                    type BorrowedSborVariant<'a> = sbor::SborFixedEnumVariant::<{ #discriminators::[!ident! VERSION_ $version_num] }, (&'a Self,)>  where Self: 'a;

                    fn as_encodable_variant(&self) -> Self::BorrowedSborVariant<'_> {
                        sbor::SborFixedEnumVariant::new((self,))
                    }

                    fn from_decoded_variant(variant: Self::OwnedSborVariant) -> Self {
                        variant.into_fields().0
                    }

                    fn into_versioned(self) -> Self::Versioned {
                        #VersionedTypePath::new(self.into())
                    }
                }
            )*)?

            impl #ImplGenerics #VersionTrait for $latest_version_type
            {
                type Versioned = $versioned_name #TypeGenerics;

                const DISCRIMINATOR: u8 = #discriminators::LATEST_VERSION;
                type OwnedSborVariant = sbor::SborFixedEnumVariant::<{ #discriminators::LATEST_VERSION }, (Self,)>;
                type BorrowedSborVariant<'a> = sbor::SborFixedEnumVariant::<{ #discriminators::LATEST_VERSION }, (&'a Self,)> where Self: 'a;

                fn as_encodable_variant(&self) -> Self::BorrowedSborVariant<'_> {
                    sbor::SborFixedEnumVariant::new((self,))
                }

                fn from_decoded_variant(variant: Self::OwnedSborVariant) -> Self {
                    variant.into_fields().0
                }

                fn into_versioned(self) -> Self::Versioned {
                    #VersionedTypePath::new(self.into())
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    crate::define_versioned!(
        #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
        VersionedExample(ExampleVersions) {
            previous_versions: [
                1 => ExampleV1: { updates_to: 2 },
                2 => ExampleV2: { updates_to: 4 },
                3 => ExampleV3: { updates_to: 4 },
            ],
            latest_version: {
                4 => Example = ExampleV4,
            },
        }
    );

    // Define the concrete versions
    type ExampleV1 = u8;
    type ExampleV2 = u16;

    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    struct ExampleV3(u16);

    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    struct ExampleV4 {
        the_value: u16,
    }

    impl ExampleV4 {
        pub fn of(value: u16) -> Self {
            Self { the_value: value }
        }
    }

    // And explicit updates between them, which are needed
    // for the versioned type
    impl From<ExampleV2> for ExampleV4 {
        fn from(value: ExampleV2) -> Self {
            Self { the_value: value }
        }
    }

    impl From<ExampleV3> for ExampleV4 {
        fn from(value: ExampleV3) -> Self {
            Self { the_value: value.0 }
        }
    }

    #[test]
    pub fn updates_to_latest_work() {
        let expected_latest = ExampleV4::of(5);
        let v1: ExampleV1 = 5;
        validate_latest(v1, expected_latest.clone());
        let v2: ExampleV2 = 5;
        validate_latest(v2, expected_latest.clone());
        let v3 = ExampleV3(5);
        validate_latest(v3, expected_latest.clone());
        let v4 = ExampleV4::of(5);
        validate_latest(v4, expected_latest);
    }

    fn validate_latest(
        actual: impl Into<VersionedExample>,
        expected: <VersionedExample as Versioned>::LatestVersion,
    ) {
        let versioned_actual = actual.into().fully_update();
        let versioned_expected = VersionedExample::from(expected.clone());
        // Check fully_update (which returns a VersionedExample)
        assert_eq!(versioned_actual, versioned_expected,);
        // Check fully_update_and_into_latest_version (which returns an ExampleV4)
        assert_eq!(
            versioned_actual.fully_update_and_into_latest_version(),
            expected,
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    struct GenericModelV1<T>(T);

    define_single_versioned!(
        /// This is some rust doc as an example annotation
        #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
        VersionedGenericModel(GenericModelVersions)<T> => GenericModel<T> = GenericModelV1<T>
    );

    #[test]
    pub fn generated_single_versioned_works() {
        let v1_model: GenericModel<_> = GenericModelV1(51u64);
        let versioned = VersionedGenericModel::from(v1_model.clone());
        let versioned_2 = v1_model.clone().into_versioned();
        assert_eq!(
            versioned.clone().fully_update_and_into_latest_version(),
            v1_model
        );
        assert_eq!(versioned, versioned_2);
    }

    #[test]
    pub fn verify_sbor_equivalence() {
        // Value model
        let v1_model: GenericModel<_> = GenericModelV1(51u64);
        let versions = GenericModelVersions::V1(v1_model.clone());
        let versioned = VersionedGenericModel::from(v1_model.clone());
        let expected_sbor_value = BasicEnumVariantValue {
            // GenericModelVersions
            discriminator: 0, // V1 maps to 0 for legacy compatibility
            fields: vec![
                // GenericModelV1
                Value::Tuple {
                    fields: vec![Value::U64 { value: 51 }],
                },
            ],
        };
        let encoded_versioned = basic_encode(&versioned).unwrap();
        let encoded_versions = basic_encode(&versions).unwrap();
        let expected = basic_encode(&expected_sbor_value).unwrap();
        assert_eq!(encoded_versioned, expected);
        assert_eq!(encoded_versions, expected);

        // Type model
        check_identical_types::<VersionedGenericModel<u64>, GenericModelVersions<u64>>(Some(
            "VersionedGenericModel",
        ));
    }

    fn check_identical_types<T1: Describe<NoCustomTypeKind>, T2: Describe<NoCustomTypeKind>>(
        name: Option<&'static str>,
    ) {
        let (type_id1, schema1) = generate_full_schema_from_single_type::<T1, NoCustomSchema>();
        let (type_id2, schema2) = generate_full_schema_from_single_type::<T2, NoCustomSchema>();

        assert_eq!(
            schema1.v1().resolve_type_kind(type_id1),
            schema2.v1().resolve_type_kind(type_id2)
        );
        assert_eq!(
            schema1
                .v1()
                .resolve_type_metadata(type_id1)
                .unwrap()
                .clone(),
            schema2
                .v1()
                .resolve_type_metadata(type_id2)
                .unwrap()
                .clone()
                .with_name(name.map(|name| Cow::Borrowed(name)))
        );
        assert_eq!(
            schema1.v1().resolve_type_validation(type_id1),
            schema2.v1().resolve_type_validation(type_id2)
        );
    }
}
