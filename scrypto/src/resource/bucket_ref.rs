use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constructs::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::format;
use crate::types::*;

/// Represents a reference to a bucket.
#[derive(Debug, TypeId, Encode, Decode)]
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
impl Describe for BucketRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BUCKET_REF.to_owned(),
        }
    }
}

impl BucketRef {
    pub fn check(self, resource_def: Address) {
        if self.amount() > 0.into() && self.resource_def() == resource_def.into() {
            self.drop();
        } else {
            Logger::error(format!("Referenced bucket does not have {}", resource_def));
            panic!();
        }
    }

    pub fn amount(&self) -> Amount {
        let input = GetRefAmountInput {
            reference: self.rid,
        };
        let output: GetRefAmountOutput = call_kernel(GET_REF_AMOUNT, input);

        output.amount
    }

    pub fn resource_def(&self) -> ResourceDef {
        let input = GetRefResourceAddressInput {
            reference: self.rid,
        };
        let output: GetRefResourceAddressOutput = call_kernel(GET_REF_RESOURCE_DEF, input);

        output.resource_def.into()
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
