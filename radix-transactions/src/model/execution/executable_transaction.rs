use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableTransaction {
    /// Originally launched with Babylon.
    /// Uses [`InstructionV1`] and [`NotarizedTransactionV1`]`.
    V1(ExecutableTransactionV1),
    /// Originally launched with Cuttlefish.
    /// Supports subintents.
    /// Has support for [`InstructionV2`] and [`NotarizedTransactionV2`]`.
    V2(ExecutableTransactionV2),
}

impl ExecutableTransaction {
    pub fn into_v1(self) -> Option<ExecutableTransactionV1> {
        match self {
            Self::V1(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn uses_free_credits(&self) -> bool {
        match self {
            Self::V1(inner) => inner.costing_parameters().free_credit_in_xrd.is_positive(),
            _ => unimplemented!(),
        }
    }

    pub fn apply_free_credit(self, free_credit: Decimal) -> Self {
        match self {
            Self::V1(inner) => Self::V1(inner.apply_free_credit(free_credit)),
            _ => unimplemented!(),
        }
    }

    pub fn skip_epoch_range_check(self) -> Self {
        match self {
            Self::V1(inner) => Self::V1(inner.skip_epoch_range_check()),
            _ => unimplemented!(),
        }
    }
}

impl From<ExecutableTransactionV1> for ExecutableTransaction {
    fn from(value: ExecutableTransactionV1) -> Self {
        Self::V1(value)
    }
}

impl From<ExecutableTransactionV2> for ExecutableTransaction {
    fn from(value: ExecutableTransactionV2) -> Self {
        Self::V2(value)
    }
}