use sbor::{Decode, Encode};

use crate::constructs::*;
use crate::kernel::*;
use crate::types::*;

/// A bucket that holds badge resource.
#[derive(Debug, Encode, Decode)]
pub struct Badges {
    bid: BID,
}

impl From<BID> for Badges {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl Into<BID> for Badges {
    fn into(self) -> BID {
        self.bid
    }
}

impl Badges {
    pub fn new(amount: U256, resource: Address) -> Self {
        assert!(amount >= U256::one());

        let input = MintBadgesInput { amount, resource };
        let output: MintBadgesOutput = call_kernel(MINT_BADGES, input);

        output.badges.into()
    }

    pub fn put(&mut self, other: Self) {
        let input = CombineBadgesInput {
            badges: self.bid,
            other: other.bid,
        };
        let _: CombineBadgesOutput = call_kernel(COMBINE_BADGES, input);
    }

    pub fn take(&mut self, amount: U256) -> Self {
        let input = SplitBadgesInput {
            badges: self.bid,
            amount,
        };
        let output: SplitBadgesOutput = call_kernel(SPLIT_BADGES, input);

        output.badges.into()
    }

    pub fn borrow(&self) -> BadgesRef {
        let input = BorrowBadgesInput { badges: self.bid };
        let output: BorrowBadgesOutput = call_kernel(BORROW_BADGES, input);

        output.reference.into()
    }

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

    pub fn address(&self) -> Address {
        self.resource().address()
    }
}
