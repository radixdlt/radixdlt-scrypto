use crate::address::AddressBech32Decoder;
use crate::address::{AddressBech32EncodeError, AddressDisplayContext, NO_NETWORK};
use crate::crypto::{hash, PublicKey};
use crate::data::manifest::model::ManifestAddress;
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::Reference;
use crate::data::scrypto::*;
use crate::types::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use sbor::rust::prelude::*;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

/// Address to a global component
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentAddress(NodeId); // private to ensure entity type check

impl ComponentAddress {
    pub const fn new_or_panic(raw: [u8; NodeId::LENGTH]) -> Self {
        let node_id = NodeId(raw);
        assert!(node_id.is_global_component());
        Self(node_id)
    }

    pub unsafe fn new_unchecked(raw: [u8; NodeId::LENGTH]) -> Self {
        Self(NodeId(raw))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn virtual_account_from_public_key<P: Into<PublicKey> + Clone>(
        public_key: &P,
    ) -> ComponentAddress {
        match public_key.clone().into() {
            PublicKey::Secp256k1(public_key) => {
                let mut node_id: [u8; NodeId::LENGTH] = hash(public_key.to_vec()).lower_bytes();
                node_id[0] = EntityType::GlobalVirtualSecp256k1Account as u8;
                Self(NodeId(node_id))
            }
            PublicKey::Ed25519(public_key) => {
                let mut node_id: [u8; NodeId::LENGTH] = hash(public_key.to_vec()).lower_bytes();
                node_id[0] = EntityType::GlobalVirtualEd25519Account as u8;
                Self(NodeId(node_id))
            }
        }
    }

    pub fn virtual_identity_from_public_key<P: Into<PublicKey> + Clone>(
        public_key: &P,
    ) -> ComponentAddress {
        match public_key.clone().into() {
            PublicKey::Secp256k1(public_key) => {
                let mut node_id: [u8; NodeId::LENGTH] = hash(public_key.to_vec()).lower_bytes();
                node_id[0] = EntityType::GlobalVirtualSecp256k1Identity as u8;
                Self(NodeId(node_id))
            }
            PublicKey::Ed25519(public_key) => {
                let mut node_id: [u8; NodeId::LENGTH] = hash(public_key.to_vec()).lower_bytes();
                node_id[0] = EntityType::GlobalVirtualEd25519Identity as u8;
                Self(NodeId(node_id))
            }
        }
    }

    pub fn as_node_id(&self) -> &NodeId {
        &self.0
    }

    pub const fn into_node_id(self) -> NodeId {
        self.0
    }

    pub fn try_from_hex(s: &str) -> Option<Self> {
        hex::decode(s)
            .ok()
            .and_then(|x| Self::try_from(x.as_ref()).ok())
    }

    pub fn try_from_bech32(decoder: &AddressBech32Decoder, s: &str) -> Option<Self> {
        if let Ok((_, full_data)) = decoder.validate_and_decode(s) {
            Self::try_from(full_data.as_ref()).ok()
        } else {
            None
        }
    }

    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
}

impl AsRef<[u8]> for ComponentAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        Self::try_from(NodeId(value))
    }
}

impl TryFrom<NodeId> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(node_id: NodeId) -> Result<Self, Self::Error> {
        if node_id.is_global_component() {
            Ok(Self(node_id))
        } else {
            Err(ParseComponentAddressError::InvalidEntityTypeId(
                node_id.0[0],
            ))
        }
    }
}

impl TryFrom<&[u8]> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => ComponentAddress::try_from(copy_u8_array(slice)),
            _ => Err(ParseComponentAddressError::InvalidLength(slice.len())),
        }
    }
}

impl TryFrom<GlobalAddress> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(address: GlobalAddress) -> Result<Self, Self::Error> {
        ComponentAddress::try_from(Into::<[u8; NodeId::LENGTH]>::into(address))
    }
}

impl Into<[u8; NodeId::LENGTH]> for ComponentAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl From<ComponentAddress> for super::GlobalAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::new_or_panic(value.into())
    }
}

impl From<ComponentAddress> for Reference {
    fn from(value: ComponentAddress) -> Self {
        Self(value.into())
    }
}

impl From<ComponentAddress> for ManifestAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Static(value.into())
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseComponentAddressError {
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseComponentAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseComponentAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    ComponentAddress,
    ScryptoCustomValueKind::Reference,
    Type::ComponentAddress,
    NodeId::LENGTH,
    COMPONENT_ADDRESS_TYPE,
    component_address_type_data
);

impl Categorize<ManifestCustomValueKind> for ComponentAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for ComponentAddress {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_discriminator(0)?;
        encoder.write_slice(self.as_ref())?;
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for ComponentAddress {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            0 => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

//======
// text
//======

impl fmt::Debug for ComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ComponentAddress {
    type Error = AddressBech32EncodeError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_to_fmt(f, self.0.as_ref());
        }

        // This could be made more performant by streaming the hex into the formatter
        write!(f, "ComponentAddress({})", hex::encode(&self.0))
            .map_err(AddressBech32EncodeError::FormatError)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::Ed25519PublicKey;

    #[test]
    fn component_address_unchecked() {
        unsafe {
            ComponentAddress::new_unchecked([0; NodeId::LENGTH]);
        }

        let public_key = Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]);
        let addr = ComponentAddress::virtual_identity_from_public_key(&PublicKey::Ed25519(public_key));

        ComponentAddress::try_from_hex(&addr.to_hex()).unwrap();

        // pass empty string to generate an error
        assert!(ComponentAddress::try_from_bech32(&AddressBech32Decoder::for_simulator(), "").is_none());

        // pass wrong lenght array to generate an error
        let v = Vec::from([0u8; NodeId::LENGTH + 1]);
        assert!(ComponentAddress::try_from(v.as_slice()).is_err());

        Reference::try_from(addr).unwrap();
        let _ = ManifestAddress::try_from(addr).unwrap();

        let v = Vec::from([0u8; NodeId::LENGTH]);
        let addr = ComponentAddress::try_from(v.as_slice());
        assert!(addr.is_err());
        println!("Decode error: {}", addr.unwrap_err());
    }

    #[test]
    fn component_address_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u32 = 1; // use u32 instead of u8 should inovke an error
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let proof_output = decoder
            .decode_deeper_body_with_value_kind::<ComponentAddress>(ComponentAddress::value_kind());

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert!(matches!(
            proof_output,
            Err(DecodeError::InvalidCustomValue)
        ));
    }

    #[test]
    fn component_address_decode_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder
            .decode_deeper_body_with_value_kind::<ComponentAddress>(ComponentAddress::value_kind());

        assert!(matches!(addr_output, Err(DecodeError::InvalidCustomValue)));
    }

    #[test]
    fn component_address_decode_success() {
        let public_key = Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]);
        let addr_input = ComponentAddress::virtual_identity_from_public_key(&PublicKey::Ed25519(public_key));

        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        assert!(addr_input.encode_body(&mut encoder).is_ok());
        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder.decode_deeper_body_with_value_kind::<ComponentAddress>(ComponentAddress::value_kind());

        assert!(addr_output.is_ok());
        assert_eq!(addr_input, addr_output.unwrap());
    }
}
