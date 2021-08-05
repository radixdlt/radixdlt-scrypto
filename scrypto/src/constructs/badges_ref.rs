use sbor::{Decode, Encode};

use crate::constructs::*;
use crate::kernel::*;
use crate::types::*;

/// A borrowed reference to a `Badges` bucket.
#[derive(Debug, Encode, Decode)]
pub struct BadgesRef {
    bid: BID,
}

impl From<BID> for BadgesRef {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl BadgesRef {
    pub fn amount(&self) -> U256 {
        let input = GetBadgesAmountInput { badges: self.bid };
        let output: GetBadgesAmountOutput = call_kernel(GET_BADGES_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Resource {
        let input = GetBadgesResourceInput { badges: self.bid };
        let output: GetBadgesResourceOutput = call_kernel(GET_BADGES_RESOURCE, input);

        output.resource.into()
    }

    pub fn destroy(self) {
        let input = ReturnBadgesInput {
            reference: self.bid,
        };
        let _: ReturnBadgesOutput = call_kernel(RETURN_BADGES, input);
    }
}
