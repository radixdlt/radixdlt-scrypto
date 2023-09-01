pub mod consensus_manager;
pub mod multi_pool;
pub mod one_pool;
pub mod resource;
pub mod two_pool;
pub mod validator;

use radix_engine::transaction::TransactionOutcome;
use radix_engine::types::*;
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct TestFuzzer {
    rng: ChaCha8Rng,
}

impl TestFuzzer {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        Self { rng }
    }

    pub fn next_amount(&mut self) -> Decimal {
        let next_amount_type = self.rng.gen_range(0u32..=8u32);
        match next_amount_type {
            0 => Decimal::ZERO,
            1 => Decimal::ONE,
            2 => Decimal::MAX,
            3 => Decimal::MIN,
            4 => Decimal(I192::ONE),
            5 => {
                let amount = self.rng.gen_range(0u64..u64::MAX);
                Decimal::from(amount)
            }
            6 => {
                let amount = self.rng.gen_range(1000u64..10000u64);
                Decimal::from(amount)
            }
            7 => {
                let mut bytes = [0u8; 24];
                let (start, _end) = bytes.split_at_mut(8);
                self.rng.fill_bytes(start);
                Decimal(I192::from_le_bytes(&bytes))
            }
            _ => {
                let mut bytes = [0u8; 24];
                self.rng.fill_bytes(&mut bytes);
                Decimal(I192::from_le_bytes(&bytes))
            }
        }
    }

    pub fn next_usize(&mut self, count: usize) -> usize {
        self.rng.gen_range(0usize..count)
    }

    pub fn next_u8(&mut self, count: u8) -> u8 {
        self.rng.gen_range(0u8..count)
    }

    pub fn next_valid_divisibility(&mut self) -> u8 {
        self.rng.gen_range(0u8..=18u8)
    }

    pub fn next_u32(&mut self, count: u32) -> u32 {
        self.rng.gen_range(0u32..count)
    }

    pub fn next_integer_non_fungible_id(&mut self) -> NonFungibleLocalId {
        NonFungibleLocalId::integer(self.rng.gen_range(0u64..4u64))
    }

    pub fn next_non_fungible_id_set(&mut self) -> BTreeSet<NonFungibleLocalId> {
        (0u64..self.rng.gen_range(0u64..4u64))
            .into_iter()
            .map(|_| self.next_integer_non_fungible_id())
            .collect()
    }

    pub fn next<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.rng.gen_range(range)
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

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum FuzzTxnResult {
    TrivialSuccess,
    Success,
    TrivialFailure,
    Failure,
}

impl FuzzTxnResult {
    pub fn from_outcome(outcome: &TransactionOutcome, trivial: bool) -> Self {
        match (outcome, trivial) {
            (TransactionOutcome::Success(..), true) => FuzzTxnResult::TrivialSuccess,
            (TransactionOutcome::Success(..), false) => FuzzTxnResult::Success,
            (TransactionOutcome::Failure(..), true) => FuzzTxnResult::TrivialFailure,
            (TransactionOutcome::Failure(..), false) => FuzzTxnResult::Failure,
        }
    }
}
