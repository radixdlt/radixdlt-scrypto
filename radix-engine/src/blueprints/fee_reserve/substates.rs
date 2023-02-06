use crate::system::kernel_modules::fee::SystemLoanFeeReserve;

#[derive(Debug)]
pub struct FeeReserveSubstate {
    pub fee_reserve: SystemLoanFeeReserve,
}

impl FeeReserveSubstate {}
