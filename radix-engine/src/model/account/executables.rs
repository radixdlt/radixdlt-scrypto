use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, Executor, ResolvedActor, ResolverApi, RuntimeError,
    SystemApi,
};
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::ComponentId;
use radix_engine_interface::api::EngineApi;
use radix_engine_interface::model::*;

//================
// Account Create
//================

impl ExecutableInvocation for AccountCreateInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountCreateInvocation {
    type Output = ComponentId;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=============
// Account New
//=============

impl ExecutableInvocation for AccountNewInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountNewInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//===========================
// Account New With Resource
//===========================

impl ExecutableInvocation for AccountNewWithResourceInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountNewWithResourceInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//==================
// Account Lock Fee
//==================

impl ExecutableInvocation for AccountLockFeeInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountLockFeeInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=============================
// Account Lock Contingent Fee
//=============================

impl ExecutableInvocation for AccountLockContingentFeeInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountLockContingentFeeInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=================
// Account Deposit
//=================

impl ExecutableInvocation for AccountDepositInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountDepositInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=======================
// Account Deposit Batch
//=======================

impl ExecutableInvocation for AccountDepositBatchInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountDepositBatchInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//==================
// Account Withdraw
//==================

impl ExecutableInvocation for AccountWithdrawInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountWithdrawInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//============================
// Account Withdraw By Amount
//============================

impl ExecutableInvocation for AccountWithdrawByAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountWithdrawByAmountInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=========================
// Account Withdraw By Ids
//=========================

impl ExecutableInvocation for AccountWithdrawByIdsInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountWithdrawByIdsInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//===========================
// Account Withdraw And Lock
//===========================

impl ExecutableInvocation for AccountLockFeeAndWithdrawInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountLockFeeAndWithdrawInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=====================================
// Account Withdraw By Amount And Lock
//=====================================

impl ExecutableInvocation for AccountLockFeeAndWithdrawByAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountLockFeeAndWithdrawByAmountInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//==================================
// Account Withdraw By Ids And Lock
//==================================

impl ExecutableInvocation for AccountLockFeeAndWithdrawByIdsInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountLockFeeAndWithdrawByIdsInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//======================
// Account Create Proof
//======================

impl ExecutableInvocation for AccountCreateProofInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountCreateProofInvocation {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//================================
// Account Create Proof By Amount
//================================

impl ExecutableInvocation for AccountCreateProofByAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountCreateProofByAmountInvocation {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

//=============================
// Account Create Proof By Ids
//=============================

impl ExecutableInvocation for AccountCreateProofByIdsInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Executor for AccountCreateProofByIdsInvocation {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}
