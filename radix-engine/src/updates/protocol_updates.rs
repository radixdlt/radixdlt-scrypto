use super::*;

macro_rules! derive_protocol_updates {
    (
        $(
            $variant_ident: ident
        ),* $(,)?
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ProtocolUpdate {
            $(
                $variant_ident
            ),*
        }

        impl ProtocolUpdate {
            pub const VARIANTS: [Self; count!( $($variant_ident),* )] = [
                $(
                    Self::$variant_ident
                ),*
            ];

            pub fn latest() -> Self {
                Self::VARIANTS[count!( $($variant_ident),* ) - 1]
            }
        }

        impl From<ProtocolUpdate> for ProtocolVersion {
            fn from(value: ProtocolUpdate) -> ProtocolVersion {
                match value {
                    $(
                        ProtocolUpdate::$variant_ident => ProtocolVersion::$variant_ident
                    ),*
                }
            }
        }
    };
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

derive_protocol_updates! {
    Anemone,
    Bottlenose,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProtocolVersion {
    Genesis,
    Anemone,
    Bottlenose,
}

impl ProtocolVersion {
    pub fn all_iterator() -> impl Iterator<Item = Self> {
        core::iter::once(Self::Genesis).chain(ProtocolUpdate::VARIANTS.map(From::from))
    }

    pub fn latest() -> Self {
        ProtocolUpdate::latest().into()
    }

    pub fn logical_name(&self) -> &'static str {
        match self {
            ProtocolVersion::Genesis => "babylon",
            ProtocolVersion::Anemone => "anemone",
            ProtocolVersion::Bottlenose => "bottlenose",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProtocolVersion::Genesis => "Babylon",
            ProtocolVersion::Anemone => "Anemone",
            ProtocolVersion::Bottlenose => "Bottlenose",
        }
    }
}
