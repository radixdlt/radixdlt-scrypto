use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// Represents a reference to a bucket.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketRef {
    rid: RID,
}

impl From<RID> for BucketRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl From<BucketRef> for RID {
    fn from(a: BucketRef) -> RID {
        a.rid
    }
}
impl Describe for BucketRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BUCKET_REF.to_owned(),
        }
    }
}

impl BucketRef {
    pub fn amount(&self) -> Amount {
        let input = GetRefAmountInput {
            reference: self.rid,
        };
        let output: GetRefAmountOutput = call_kernel(GET_REF_AMOUNT, input);

        output.amount
    }

    pub fn resource_address(&self) -> Address {
        let input = GetRefResourceAddressInput {
            reference: self.rid,
        };
        let output: GetRefResourceAddressOutput = call_kernel(GET_REF_RESOURCE_ADDRESS, input);

        output.resource_address
    }

    pub fn drop(self) {
        let input = DropReferenceInput {
            reference: self.rid,
        };
        let _: DropReferenceOutput = call_kernel(DROP_REFERENCE, input);
    }

    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }
}
