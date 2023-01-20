use crate::engine::Executor;
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::*;

//=================================
// Access Controller Create Global
//=================================

impl ExecutableInvocation for AccessControllerCreateGlobalInvocation {
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

impl Executor for AccessControllerCreateGlobalInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//================================
// Access Controller Create Proof
//================================

impl ExecutableInvocation for AccessControllerCreateProofInvocation {
    type Exec = AccessControllerCreateProofExecutable;

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

impl Executor for AccessControllerCreateProofExecutable {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//===============================================
// Access Controller Update Timed Recovery Delay
//===============================================

impl ExecutableInvocation for AccessControllerUpdateTimedRecoveryDelayInvocation {
    type Exec = AccessControllerUpdateTimedRecoveryDelayExecutable;

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

impl Executor for AccessControllerUpdateTimedRecoveryDelayExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//================================================
// Access Controller Initiate Recovery As Primary
//================================================

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    type Exec = AccessControllerInitiateRecoveryAsPrimaryExecutable;

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

impl Executor for AccessControllerInitiateRecoveryAsPrimaryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=================================================
// Access Controller Initiate Recovery As Recovery
//=================================================

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    type Exec = AccessControllerInitiateRecoveryAsRecoveryExecutable;

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

impl Executor for AccessControllerInitiateRecoveryAsRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=====================================================
// Access Controller Initiate Recovery As Confirmation
//=====================================================

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsConfirmationInvocation {
    type Exec = AccessControllerInitiateRecoveryAsConfirmationExecutable;

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

impl Executor for AccessControllerInitiateRecoveryAsConfirmationExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=====================================================
// Access Controller Quick Confirm Recovery As Primary
//=====================================================

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryAsPrimaryExecutable;

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

impl Executor for AccessControllerQuickConfirmRecoveryAsPrimaryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//======================================================
// Access Controller Quick Confirm Recovery As Recovery
//======================================================

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryAsRecoveryExecutable;

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

impl Executor for AccessControllerQuickConfirmRecoveryAsRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//==========================================================
// Access Controller Quick Confirm Recovery As Confirmation
//==========================================================

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryAsConfirmationExecutable;

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

impl Executor for AccessControllerQuickConfirmRecoveryAsConfirmationExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=====================================================
// Access Controller Timed Confirm Recovery As Primary
//=====================================================

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryAsPrimaryInvocation {
    type Exec = AccessControllerTimedConfirmRecoveryAsPrimaryExecutable;

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

impl Executor for AccessControllerTimedConfirmRecoveryAsPrimaryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//======================================================
// Access Controller Timed Confirm Recovery As Recovery
//======================================================

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryAsRecoveryInvocation {
    type Exec = AccessControllerTimedConfirmRecoveryAsRecoveryExecutable;

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

impl Executor for AccessControllerTimedConfirmRecoveryAsRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//==========================================================
// Access Controller Timed Confirm Recovery As Confirmation
//==========================================================

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryAsConfirmationInvocation {
    type Exec = AccessControllerTimedConfirmRecoveryAsConfirmationExecutable;

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

impl Executor for AccessControllerTimedConfirmRecoveryAsConfirmationExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//======================================================
// Access Controller Cancel Recovery Attempt As Primary
//======================================================

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    type Exec = AccessControllerCancelRecoveryAttemptAsPrimaryExecutable;

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

impl Executor for AccessControllerCancelRecoveryAttemptAsPrimaryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=======================================================
// Access Controller Cancel Recovery Attempt As Recovery
//=======================================================

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    type Exec = AccessControllerCancelRecoveryAttemptAsRecoveryExecutable;

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

impl Executor for AccessControllerCancelRecoveryAttemptAsRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//===========================================================
// Access Controller Cancel Recovery Attempt As Confirmation
//===========================================================

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptAsConfirmationInvocation {
    type Exec = AccessControllerCancelRecoveryAttemptAsConfirmationExecutable;

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

impl Executor for AccessControllerCancelRecoveryAttemptAsConfirmationExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=====================================
// Access Controller Lock Primary Role
//=====================================

impl ExecutableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type Exec = AccessControllerLockPrimaryRoleExecutable;

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

impl Executor for AccessControllerLockPrimaryRoleExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}

//=======================================
// Access Controller Unlock Primary Role
//=======================================

impl ExecutableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type Exec = AccessControllerUnlockPrimaryRoleExecutable;

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

impl Executor for AccessControllerUnlockPrimaryRoleExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        _api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        todo!()
    }
}
