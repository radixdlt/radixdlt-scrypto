use crate::engine::{deref_and_update, Executor};
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
        let actor =
            ResolvedActor::function(NativeFn::AccessController(AccessControllerFn::CreateGlobal));

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.controlled_asset));

        Ok((actor, call_frame_update, self))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CreateProof),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CreateProof),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            timed_recovery_delay_in_hours: self.timed_recovery_delay_in_hours,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsConfirmation),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecoveryAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecoveryAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecoveryAsConfirmation),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::TimedConfirmRecoveryAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::TimedConfirmRecoveryAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::TimedConfirmRecoveryAsConfirmation),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttemptAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttemptAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttemptAsConfirmation),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::LockPrimaryRole),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
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
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::UnlockPrimaryRole),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
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
