use crate::data::manifest::ManifestCustomValueKind;
use crate::types::EntityType;
use crate::types::NodeId;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::fmt;
use sbor::*;

/// Any address supported by manifest, both global and local.
///
/// Must start with a supported entity type byte.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestAddress {
    /// Static address, either global or internal, with entity type byte checked.
    /// TODO: prevent direct construction, as in `NonFungibleLocalId`
    Static(NodeId),
    /// Named address, global only at the moment.
    Named(u32),
}

//========
// binary
//========

impl Categorize<ManifestCustomValueKind> for ManifestAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for ManifestAddress {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::Static(node_id) => {
                encoder.write_discriminator(0)?;
                encoder.write_slice(node_id.as_bytes())?;
            }
            Self::Named(address_id) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(&address_id.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for ManifestAddress {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            0 => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                if EntityType::from_repr(slice[0]).is_none() {
                    return Err(DecodeError::InvalidCustomValue);
                }
                Ok(Self::Static(NodeId(slice.try_into().unwrap())))
            }
            1 => {
                let slice = decoder.read_slice(4)?;
                let id = u32::from_le_bytes(slice.try_into().unwrap());
                Ok(Self::Named(id))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

//======
// text
//======

impl fmt::Debug for ManifestAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ManifestAddress::Static(node_id) => {
                write!(f, "Address({})", hex::encode(node_id.as_bytes()))
            }
            ManifestAddress::Named(name) => write!(f, "NamedAddress({})", name),
        }
    }
}
