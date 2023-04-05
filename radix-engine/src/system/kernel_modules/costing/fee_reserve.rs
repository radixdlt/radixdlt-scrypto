use super::FeeSummary;
use crate::{errors::CanBeAbortion, transaction::AbortReason, types::*};
use radix_engine_constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use sbor::rust::cmp::min;
use strum::EnumCount;

// Note: for performance reason, `u128` is used to represent decimal in this file.

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FeeReserveError {
    InsufficientBalance,
    Overflow,
    LimitExceeded {
        limit: u32,
        committed: u32,
        new: u32,
    },
    LoanRepaymentFailed,
    Abort(AbortReason),
}

impl CanBeAbortion for FeeReserveError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::Abort(reason) => Some(reason),
            _ => None,
        }
    }
}

pub trait PreExecutionFeeReserve {
    /// This is only allowed before a transaction properly begins.
    /// After any other methods are called, this cannot be called again.
    fn consume_deferred(
        &mut self,
        amount: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError>;
}

pub trait ExecutionFeeReserve {
    fn consume_royalty(
        &mut self,
        cost_units: u32,
        recipient: RoyaltyRecipient,
        recipient_vault_id: NodeId,
    ) -> Result<(), FeeReserveError>;

    fn consume_multiplied_execution(
        &mut self,
        cost_units_per_multiple: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError>;

    fn consume_execution(
        &mut self,
        cost_units: u32,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError>;

    fn lock_fee(
        &mut self,
        vault_id: NodeId,
        fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError>;
}

pub trait FinalizingFeeReserve {
    fn finalize(self) -> FeeSummary;
}

pub trait FeeReserve: PreExecutionFeeReserve + ExecutionFeeReserve + FinalizingFeeReserve {}

#[repr(usize)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoSbor,
    IntoStaticStr,
    EnumCount,
    Display,
    FromRepr,
)]
pub enum CostingReason {
    TxBaseCost,
    TxPayloadCost,
    TxSignatureVerification,
    Invoke,
    DropNode,
    CreateNode,
    LockSubstate,
    ReadSubstate,
    WriteSubstate,
    DropLock,
    RunWasm,
    RunNative,
    RunSystem,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub enum RoyaltyRecipient {
    Package(PackageAddress),
    Component(ComponentAddress),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct SystemLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: u128,
    /// The tip percentage
    tip_percentage: u16,
    /// The number of cost units that can be consumed at most
    cost_unit_limit: u32,
    /// The number of cost units from system loan
    system_loan: u32,
    /// Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    abort_when_loan_repaid: bool,

    /// (Cache) The effective execution price
    effective_execution_price: u128,
    /// (Cache) The effective royalty price
    effective_royalty_price: u128,

    /// The XRD balance
    xrd_balance: u128,
    /// The amount of XRD owed to the system
    xrd_owed: u128,

    /// Execution costs committed
    execution_committed: [u32; CostingReason::COUNT],
    execution_committed_sum: u32,
    /// Execution costs deferred
    execution_deferred: [u32; CostingReason::COUNT],

    /// Royalty costs
    royalty_committed: BTreeMap<RoyaltyRecipient, (NodeId, u128)>,
    royalty_committed_sum: u32,

    /// Payments made during the execution of a transaction.
    payments: Vec<(NodeId, LiquidFungibleResource, bool)>,
}

#[inline]
fn checked_add(a: u32, b: u32) -> Result<u32, FeeReserveError> {
    a.checked_add(b).ok_or(FeeReserveError::Overflow)
}

#[inline]
fn checked_assign_add(value: &mut u32, summand: u32) -> Result<(), FeeReserveError> {
    *value = checked_add(*value, summand)?;
    Ok(())
}

#[inline]
fn checked_multiply(amount: u32, multiplier: usize) -> Result<u32, FeeReserveError> {
    u32::try_from(multiplier)
        .map_err(|_| FeeReserveError::Overflow)
        .and_then(|x| x.checked_mul(amount).ok_or(FeeReserveError::Overflow))
}

pub fn u128_to_decimal(a: u128) -> Decimal {
    Decimal(a.into())
}

pub fn decimal_to_u128(a: Decimal) -> u128 {
    let i256 = a.0;
    i256.try_into().expect("Overflow")
}

impl SystemLoanFeeReserve {
    pub fn no_fee() -> Self {
        Self::new(0, 0, DEFAULT_COST_UNIT_LIMIT, DEFAULT_SYSTEM_LOAN, false)
    }

    pub fn new(
        cost_unit_price: u128,
        tip_percentage: u16,
        cost_unit_limit: u32,
        system_loan: u32,
        abort_when_loan_repaid: bool,
    ) -> Self {
        let effective_execution_price =
            cost_unit_price + cost_unit_price * tip_percentage as u128 / 100;
        let effective_royalty_price = cost_unit_price;

        Self {
            cost_unit_price,
            tip_percentage,
            cost_unit_limit,
            system_loan,
            abort_when_loan_repaid,

            effective_execution_price,
            effective_royalty_price,

            // System loan is used for both execution and royalty
            xrd_balance: cost_unit_price * system_loan as u128,
            xrd_owed: cost_unit_price * system_loan as u128,

            execution_committed: [0u32; CostingReason::COUNT],
            execution_committed_sum: 0,
            execution_deferred: [0u32; CostingReason::COUNT],
            royalty_committed: BTreeMap::new(),
            royalty_committed_sum: 0,

            payments: Vec::new(),
        }
    }

    fn check_cost_unit_limit(&self, cost_units: u32) -> Result<(), FeeReserveError> {
        if checked_add(
            self.execution_committed_sum,
            checked_add(self.royalty_committed_sum, cost_units)?,
        )? > self.cost_unit_limit
        {
            return Err(FeeReserveError::LimitExceeded {
                limit: self.cost_unit_limit,
                committed: self.execution_committed_sum + self.royalty_committed_sum,
                new: cost_units,
            });
        }
        Ok(())
    }

    fn consume_execution_internal(
        &mut self,
        cost_units: u32,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        self.check_cost_unit_limit(cost_units)?;

        let amount = self.effective_execution_price * cost_units as u128;
        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance);
        } else {
            self.xrd_balance -= amount;
            self.execution_committed[reason as usize] += cost_units;
            self.execution_committed_sum += cost_units;
            Ok(())
        }
    }

    fn consume_royalty_internal(
        &mut self,
        cost_units: u32,
        recipient: RoyaltyRecipient,
        recipient_vault_id: NodeId,
    ) -> Result<(), FeeReserveError> {
        self.check_cost_unit_limit(cost_units)?;

        let amount = self.effective_royalty_price * cost_units as u128;
        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance);
        } else {
            self.xrd_balance -= amount;
            self.royalty_committed
                .entry(recipient)
                .or_insert((recipient_vault_id, 0))
                .1
                .add_assign(amount);
            self.royalty_committed_sum += cost_units;
            Ok(())
        }
    }

    pub fn repay_all(&mut self) -> Result<(), FeeReserveError> {
        // Apply deferred execution cost
        for i in 0..CostingReason::COUNT {
            let cost_units = self.execution_deferred[i];
            self.consume_execution_internal(cost_units, CostingReason::from_repr(i).unwrap())?;
            self.execution_deferred[i] = 0;
        }

        // Repay owed with balance
        self.xrd_owed -= min(self.xrd_balance, self.xrd_owed);

        // Check outstanding loan
        if self.xrd_owed != 0 {
            return Err(FeeReserveError::LoanRepaymentFailed);
        }

        if self.abort_when_loan_repaid {
            return Err(FeeReserveError::Abort(
                AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment,
            ));
        }

        Ok(())
    }

    pub fn revert_royalty(&mut self) {
        self.xrd_balance += self.royalty_committed.values().map(|x| x.1).sum::<u128>();
        self.royalty_committed.clear();
        self.royalty_committed_sum = 0;
    }

    pub fn royalty_cost(&self) -> BTreeMap<RoyaltyRecipient, (NodeId, Decimal)> {
        self.royalty_committed
            .clone()
            .into_iter()
            .map(|(k, v)| (k, (v.0, u128_to_decimal(v.1))))
            .collect()
    }

    pub fn execution_cost(&self) -> BTreeMap<CostingReason, u32> {
        self.execution_committed
            .into_iter()
            .enumerate()
            .filter_map(|(i, sum)| {
                if sum == 0 {
                    None
                } else {
                    Some((CostingReason::from_repr(i).unwrap(), sum))
                }
            })
            .collect()
    }

    #[inline]
    pub fn fully_repaid(&self) -> bool {
        self.xrd_owed == 0
    }
}

impl PreExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_deferred(
        &mut self,
        cost_units: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        checked_assign_add(
            &mut self.execution_deferred[reason as usize],
            checked_multiply(cost_units, multiplier)?,
        )?;

        Ok(())
    }
}

impl ExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_royalty(
        &mut self,
        cost_units: u32,
        recipient: RoyaltyRecipient,
        recipient_vault_id: NodeId,
    ) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume_royalty_internal(cost_units, recipient, recipient_vault_id)?;

        if !self.fully_repaid() && self.execution_committed_sum >= self.system_loan {
            self.repay_all()?;
        }

        Ok(())
    }

    fn consume_execution(
        &mut self,
        cost_units: u32,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume_execution_internal(cost_units, reason)?;

        if !self.fully_repaid() && self.execution_committed_sum >= self.system_loan {
            self.repay_all()?;
        }

        Ok(())
    }

    fn consume_multiplied_execution(
        &mut self,
        cost_units_per_multiple: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        if multiplier == 0 {
            return Ok(());
        }

        self.consume_execution(
            checked_multiply(cost_units_per_multiple, multiplier)?,
            reason,
        )
    }

    fn lock_fee(
        &mut self,
        vault_id: NodeId,
        mut fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError> {
        // Update balance
        if !contingent {
            // Assumption: no overflow due to limited XRD supply
            self.xrd_balance += decimal_to_u128(fee.amount());
        }

        // Move resource
        self.payments.push((vault_id, fee.take_all(), contingent));

        Ok(fee)
    }
}

impl FinalizingFeeReserve for SystemLoanFeeReserve {
    fn finalize(self) -> FeeSummary {
        let execution_cost_breakdown = self.execution_cost();
        let royalty_cost_breakdown = self.royalty_cost();
        let total_royalty_cost_xrd = royalty_cost_breakdown.values().map(|x| x.1).sum();
        FeeSummary {
            cost_unit_limit: self.cost_unit_limit,
            cost_unit_price: u128_to_decimal(self.cost_unit_price),
            tip_percentage: self.tip_percentage,
            total_execution_cost_xrd: u128_to_decimal(
                self.effective_execution_price * self.execution_committed_sum as u128,
            ),
            total_royalty_cost_xrd,
            total_bad_debt_xrd: u128_to_decimal(self.xrd_owed),
            locked_fees: self.payments,
            execution_cost_breakdown,
            execution_cost_sum: self.execution_committed_sum,
            royalty_cost_breakdown,
        }
    }
}

impl FeeReserve for SystemLoanFeeReserve {}

impl Default for SystemLoanFeeReserve {
    fn default() -> Self {
        Self::new(
            DEFAULT_COST_UNIT_PRICE,
            0,
            DEFAULT_COST_UNIT_LIMIT,
            DEFAULT_SYSTEM_LOAN,
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_COMPONENT: ComponentAddress =
        component_address(EntityType::GlobalGenericComponent, 5);
    const TEST_VAULT_ID: NodeId = NodeId([0u8; NodeId::LENGTH]);
    const TEST_VAULT_ID_2: NodeId = NodeId([1u8; NodeId::LENGTH]);

    fn xrd<T: Into<Decimal>>(amount: T) -> LiquidFungibleResource {
        LiquidFungibleResource::new(amount.into())
    }

    #[test]
    fn test_consume_and_repay() {
        let mut fee_reserve = SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 5, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, CostingReason::Invoke)
            .unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("2") + dec!("0.04"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 5, false);
        assert_eq!(
            Err(FeeReserveError::InsufficientBalance),
            fee_reserve.consume_multiplied_execution(6, 1, CostingReason::Invoke)
        );
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_lock_fee() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(5)), 0, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
        assert_eq!(summary.locked_fees, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }

    #[test]
    fn test_bad_debt() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(5)), 1, 100, 50, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, CostingReason::Invoke)
            .unwrap();
        assert_eq!(
            fee_reserve.repay_all(),
            Err(FeeReserveError::LoanRepaymentFailed)
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), false);
        assert_eq!(summary.execution_cost_sum, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("10.1"));
        assert_eq!(summary.locked_fees, vec![],);
    }

    #[test]
    fn test_royalty_execution_mix() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(5)), 1, 100, 50, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, CostingReason::Invoke)
            .unwrap();
        fee_reserve
            .consume_royalty(2, RoyaltyRecipient::Package(PACKAGE_PACKAGE), TEST_VAULT_ID)
            .unwrap();
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("10"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
        assert_eq!(summary.locked_fees, vec![(TEST_VAULT_ID, xrd(100), false)]);
        assert_eq!(
            summary.execution_cost_breakdown,
            btreemap!(
                CostingReason::Invoke => 2
            )
        );
        assert_eq!(summary.execution_cost_sum, 2);
        assert_eq!(
            summary.royalty_cost_breakdown,
            btreemap!(
                RoyaltyRecipient::Package(PACKAGE_PACKAGE) => (TEST_VAULT_ID, dec!("10"))
            )
        );
    }

    #[test]
    fn test_royalty_insufficient_balance() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 0, 1000, 50, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve
            .consume_royalty(
                90,
                RoyaltyRecipient::Package(PACKAGE_PACKAGE),
                TEST_VAULT_ID,
            )
            .unwrap();
        assert_eq!(
            fee_reserve.consume_royalty(
                80,
                RoyaltyRecipient::Component(TEST_COMPONENT),
                TEST_VAULT_ID_2
            ),
            Err(FeeReserveError::InsufficientBalance)
        );
    }

    #[test]
    fn test_royalty_exceeds_cost_unit_limit() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 0, 100, 50, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(500), false)
            .unwrap();
        assert_eq!(
            fee_reserve.consume_royalty(
                200,
                RoyaltyRecipient::Component(TEST_COMPONENT),
                TEST_VAULT_ID_2
            ),
            Err(FeeReserveError::LimitExceeded {
                limit: 100,
                committed: 0,
                new: 200
            })
        );
    }
}
