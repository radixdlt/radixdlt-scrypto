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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub enum ManifestAddress {
    /// Static address, either global or internal, with entity type byte checked.
    /// TODO: prevent direct construction, as in `NonFungibleLocalId`
    Static(NodeId),
    /// Named address, global only at the moment.
    Named(u32),
}

pub const MANIFEST_ADDRESS_DISCRIMINATOR_STATIC: u8 = 0u8;
pub const MANIFEST_ADDRESS_DISCRIMINATOR_NAMED: u8 = 1u8;

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
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_STATIC)?;
                encoder.write_slice(node_id.as_bytes())?;
            }
            Self::Named(address_id) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_NAMED)?;
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
            MANIFEST_ADDRESS_DISCRIMINATOR_STATIC => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                if EntityType::from_repr(slice[0]).is_none() {
                    return Err(DecodeError::InvalidCustomValue);
                }
                Ok(Self::Static(NodeId(slice.try_into().unwrap())))
            }
            MANIFEST_ADDRESS_DISCRIMINATOR_NAMED => {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use sbor::prelude::Vec;

    fn prepare(addr_input: &ManifestAddress) -> Result<ManifestAddress, sbor::DecodeError> {
        #[cfg(not(feature = "alloc"))]
        println!("Encoding manifest address: {:?}", addr_input);
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        assert!(addr_input.encode_body(&mut encoder).is_ok());
        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        decoder.decode_deeper_body_with_value_kind::<ManifestAddress>(ManifestAddress::value_kind())
    }

    #[test]
    fn manifest_address_decode_static_success() {
        let node_id = NodeId::new(EntityType::GlobalPackage as u8, &[0; NodeId::RID_LENGTH]);
        let addr_input = ManifestAddress::Static(node_id);
        let addr_output = prepare(&addr_input);
        assert!(addr_output.is_ok());
        assert_eq!(addr_input, addr_output.unwrap());
    }

    #[test]
    fn manifest_address_decode_named_success() {
        let addr_input = ManifestAddress::Named(1);
        let addr_output = prepare(&addr_input);
        assert!(addr_output.is_ok());
        assert_eq!(addr_input, addr_output.unwrap());
    }

    #[test]
    fn manifest_address_decode_static_fail() {
        // use invalid entity type (0) to an generate error
        let node_id = NodeId::new(0, &[0; NodeId::RID_LENGTH]);
        let addr_input = ManifestAddress::Static(node_id);
        let addr_output = prepare(&addr_input);
        assert!(matches!(addr_output, Err(DecodeError::InvalidCustomValue)));
    }

    #[test]
    fn manifest_address_decode_named_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        encoder
            .write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_NAMED)
            .unwrap();
        let malformed_value: u8 = 1; // use u8 instead of u32 should inovke an error
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder
            .decode_deeper_body_with_value_kind::<ManifestAddress>(ManifestAddress::value_kind());

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert!(matches!(
            addr_output,
            Err(DecodeError::BufferUnderflow { .. })
        ));
    }

    #[test]
    fn manifest_address_decode_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder
            .decode_deeper_body_with_value_kind::<ManifestAddress>(ManifestAddress::value_kind());

        assert!(matches!(addr_output, Err(DecodeError::InvalidCustomValue)));
    }
}
