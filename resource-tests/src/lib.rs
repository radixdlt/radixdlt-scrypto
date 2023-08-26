use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use radix_engine::types::*;
use transaction::prelude::*;

pub struct ResourceTestFuzzer {
    rng: ChaCha8Rng,
}

impl ResourceTestFuzzer {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        Self {
            rng,
        }
    }

    pub fn next_amount(&mut self) -> Decimal {
        let next_amount_type = self.rng.gen_range(0u32..6u32);
        match next_amount_type {
            0 => Decimal::ZERO,
            1 => Decimal::ONE,
            2 => Decimal::MAX,
            3 => Decimal::MIN,
            4 => Decimal(I192::ONE),
            _ => {
                let mut bytes = [0u8; 24];
                self.rng.fill_bytes(&mut bytes);
                Decimal(I192::from_le_bytes(&bytes))
            }
        }
    }

    pub fn next_u32(&mut self, count: u32) -> u32 {
        self.rng.gen_range(0u32..count)
    }

    pub fn next_withdraw_strategy(&mut self) -> WithdrawStrategy {
        match self.next_u32(4) {
            0u32 => WithdrawStrategy::Exact,
            1u32 => WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
            2u32 => WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointAwayFromZero),
            3u32 => WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
            4u32 => WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointTowardZero),
            5u32 => WithdrawStrategy::Rounded(RoundingMode::ToNegativeInfinity),
            6u32 => WithdrawStrategy::Rounded(RoundingMode::ToPositiveInfinity),
            _ => WithdrawStrategy::Rounded(RoundingMode::ToZero),
        }
    }
}