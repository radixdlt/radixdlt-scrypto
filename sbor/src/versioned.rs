pub enum UpdateResult<T> {
    Updated(T),
    AtLatest(T),
}

/// A marker trait to indicate that the type is versioned.
/// This can be used for type bounds for requiring that types are versioned.
pub trait HasLatestVersion {
    type Latest;
    fn into_latest(self) -> Self::Latest;
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
        $vis:vis enum $name:ident {
            previous_versions: [
                $($version_num:expr => $version_ident:ident -> { updates_to: $update_to_version_num:expr }),*
                $(,)? // Optional trailing comma
            ],
            latest_version: {
                $latest_version:expr => $latest_version_ident:ident,
            }$(,)?
        }
    ) => {
        paste::paste! {
            $(#[$attributes])*
            // We include the repr(u8) so that the SBOR discriminants are assigned
            // to match the version numbers if SBOR is used on the versioned enum
            #[repr(u8)]
            $vis enum $name {
                $(
                    [<V $version_num>]($version_ident) = $version_num,
                )*
                [<V $latest_version>]($latest_version_ident) = $latest_version,
            }

            impl $name {
                pub fn update_once(self) -> UpdateResult<Self> {
                    match self {
                    $(
                        Self::[<V $version_num>](value) => crate::UpdateResult::Updated(Self::[<V $update_to_version_num>](value.into())),
                    )*
                        Self::[<V $latest_version>](value) => crate::UpdateResult::AtLatest(Self::[<V $latest_version>](value)),
                    }
                }

                pub fn update_to_latest(mut self) -> Self {
                    loop {
                        match self.update_once() {
                            UpdateResult::Updated(new) => {
                                self = new;
                            }
                            UpdateResult::AtLatest(latest) => {
                                return latest;
                            }
                        }
                    }
                }
            }

            impl crate::HasLatestVersion for $name {
                type Latest = $latest_version_ident;

                fn into_latest(self) -> Self::Latest {
                    let Self::[<V $latest_version>](latest) = self.update_to_latest() else {
                        panic!("Invalid resolved latest version not equal to latest type")
                    };
                    return latest;
                }
            }

            $(
                impl From<$version_ident> for $name {
                    fn from(value: $version_ident) -> Self {
                        Self::[<V $version_num>](value)
                    }
                }
            )*

            impl From<$latest_version_ident> for $name {
                fn from(value: $latest_version_ident) -> Self {
                    Self::[<V $latest_version>](value)
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
        enum VersionedExample {
            previous_versions: [
                1 => ExampleV1 -> { updates_to: 2 },
                2 => ExampleV2 -> { updates_to: 4 },
                3 => ExampleV3 -> { updates_to: 4 },
            ],
            latest_version: {
                4 => ExampleV4,
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
            Self {
                the_value: value,
            }
        }
    }

    // And explicit updates between them, which are needed
    // for the versioned type
    impl From<ExampleV2> for ExampleV4 {
        fn from(value: ExampleV2) -> Self {
            Self {
                the_value: value,
            }
        }
    }

    impl From<ExampleV3> for ExampleV4 {
        fn from(value: ExampleV3) -> Self {
            Self {
                the_value: value.0,
            }
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

    fn validate_latest(actual: impl Into<VersionedExample>, expected: <VersionedExample as HasLatestVersion>::Latest) {
        let versioned_actual = actual.into();
        let versioned_expected = VersionedExample::from(expected.clone());
        // Check update_to_latest (which returns a VersionedExample)
        assert_eq!(
            versioned_actual.clone().update_to_latest(),
            versioned_expected,
        );
        // Check into_latest (which returns an ExampleV4)
        assert_eq!(
            versioned_actual.into_latest(),
            expected,
        );
    }
}