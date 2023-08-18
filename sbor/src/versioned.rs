pub enum UpdateResult<T> {
    Updated(T),
    AtLatest(T),
}

/// A marker trait to indicate that the type is versioned.
/// This can be used for type bounds for requiring that types are versioned.
pub trait HasLatestVersion {
    type Latest;
    fn into_latest(self) -> Self::Latest;
    fn as_latest_ref(&self) -> Option<&Self::Latest>;
}

pub trait CloneIntoLatest {
    type Latest;
    fn clone_into_latest(&self) -> Self::Latest;
}

impl<T: HasLatestVersion<Latest = Latest> + Clone, Latest> CloneIntoLatest for T {
    type Latest = Latest;

    fn clone_into_latest(&self) -> Self::Latest {
        self.clone().into_latest()
    }
}

/// This macro is intended for creating a data model which supports versioning.
/// This is useful for creating an SBOR data model which can be updated in future.
/// In future, enum variants can be added, and automatically mapped to.
///
/// This macro is just a simpler wrapper around the [`define_versioned`] macro,
/// for use when there's just a single version.
#[macro_export]
macro_rules! define_single_versioned {
    (
        $(#[$attributes:meta])*
        $vis:vis enum $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
        =>
        $latest_version_alias:ty = $latest_version_type:ty
    ) => {
        $crate::define_versioned!(
            $(#[$attributes])*
            $vis enum $name
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                previous_versions: [],
                latest_version: {
                    1 => $latest_version_alias = $latest_version_type
                },
            }
        );
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
#[macro_export]
macro_rules! define_versioned {
    (
        $(#[$attributes:meta])*
        $vis:vis enum $name:ident
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
        paste::paste! {
            // Create inline sub-macros to handle the type generics nested inside
            // iteration over previous_versions
            // See eg https://stackoverflow.com/a/73543948
            macro_rules! [<$name _trait_impl>] {
                (
                    $trait:ty,
                    $impl_block:tt
                ) => {
                    #[allow(dead_code)]
                    impl
                    $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
                    $trait
                    for $name $(< $( $lt ),+ >)?
                    $impl_block
                };
            }

            #[allow(dead_code)]
            $vis type $latest_version_alias = $latest_version_type;

            $(#[$attributes])*
            // We include the repr(u8) so that the SBOR discriminants are assigned
            // to match the version numbers if SBOR is used on the versioned enum
            #[repr(u8)]
            $vis enum $name $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                $($(
                    [<V $version_num>]($version_type) = $version_num,
                )*)?
                [<V $latest_version>]($latest_version_type) = $latest_version,
            }

            #[allow(dead_code)]
            impl
            $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $name
            $(< $( $lt ),+ >)?
            {
                pub fn new_latest(value: $latest_version_type) -> Self {
                    Self::[<V $latest_version>](value)
                }

                pub fn update_once(self) -> $crate::UpdateResult<Self> {
                    match self {
                    $($(
                        Self::[<V $version_num>](value) => $crate::UpdateResult::Updated(Self::[<V $update_to_version_num>](value.into())),
                    )*)?
                        Self::[<V $latest_version>](value) => $crate::UpdateResult::AtLatest(Self::[<V $latest_version>](value)),
                    }
                }

                pub fn update_to_latest(mut self) -> Self {
                    loop {
                        match self.update_once() {
                            $crate::UpdateResult::Updated(new) => {
                                self = new;
                            }
                            $crate::UpdateResult::AtLatest(latest) => {
                                return latest;
                            }
                        }
                    }
                }
            }

            [<$name _trait_impl>]!(
                $crate::HasLatestVersion,
                {
                    type Latest = $latest_version_type;

                    #[allow(irrefutable_let_patterns)]
                    fn into_latest(self) -> Self::Latest {
                        let Self::[<V $latest_version>](latest) = self.update_to_latest() else {
                            panic!("Invalid resolved latest version not equal to latest type")
                        };
                        return latest;
                    }

                    #[allow(unreachable_patterns)]
                    fn as_latest_ref(&self) -> Option<&Self::Latest> {
                        match self {
                            Self::[<V $latest_version>](latest) => Some(latest),
                            _ => None,
                        }
                    }
                }
            );

            $($([<$name _trait_impl>]!(
                From<$version_type>,
                {
                    fn from(value: $version_type) -> Self {
                        Self::[<V $version_num>](value)
                    }
                }
            );)*)?

            [<$name _trait_impl>]!(
                From<$latest_version_type>,
                {
                    fn from(value: $latest_version_type) -> Self {
                        Self::[<V $latest_version>](value)
                    }
                }
            );

            #[allow(dead_code)]
            $vis trait [<$name Version>] {
                // Note - we have an explicit Versioned associated type so that
                // different generic parameters can each create their own specific concrete type
                type Versioned;

                fn into_versioned(self) -> Self::Versioned;
            }

            macro_rules! [<$name _versionable_impl>] {
                ($inner_type:ty) => {
                    impl$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? [<$name Version>] for $inner_type
                    {
                        type Versioned = $name $(< $( $lt ),+ >)?;

                        fn into_versioned(self) -> Self::Versioned {
                            self.into()
                        }
                    }
                };
            }

            $($([<$name _versionable_impl>]!($version_type);)*)?
            [<$name _versionable_impl>]!($latest_version_type);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    crate::define_versioned!(
        #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
        enum VersionedExample {
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
        validate_latest(v4, expected_latest.clone());
    }

    fn validate_latest(
        actual: impl Into<VersionedExample>,
        expected: <VersionedExample as HasLatestVersion>::Latest,
    ) {
        let versioned_actual = actual.into();
        let versioned_expected = VersionedExample::from(expected.clone());
        // Check update_to_latest (which returns a VersionedExample)
        assert_eq!(
            versioned_actual.clone().update_to_latest(),
            versioned_expected,
        );
        // Check into_latest (which returns an ExampleV4)
        assert_eq!(versioned_actual.into_latest(), expected,);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct GenericModelV1<T>(T);

    define_single_versioned!(
        /// This is some rust doc as an example annotation
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum VersionedGenericModel<T> => GenericModel<T> = GenericModelV1<T>
    );

    #[test]
    pub fn generated_single_versioned_works() {
        let v1_model: GenericModel<_> = GenericModelV1(51u64);
        let versioned = VersionedGenericModel::from(v1_model.clone());
        let versioned_2 = v1_model.clone().into_versioned();
        assert_eq!(versioned.clone().into_latest(), v1_model.clone());
        assert_eq!(versioned, versioned_2);
    }
}
