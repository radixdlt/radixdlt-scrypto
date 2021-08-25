use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A borrowed rid to a `Badges` bucket.
#[derive(Debug, Encode, Decode)]
pub struct BadgesRef {
    rid: RID,
}

impl Describe for BadgesRef {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::resource::BadgesRef".to_owned(),
        }
    }
}

impl From<RID> for BadgesRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl BadgesRef {
    pub fn amount(&self) -> U256 {
        let input = GetAmountRefInput {
            reference: self.rid,
        };
        let output: GetAmountRefOutput = call_kernel(GET_AMOUNT_REF, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetResourceRefInput {
            reference: self.rid,
        };
        let output: GetResourceRefOutput = call_kernel(GET_RESOURCE_REF, input);

        output.resource
    }

    pub fn destroy(self) {
        let input = DropReferenceInput {
            reference: self.rid,
        };
        let _: DropReferenceOutput = call_kernel(DROP_REFERENCE, input);
    }
}
