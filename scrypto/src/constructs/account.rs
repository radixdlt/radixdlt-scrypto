use crate::kernel::*;
use crate::resource::*;
use crate::types::*;

/// An account holds tokens and badges.
#[derive(Debug)]
pub struct Account {
    address: Address,
}

impl From<Address> for Account {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Into<Address> for Account {
    fn into(self) -> Address {
        self.address
    }
}

impl Account {
    fn withdraw(&mut self, amount: U256, resource: Address) -> BID {
        let input = WithdrawInput {
            account: self.address,
            amount,
            resource,
        };
        let output: WithdrawOutput = call_kernel(WITHDRAW, input);

        output.bucket
    }

    fn deposit(&mut self, bucket: BID) {
        let input = DepositInput {
            account: self.address,
            bucket: bucket,
        };
        let _: DepositOutput = call_kernel(DEPOSIT, input);
    }

    pub fn withdraw_tokens(&mut self, amount: U256, resource: Address) -> Tokens {
        self.withdraw(amount, resource).into()
    }

    pub fn deposit_tokens(&mut self, tokens: Tokens) {
        self.deposit(tokens.into());
    }

    pub fn withdraw_badges(&mut self, amount: U256, resource: Address) -> Badges {
        self.withdraw(amount, resource).into()
    }

    pub fn deposit_badges(&mut self, badges: Badges) {
        self.deposit(badges.into());
    }

    pub fn address(&self) -> Address {
        self.address
    }
}
