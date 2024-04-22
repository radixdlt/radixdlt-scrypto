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
    fn fully_update_to_latest_version_mut(&mut self) -> &mut Self::LatestVersion {
        self.fully_update();
        self.as_latest_version_mut().unwrap()
    }

    /// Updates to the latest version in place.
    fn fully_update(&mut self);

    /// Updates itself to the latest version, then returns the latest content
    fn fully_update_into_latest_version(self) -> Self::LatestVersion;

    /// Constructs a versioned wrapper around the latest content
    fn from_latest_version(latest: Self::LatestVersion) -> Self;

    /// If the versioned wrapper is at the latest version, it returns
    /// an immutable reference to the latest content, otherwise it returns `None`.
    ///
    /// If you require the latest version unconditionally, consider using
    /// [`fully_update_to_latest_version_mut`] to update to the latest version first - or, if
    /// there is only a single version, use [`as_unique_version_ref`].
    fn as_latest_version_ref(&self) -> Option<&Self::LatestVersion>;

    /// If the versioned wrapper is at the latest version, it returns
    /// a mutable reference to the latest content, otherwise it returns `None`.
    ///
    /// If you require the latest version unconditionally, consider using
    /// [`fully_update_to_latest_version_mut`] to update to the latest version first  - or, if
    /// there is only a single version, use [`as_unique_version_mut`].
    fn as_latest_version_mut(&mut self) -> Option<&mut Self::LatestVersion>;

    /// Gets a reference the inner versions enum, for e.g. matching on the enum.
    ///
    /// This is essentially a clearer alias for `as_ref`.
    fn as_versions_ref(&self) -> &Self::Versions;

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
    fn as_unique_version_ref(&self) -> &Self::LatestVersion;

    /// Returns a mutable reference to (currently) the only possible version of the inner content.
    ///
    /// This is somewhat equivalent to `fully_update_to_latest_version_mut`, but doesn't need to do
    /// any updating, so can be used where logical correctness requires there to be a unique version,
    /// requires no updating, or simply for slightly better performance.
    fn as_unique_version_mut(&mut self) -> &mut Self::LatestVersion;

    /// Returns the (currently) only possible version of the inner content.
    ///
    /// This is somewhat equivalent to `fully_update_into_latest_version`, but doesn't need to do
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
/// Example usage:
/// ```rust
/// use ::sbor::prelude::*;
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
/// assert_eq!(42, a.as_unique_version_ref().bar);
/// ```
#[macro_export]
macro_rules! define_single_versioned {
    (
        $(#[$attributes:meta])*
        $vis:vis $versioned_name:ident($versions_name:ident)
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
        =>
        $latest_version_alias:ty = $latest_version_type:ty
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
        );

        $crate::paste::paste! {
            #[allow(dead_code)]
            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            UniqueVersioned
            for $versioned_name $(< $( $lt ),+ >)?
            {
                fn as_unique_version_ref(&self) -> &Self::LatestVersion {
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
/// Example usage:
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
/// assert_eq!(&*a.fully_update_to_latest_version_mut(), b.as_latest_version_ref().unwrap());
///
/// // After a call to `a.fully_update_to_latest_version_mut()`, `a` has now been updated:
/// assert_eq!(a, b);
/// ```
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
    ) => {
        $crate::eager_replace! {
        $crate::paste::paste! {
            // Create inline sub-macros to handle the type generics nested inside
            // iteration over previous_versions
            // See eg https://stackoverflow.com/a/73543948
            macro_rules! [<$versioned_name _versions_trait_impl>] {
                (
                    $trait:ty,
                    $impl_block:tt
                ) => {
                    #[allow(dead_code)]
                    impl
                    $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
                    $trait
                    for $versions_name $(< $( $lt ),+ >)?
                    $impl_block
                };
            }

            macro_rules! [<$versioned_name _versioned_trait_impl>] {
                (
                    $trait:ty,
                    $impl_block:tt
                ) => {
                    #[allow(dead_code)]
                    impl
                    $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
                    $trait
                    for $versioned_name $(< $( $lt ),+ >)?
                    $impl_block
                };
            }

            #[allow(dead_code)]
            $vis type $latest_version_alias = $latest_version_type;

            use $crate::PermitSborAttributes as [<$versioned_name _PermitSborAttributes>];

            #[derive([<$versioned_name _PermitSborAttributes>])]
            $(#[$attributes])*
            // Needs to go below $attributes so that a #[derive(Sbor)] in the attributes can see it.
            #[sbor(as_type = eager_stringify!($versions_name $(< $( $lt ),+ >)?))]
            /// If you wish to get access to match on the versions, use `.as_ref()` or `.as_mut()`.
            $vis struct $versioned_name $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                inner: Option<$versions_name $(< $( $lt ),+ >)?>,
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $versioned_name $(< $( $lt ),+ >)?
            {
                pub fn new(inner: $versions_name $(< $( $lt ),+ >)?) -> Self {
                    Self {
                        inner: Some(inner),
                    }
                }
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            AsRef<$versions_name $(< $( $lt ),+ >)?>
            for $versioned_name $(< $( $lt ),+ >)?
            {
                fn as_ref(&self) -> &$versions_name $(< $( $lt ),+ >)? {
                    self.inner.as_ref().unwrap()
                }
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            AsMut<$versions_name $(< $( $lt ),+ >)?>
            for $versioned_name $(< $( $lt ),+ >)?
            {
                fn as_mut(&mut self) -> &mut $versions_name $(< $( $lt ),+ >)? {
                    self.inner.as_mut().unwrap()
                }
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            From<$versions_name $(< $( $lt ),+ >)?>
            for $versioned_name $(< $( $lt ),+ >)?
            {
                fn from(value: $versions_name $(< $( $lt ),+ >)?) -> Self {
                    Self::new(value)
                }
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            From<$versioned_name $(< $( $lt ),+ >)?>
            for $versions_name $(< $( $lt ),+ >)?
            {
                fn from(value: $versioned_name $(< $( $lt ),+ >)?) -> Self {
                    value.inner.unwrap()
                }
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            Versioned
            for $versioned_name $(< $( $lt ),+ >)?
            {
                type Versions = $versions_name $(< $( $lt ),+ >)?;
                type LatestVersion = $latest_version_type;

                fn is_fully_updated(&self) -> bool {
                    self.as_ref().is_fully_updated()
                }

                fn fully_update(&mut self) {
                    if !self.is_fully_updated() {
                        let current = self.inner.take().unwrap();
                        self.inner = Some(current.fully_update());
                    }
                }

                fn fully_update_into_latest_version(self) -> Self::LatestVersion {
                    self.inner.unwrap().fully_update_into_latest_version()
                }

                /// Constructs the versioned enum from the latest content
                fn from_latest_version(latest: Self::LatestVersion) -> Self {
                    Self::new(latest.into())
                }

                fn as_latest_version_ref(&self) -> Option<&Self::LatestVersion> {
                    self.as_ref().as_latest_version_ref()
                }

                fn as_latest_version_mut(&mut self) -> Option<&mut Self::LatestVersion> {
                    self.as_mut().as_latest_version_mut()
                }

                fn as_versions_ref(&self) -> &Self::Versions {
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

            #[derive([<$versioned_name _PermitSborAttributes>])]
            $(#[$attributes])*
            $vis enum $versions_name $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                $($(
                    #[sbor(discriminator($version_num))]
                    [<V $version_num>]($version_type),
                )*)?
                #[sbor(discriminator($latest_version))]
                [<V $latest_version>]($latest_version_type),
            }

            #[allow(dead_code)]
            impl
            $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $versions_name
            $(< $( $lt ),+ >)?
            {
                /// Returns if update happened, and the updated versioned enum.
                fn attempt_single_update(self) -> (bool, Self) {
                    match self {
                    $($(
                        Self::[<V $version_num>](value) => (true, Self::[<V $update_to_version_num>](value.into())),
                    )*)?
                        this @ Self::[<V $latest_version>](_) => (false, this),
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
                        Self::[<V $latest_version>](_) => true,
                        _ => false,
                    }
                }

                #[allow(irrefutable_let_patterns)]
                fn fully_update_into_latest_version(self) -> $latest_version_type {
                    let Self::[<V $latest_version>](latest) = self.fully_update() else {
                        panic!("Invalid resolved latest version not equal to latest type")
                    };
                    return latest;
                }

                fn from_latest_version(latest: $latest_version_type) -> Self {
                    Self::[<V $latest_version>](latest)
                }

                #[allow(unreachable_patterns)]
                fn as_latest_version_ref(&self) -> Option<&$latest_version_type> {
                    match self {
                        Self::[<V $latest_version>](latest) => Some(latest),
                        _ => None,
                    }
                }

                #[allow(unreachable_patterns)]
                fn as_latest_version_mut(&mut self) -> Option<&mut $latest_version_type> {
                    match self {
                        Self::[<V $latest_version>](latest) => Some(latest),
                        _ => None,
                    }
                }
            }

            $($([<$versioned_name _versions_trait_impl>]!(
                From<$version_type>,
                {
                    fn from(value: $version_type) -> Self {
                        Self::[<V $version_num>](value)
                    }
                }
            );)*)?

            $($([<$versioned_name _versioned_trait_impl>]!(
                From<$version_type>,
                {
                    fn from(value: $version_type) -> Self {
                        Self::new($versions_name::[<V $version_num>](value))
                    }
                }
            );)*)?

            [<$versioned_name _versions_trait_impl>]!(
                From<$latest_version_type>,
                {
                    fn from(value: $latest_version_type) -> Self {
                        Self::[<V $latest_version>](value)
                    }
                }
            );

            [<$versioned_name _versioned_trait_impl>]!(
                From<$latest_version_type>,
                {
                    fn from(value: $latest_version_type) -> Self {
                        Self::from($versions_name::[<V $latest_version>](value))
                    }
                }
            );

            #[allow(dead_code)]
            $vis trait [<$versioned_name Version>] {
                // Note: We need to use an explicit associated type to capture the generics.
                type Versioned: $crate::Versioned;

                fn into_versioned(self) -> Self::Versioned;
            }

            impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            [<$versioned_name Version>]
            for $versions_name $(< $( $lt ),+ >)?
            {
                type Versioned = $versioned_name $(< $( $lt ),+ >)?;

                fn into_versioned(self) -> Self::Versioned {
                    $versioned_name $(::< $( $lt ),+ >)?::new(self)
                }
            }

            macro_rules! [<$versioned_name _versionable_impl>] {
                ($inner_type:ty) => {
                    impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? [<$versioned_name Version>] for $inner_type
                    {
                        type Versioned = $versioned_name $(< $( $lt ),+ >)?;

                        fn into_versioned(self) -> Self::Versioned {
                            $versioned_name $(::< $( $lt ),+ >)?::new(self.into())
                        }
                    }
                };
            }

            $($([<$versioned_name _versionable_impl>]!($version_type);)*)?
            [<$versioned_name _versionable_impl>]!($latest_version_type);
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
        let mut versioned_actual = actual.into();
        versioned_actual.fully_update();
        let versioned_expected = VersionedExample::from(expected.clone());
        // Check fully_update (which returns a VersionedExample)
        assert_eq!(versioned_actual, versioned_expected,);
        // Check fully_update_into_latest_version (which returns an ExampleV4)
        assert_eq!(
            versioned_actual.fully_update_into_latest_version(),
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
            versioned.clone().fully_update_into_latest_version(),
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
            discriminator: 1,
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
