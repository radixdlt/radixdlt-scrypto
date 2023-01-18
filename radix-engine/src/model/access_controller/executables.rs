use crate::engine::{
    deref_and_update, CallFrameUpdate, ExecutableInvocation, Executor, ResolvedActor, ResolverApi,
    RuntimeError, SystemApi,
};
use crate::types::AccessControllerFn;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{GlobalAddress, NativeFn, RENodeId};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessControllerError {
    RuleAssertionFailed { asserted_against: AccessRule },
}

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
        let call_frame_update = CallFrameUpdate::empty();
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
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        todo!()
    }
}

impl ExecutableInvocation for AccessControllerCreateProofInvocation {
    type Exec = Self;

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

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerCreateProofInvocation {
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

impl ExecutableInvocation for AccessControllerUpdateTimedRecoveryDelayInvocation {
    type Exec = Self;

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
            NativeFn::AccessController(AccessControllerFn::UpdateTimedRecoveryDelay),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerUpdateTimedRecoveryDelayInvocation {
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

impl ExecutableInvocation for AccessControllerInitiateRecoveryInvocation {
    type Exec = Self;

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
            NativeFn::AccessController(AccessControllerFn::InitiateRecovery),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerInitiateRecoveryInvocation {
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

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryInvocation {
    type Exec = Self;

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
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecovery),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerQuickConfirmRecoveryInvocation {
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

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryInvocation {
    type Exec = Self;

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
            NativeFn::AccessController(AccessControllerFn::TimedConfirmRecovery),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerTimedConfirmRecoveryInvocation {
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

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptInvocation {
    type Exec = Self;

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
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttempt),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerCancelRecoveryAttemptInvocation {
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

impl ExecutableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type Exec = Self;

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

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerLockPrimaryRoleInvocation {
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

impl ExecutableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type Exec = Self;

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

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerUnlockPrimaryRoleInvocation {
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
