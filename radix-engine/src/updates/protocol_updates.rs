use crate::internal_prelude::*;

define_single_versioned! {
    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    pub ProtocolUpdateStatusSummarySubstate(ProtocolUpdateStatusSummaryVersions) => ProtocolUpdateStatusSummary = ProtocolUpdateStatusSummaryV1,
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:protocol_update_status_substate_cuttlefish_schema.bin",
        ))]
    ]
}

impl ProtocolUpdateStatusSummarySubstate {
    pub fn load(database: &impl SubstateDatabase) -> Self {
        let substate = database.get_substate(
            TRANSACTION_TRACKER,
            PROTOCOL_UPDATE_STATUS_PARTITION,
            ProtocolUpdateStatusField::Summary,
        );
        if let Some(value) = substate {
            return value;
        }
        // We are pre-cuttlefish. Need to distinguish between different versions.
        let protocol_version = if database
            .get_raw_substate(
                TRANSACTION_TRACKER,
                BOOT_LOADER_PARTITION,
                BootLoaderField::SystemBoot,
            )
            .is_some()
        {
            ProtocolVersion::Bottlenose
        } else if database
            .get_raw_substate(
                TRANSACTION_TRACKER,
                BOOT_LOADER_PARTITION,
                BootLoaderField::VmBoot,
            )
            .is_some()
        {
            ProtocolVersion::Anemone
        } else if database
            .get_raw_substate(
                TRANSACTION_TRACKER,
                TYPE_INFO_FIELD_PARTITION,
                TypeInfoField::TypeInfo,
            )
            .is_some()
        {
            ProtocolVersion::Babylon
        } else {
            ProtocolVersion::Unbootstrapped
        };

        ProtocolUpdateStatusSummaryV1 {
            protocol_version,
            update_status: ProtocolUpdateStatus::Complete,
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct ProtocolUpdateStatusSummaryV1 {
    pub protocol_version: ProtocolVersion,
    pub update_status: ProtocolUpdateStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ProtocolUpdateStatus {
    Complete,
    InProgress {
        latest_commit: LatestProtocolUpdateCommitBatch,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct LatestProtocolUpdateCommitBatch {
    pub batch_group_index: usize,
    pub batch_group_name: String,
    pub batch_index: usize,
    pub batch_name: String,
}

macro_rules! count {
    (
        $ident: ident, $($other_idents: ident),* $(,)?
    ) => {
        1 + count!( $($other_idents),* )
    };
    (
        $ident: ident $(,)?
    ) => {
        1
    }
}

macro_rules! latest {
    (
        $enum_ident: ident, $ident: ident, $($other_idents: ident),* $(,)?
    ) => {
        latest!( $enum_ident, $($other_idents),* )
    };
    (
        $enum_ident: ident, $ident: ident $(,)?
    ) => {
        $enum_ident :: $ident
    }
}

macro_rules! define_enum {
    (
        $ident:ident,
        $(
            (
                $variant_name: ident,
                $logical_name: expr,
                $display_name: expr
            )
        ),* $(,)?
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Sbor)]
        pub enum $ident {
            $($variant_name),*
        }

        impl $ident {
            const VARIANTS: [Self; count!( $($variant_name),* )] = [
                $(
                    Self::$variant_name
                ),*
            ];

            pub const LATEST: $ident = latest!( $ident, $($variant_name),* );

            pub const fn logical_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant_name => $logical_name
                    ),*
                }
            }

            pub const fn display_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant_name => $display_name
                    ),*
                }
            }

            pub fn try_from_logical_name(logical_name: &str) -> Option<Self> {
                match logical_name {
                    $(
                        $logical_name => Some(Self::$variant_name)
                    ),*,
                    _ => None
                }
            }

            pub fn try_from_display_name(display_name: &str) -> Option<Self> {
                match display_name {
                    $(
                        $display_name => Some(Self::$variant_name)
                    ),*,
                    _ => None
                }
            }
        }
    };
}

macro_rules! define_protocol_version_and_updates {
    (
        pregenesis: {
            variant_name: $pregenesis_variant_name: ident,
            logical_name: $pregenesis_logical_name: expr,
            display_name: $pregenesis_display_name: expr $(,)?
        },
        genesis: {
            variant_name: $genesis_variant_name: ident,
            logical_name: $genesis_logical_name: expr,
            display_name: $genesis_display_name: expr $(,)?
        },
        protocol_updates: [
            $(
                {
                    variant_name: $protocol_update_variant_name: ident,
                    logical_name: $protocol_update_logical_name: expr,
                    display_name: $protocol_update_display_name: expr $(,)?
                }
            ),* $(,)?
        ]
    ) => {
        define_enum!(
            ProtocolVersion,
            ($pregenesis_variant_name, $pregenesis_logical_name, $pregenesis_display_name),
            ($genesis_variant_name, $genesis_logical_name, $genesis_display_name),
            $(($protocol_update_variant_name, $protocol_update_logical_name, $protocol_update_display_name)),*
        );

        impl ProtocolVersion {
            pub const PRE_GENESIS: Self = Self::$pregenesis_variant_name;
            pub const GENESIS: Self = Self::$genesis_variant_name;
        }
    };
}

impl ProtocolVersion {
    /// This points to `CuttlefishPart2`, for symmetry with updates which didn't need to be
    /// in two parts.
    #[allow(non_upper_case_globals)]
    pub const Cuttlefish: Self = Self::CuttlefishPart2;
}

// This macro defines the protocol version and the protocol updates enums and all of the methods
// needed on them.
//
// The order in which the protocol updates is defined is very important since many places in our
// codebase relies on it such as applying the protocol updates in order. If the order is changed
// then the protocol updates will be applied in a different order. So, only thing we can do to
// is append to this list, never change.
define_protocol_version_and_updates! {
    pregenesis: {
        variant_name: Unbootstrapped,
        logical_name: "unbootstrapped",
        display_name: "Unbootstrapped",
    },
    genesis: {
        variant_name: Babylon,
        logical_name: "babylon",
        display_name: "Babylon",
    },
    protocol_updates: [
        {
            variant_name: Anemone,
            logical_name: "anemone",
            display_name: "Anemone",
        },
        {
            variant_name: Bottlenose,
            logical_name: "bottlenose",
            display_name: "Bottlenose",
        },
        {
            variant_name: CuttlefishPart1,
            logical_name: "cuttlefish",
            display_name: "Cuttlefish (Part 1)",
        },
        {
            variant_name: CuttlefishPart2,
            logical_name: "cuttlefish-part2",
            display_name: "Cuttlefish (Part 2)",
        }
    ]
}

impl ProtocolVersion {
    pub fn all_from(
        from_version_inclusive: ProtocolVersion,
    ) -> impl Iterator<Item = ProtocolVersion> {
        Self::VARIANTS
            .into_iter()
            .skip_while(move |v| *v < from_version_inclusive)
    }

    pub fn all_between_inclusive(
        from_version_inclusive: ProtocolVersion,
        to_version_inclusive: ProtocolVersion,
    ) -> impl Iterator<Item = ProtocolVersion> {
        Self::VARIANTS
            .into_iter()
            .skip_while(move |v| *v < from_version_inclusive)
            .take_while(move |v| *v <= to_version_inclusive)
    }

    pub fn all_between(
        from_version_inclusive: ProtocolVersion,
        to_version_exclusive: ProtocolVersion,
    ) -> impl Iterator<Item = ProtocolVersion> {
        Self::VARIANTS
            .into_iter()
            .skip_while(move |v| *v < from_version_inclusive)
            .take_while(move |v| *v < to_version_exclusive)
    }

    pub fn next(&self) -> Option<Self> {
        Self::VARIANTS
            .iter()
            .skip_while(|v| v <= &self)
            .next()
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_latest_protocol_version_is_as_expected() {
        assert_eq!(ProtocolVersion::LATEST, ProtocolVersion::CuttlefishPart2);
    }

    #[test]
    fn test_next() {
        assert_eq!(
            ProtocolVersion::PRE_GENESIS.next(),
            Some(ProtocolVersion::GENESIS)
        );
        assert_eq!(
            ProtocolVersion::GENESIS.next(),
            Some(ProtocolVersion::Anemone)
        );
        assert_eq!(
            ProtocolVersion::Anemone.next(),
            Some(ProtocolVersion::Bottlenose)
        );
        assert_eq!(ProtocolVersion::LATEST.next(), None);
    }

    #[test]
    fn assert_protocol_versions_have_the_expected_order() {
        let variants =
            ProtocolVersion::all_from(ProtocolVersion::Unbootstrapped).collect::<Vec<_>>();

        assert_eq!(
            variants,
            vec![
                ProtocolVersion::Unbootstrapped,
                ProtocolVersion::Babylon,
                ProtocolVersion::Anemone,
                ProtocolVersion::Bottlenose,
                ProtocolVersion::CuttlefishPart1,
                ProtocolVersion::CuttlefishPart2,
            ],
        );
        assert!(variants.windows(2).all(|item| item[0] < item[1]))
    }

    #[test]
    fn assert_protocol_version_range_queries_work() {
        assert_eq!(
            ProtocolVersion::all_between(ProtocolVersion::Babylon, ProtocolVersion::Bottlenose,)
                .collect::<Vec<_>>(),
            vec![ProtocolVersion::Babylon, ProtocolVersion::Anemone,],
        );
        assert_eq!(
            ProtocolVersion::all_between_inclusive(
                ProtocolVersion::Babylon,
                ProtocolVersion::Bottlenose,
            )
            .collect::<Vec<_>>(),
            vec![
                ProtocolVersion::Babylon,
                ProtocolVersion::Anemone,
                ProtocolVersion::Bottlenose,
            ],
        );
    }
}
