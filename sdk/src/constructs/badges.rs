use serde::{Deserialize, Serialize};

use crate::abi::*;
use crate::constructs::*;
use crate::types::*;
use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Badges {
    rid: RID,
}

impl From<RID> for Badges {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl Into<RID> for Badges {
    fn into(self) -> RID {
        self.rid.clone()
    }
}

impl Badges {
    pub fn new(amount: U256, resource: &Resource) -> Self {
        assert!(amount >= U256::one());

        let input = MintBadgesInput {
            amount,
            resource: resource.address(),
        };
        let output: MintBadgesOutput = call_kernel!(MINT_BADGES, input);

        output.badges.into()
    }

    pub fn put(&mut self, other: Self) {
        let input = CombineBadgesInput {
            badges: self.rid.clone(),
            other: other.rid.clone(),
        };
        let _: CombineBadgesOutput = call_kernel!(COMBINE_BADGES, input);
    }

    pub fn take(&mut self, amount: U256) -> Self {
        let input = SplitBadgesInput {
            badges: self.rid.clone(),
            amount,
        };
        let output: SplitBadgesOutput = call_kernel!(SPLIT_BADGES, input);

        output.badges.into()
    }

    pub fn borrow(&self) -> BadgesRef {
        let input = BorrowBadgesInput {
            badges: self.rid.clone(),
        };
        let output: BorrowBadgesOutput = call_kernel!(BORROW_BADGES, input);

        output.reference.into()
    }

    pub fn amount(&self) -> U256 {
        let input = GetBadgesAmountInput {
            badges: self.rid.clone(),
        };
        let output: GetBadgesAmountOutput = call_kernel!(GET_BADGES_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Resource {
        let input = GetBadgesResourceInput {
            badges: self.rid.clone(),
        };
        let output: GetBadgesResourceOutput = call_kernel!(GET_BADGES_RESOURCE, input);

        output.resource.into()
    }

    pub fn address(&self) -> Address {
        self.resource().address()
    }
}
