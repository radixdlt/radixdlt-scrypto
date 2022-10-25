use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::core::*;
use crate::misc::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reference(pub RENodeId);

impl Reference {}

//========
// binary
//========

impl TryFrom<&[u8]> for Reference {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Reference => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ReferenceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::reference(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

scrypto_type!(ReferenceAddress, ScryptoType::ReferenceAddress, Vec::new());

//======
// text
//======

impl fmt::Debug for ReferenceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ReferenceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_reference_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ReferenceAddress::Normal(_) => {
                write!(f, "NormalReference[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
