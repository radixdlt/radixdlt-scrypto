use scrypto::prelude::*;

use crate::lendingMock::Lending;
use crate::synthetixMock::Synthetix;
use crate::radiswap::Radiswap;

blueprint! {
    struct Farmix {
        lpTokenPool: Vault,
        lendingComponent: Lending,
        synthetixComponent: Synthetix,
        radiswapComponent: Radiswap,
        totalAmount: u32,
        shares: HashMap<u32, u32>,
    }

    impl Farmix {
        pub fn new(
            lendingAddress: Address, 
            synthetixAddress: Address,
            radiswapAddress: Address,
            lpTokenAddress: Address,
        ) -> Component {
            Self {
                lpTokenPool: Vault::new(lpTokenAddress),
                lendingComponent: Lending::from(lendingAddress),
                synthetixComponent: Synthetix::from(synthetixAddress),
                radiswapComponent: Radiswap::from(radiswapAddress),
                totalAmount: 0,
                shares: HashMap::new()
            }
            .instantiate()
        }

        pub fn deposit(&mut self, amount: Bucket, reference: u32) {
            let amountValue: u32 = amount.amount().as_u32();

            let colRatio: u8 = self.lendingComponent.get_collateralization_ratio();
            let loan: Bucket = self.lendingComponent.borrow((amount.amount().as_u32() / colRatio as u32) as u8, amount);
            let synthetic_collateral: Bucket = loan.take(loan.amount() / 2);
            let synthetics: Bucket = self.synthetixComponent.mint(synthetic_collateral);
            let (lpTokens, remainder) = self.radiswapComponent.add_liquidity(loan, synthetics);

            self.lpTokenPool.put(lpTokens);

            self.totalAmount += amountValue;
            self.shares.insert(reference, amountValue / self.totalAmount);
        }

        pub fn withdraw(&mut self, reference: u32) -> Bucket {
            let share: u32 = *self.shares.get(&reference).unwrap();
            let lpTokenAmount: u32 = self.lpTokenPool.amount().as_u32() * share;
            let lpTokens: Bucket = self.lpTokenPool.take(lpTokenAmount);
            let (usdc, synthetic) = self.radiswapComponent.remove_liquidity(lpTokens);
            let moreUSDC = self.radiswapComponent.swap(synthetic);
            usdc.put(moreUSDC);
            return usdc;
        }
    }
}


// borrow(loanAmount: u8, collateral: Bucket) -> Bucket*/