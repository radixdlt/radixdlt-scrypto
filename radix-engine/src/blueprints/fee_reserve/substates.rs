use crate::system::kernel_modules::costing::{FeeTable, SystemLoanFeeReserve};

#[derive(Debug)]
pub struct FeeReserveSubstate {
    pub fee_reserve: SystemLoanFeeReserve,
    pub fee_table: FeeTable,
}

impl FeeReserveSubstate {}
