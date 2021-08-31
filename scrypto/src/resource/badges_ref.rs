use sbor::{model::Type, *};

use crate::constants::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A reference to a `Badges` bucket.
#[derive(Debug)]
pub struct BadgesRef {
    rid: RID,
}

impl Encode for BadgesRef {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.rid.encode_value(encoder);
    }

    #[inline]
    fn sbor_type() -> u8 {
        SCRYPTO_TYPE_BADGES_REF
    }
}

impl Decode for BadgesRef {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let rid = RID::decode_value(decoder)?;
        Ok(rid.into())
    }

    #[inline]
    fn sbor_type() -> u8 {
        SCRYPTO_TYPE_BADGES_REF
    }
}

impl Describe for BadgesRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BADGES_REF.to_owned(),
        }
    }
}

impl From<RID> for BadgesRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl Into<RID> for BadgesRef {
    fn into(self) -> RID {
        self.rid
    }
}

impl BadgesRef {
    pub fn check(&self, resource: Address) {
        assert!(self.resource() == resource && self.amount() >= 1.into());
    }

    pub fn amount(&self) -> U256 {
        self.rid.amount()
    }

    pub fn resource(&self) -> Address {
        self.rid.resource()
    }

    pub fn destroy(self) {
        self.rid.destroy()
    }
}
