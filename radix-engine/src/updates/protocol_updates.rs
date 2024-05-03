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

macro_rules! earliest {
    (
        $enum_ident: ident, $ident: ident, $($other_idents: ident),* $(,)?
    ) => {
        $enum_ident::$ident
    };
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
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum $ident {
            $($variant_name),*
        }

        impl $ident {
            pub const VARIANTS: [Self; count!( $($variant_name),* )] = [
                $(
                    Self::$variant_name
                ),*
            ];

            pub const EARLIEST: $ident = earliest!( $ident, $($variant_name),* );
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
        genesis: {
            variant_name: $variant_name: ident,
            logical_name: $logical_name: expr,
            display_name: $display_name: expr $(,)?
        },
        protocol_updates: [
            $(
                {
                    variant_name: $protocol_update_variant_name: ident,
                    logical_name: $protocol_update_logical_name: expr,
                    display_name: $protocol_update_display_name: expr $(,)?
                }
            ),* $(,)*
        ]
    ) => {
        define_enum!(
            ProtocolVersion,
            ($variant_name, $logical_name, $display_name)
            $(, ($protocol_update_variant_name, $protocol_update_logical_name, $protocol_update_display_name))*
        );
        define_enum!(
            ProtocolUpdate,
            $(($protocol_update_variant_name, $protocol_update_logical_name, $protocol_update_display_name)),*
        );

        impl From<ProtocolUpdate> for ProtocolVersion {
            fn from(value: ProtocolUpdate) -> ProtocolVersion {
                match value {
                    $(
                        ProtocolUpdate::$protocol_update_variant_name
                            => ProtocolVersion::$protocol_update_variant_name
                    ),*
                }
            }
        }
    };
}

// This macro defines the protocol version and the protocol updates enums and all of the methods
// needed on them.
//
// The order in which the protocol updates is defined is very important since many places in our
// codebase relies on it such as applying the protocol updates in order. If the order is changed
// then the protocol updates will be applied in a different order. So, only thing we can do to
// is append to this list, never change.
define_protocol_version_and_updates! {
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
        }
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_earliest_protocol_update_is_as_expected() {
        assert_eq!(ProtocolUpdate::EARLIEST, ProtocolUpdate::Anemone);
    }

    #[test]
    fn assert_earliest_protocol_version_is_as_expected() {
        assert_eq!(ProtocolVersion::EARLIEST, ProtocolVersion::Babylon);
    }

    #[test]
    fn assert_latest_protocol_update_is_as_expected() {
        assert_eq!(ProtocolUpdate::LATEST, ProtocolUpdate::Bottlenose);
    }

    #[test]
    fn assert_latest_protocol_version_is_as_expected() {
        assert_eq!(ProtocolVersion::LATEST, ProtocolVersion::Bottlenose);
    }

    #[test]
    fn assert_protocol_versions_have_the_expected_order() {
        let variants = ProtocolVersion::VARIANTS;

        assert_eq!(
            variants,
            [
                ProtocolVersion::Babylon,
                ProtocolVersion::Anemone,
                ProtocolVersion::Bottlenose
            ]
        );
        assert!(variants.windows(2).all(|item| item[0] < item[1]))
    }

    #[test]
    fn assert_protocol_updates_have_the_expected_order() {
        let variants = ProtocolUpdate::VARIANTS;

        assert_eq!(
            variants,
            [ProtocolUpdate::Anemone, ProtocolUpdate::Bottlenose]
        );
        assert!(variants.windows(2).all(|item| item[0] < item[1]))
    }
}
