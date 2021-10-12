use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::format;
use crate::rust::vec;
use crate::types::*;

/// Represents a reference to a bucket.
#[derive(Debug)]
pub struct BucketRef {
    rid: Rid,
}

impl From<Rid> for BucketRef {
    fn from(rid: Rid) -> Self {
        Self { rid }
    }
}

impl From<BucketRef> for Rid {
    fn from(a: BucketRef) -> Rid {
        a.rid
    }
}

impl BucketRef {
    pub fn check<A: Into<ResourceDef>>(self, resource_def: A) {
        let resource_def: ResourceDef = resource_def.into();
        if self.amount() > 0.into() && self.resource_def() == resource_def {
            self.drop();
        } else {
            Logger::error(format!(
                "Referenced bucket does not have {}",
                resource_def.address()
            ));
            panic!();
        }
    }

    pub fn amount(&self) -> Amount {
        let input = GetBucketRefAmountInput {
            bucket_ref: self.rid,
        };
        let output: GetBucketRefAmountOutput = call_kernel(GET_BUCKET_REF_AMOUNT, input);

        output.amount
    }

    pub fn resource_def(&self) -> ResourceDef {
        let input = GetBucketRefResourceDefInput {
            bucket_ref: self.rid,
        };
        let output: GetBucketRefResourceDefOutput = call_kernel(GET_BUCKET_REF_RESOURCE_DEF, input);

        output.resource_def.into()
    }

    pub fn drop(self) {
        let input = DropBucketRefInput {
            bucket_ref: self.rid,
        };
        let _: DropBucketRefOutput = call_kernel(DROP_BUCKET_REF, input);
    }

    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }
}

//========
// SBOR
//========

impl TypeId for BucketRef {
    fn type_id() -> u8 {
        Rid::type_id()
    }
}

impl Encode for BucketRef {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.rid.encode_value(encoder);
    }
}

impl Decode for BucketRef {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Rid::decode_value(decoder).map(Into::into)
    }
}

impl Describe for BucketRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BUCKET_REF.to_owned(),
            generics: vec![],
        }
    }
}
