use serde::{Deserialize, Serialize};

use crate::abi::*;
use crate::constructs::*;
use crate::types::*;
use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgesRef {
    rid: RID,
}

impl From<RID> for BadgesRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl BadgesRef {
    pub fn amount(&self) -> U256 {
        let input = GetBadgesAmountInput { badges: self.rid };
        let output: GetBadgesAmountOutput = call_kernel!(GET_BADGES_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Resource {
        let input = GetBadgesResourceInput { badges: self.rid };
        let output: GetBadgesResourceOutput = call_kernel!(GET_BADGES_RESOURCE, input);

        output.resource.into()
    }

    pub fn destroy(self) {
        let input = ReturnBadgesInput {
            reference: self.rid,
        };
        let _: ReturnBadgesOutput = call_kernel!(RETURN_BADGES, input);
    }
}
