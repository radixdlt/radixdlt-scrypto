use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use sbor::rust::fmt::Debug;
use sbor::*;
use scrypto::resource::AuthZoneDrainInput;

impl<E: Into<ApplicationError>> Into<RuntimeError> for InvokeError<E> {
    fn into(self) -> RuntimeError {
        match self {
            InvokeError::Downstream(runtime_error) => runtime_error,
            InvokeError::Error(e) => RuntimeError::ApplicationError(e.into()),
        }
    }
}

impl Into<ApplicationError> for TransactionProcessorError {
    fn into(self) -> ApplicationError {
        ApplicationError::TransactionProcessorError(self)
    }
}

impl Into<ApplicationError> for PackageError {
    fn into(self) -> ApplicationError {
        ApplicationError::PackageError(self)
    }
}

impl Into<ApplicationError> for ResourceManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::ResourceManagerError(self)
    }
}

impl Into<ApplicationError> for BucketError {
    fn into(self) -> ApplicationError {
        ApplicationError::BucketError(self)
    }
}

impl Into<ApplicationError> for ProofError {
    fn into(self) -> ApplicationError {
        ApplicationError::ProofError(self)
    }
}

impl Into<ApplicationError> for AuthZoneError {
    fn into(self) -> ApplicationError {
        ApplicationError::AuthZoneError(self)
    }
}

impl Into<ApplicationError> for WorktopError {
    fn into(self) -> ApplicationError {
        ApplicationError::WorktopError(self)
    }
}

impl Into<ApplicationError> for VaultError {
    fn into(self) -> ApplicationError {
        ApplicationError::VaultError(self)
    }
}

impl Into<ApplicationError> for ComponentError {
    fn into(self) -> ApplicationError {
        ApplicationError::ComponentError(self)
    }
}

impl Into<ApplicationError> for EpochManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::EpochManagerError(self)
    }
}

pub trait InvokableNative<'a>:
    Invokable<EpochManagerCreateInput>
    + Invokable<PackagePublishInput>
    + Invokable<ResourceManagerBurnInput>
    + Invokable<ResourceManagerCreateInput>
    + Invokable<TransactionProcessorRunInput<'a>>
    + Invokable<BucketTakeInput>
    + Invokable<BucketCreateProofInput>
    + Invokable<BucketTakeNonFungiblesInput>
    + Invokable<BucketGetNonFungibleIdsInput>
    + Invokable<BucketGetAmountInput>
    + Invokable<BucketPutInput>
    + Invokable<BucketGetResourceAddressInput>
    + Invokable<AuthZonePopInput>
    + Invokable<AuthZonePushInput>
    + Invokable<AuthZoneCreateProofInput>
    + Invokable<AuthZoneCreateProofByAmountInput>
    + Invokable<AuthZoneCreateProofByIdsInput>
    + Invokable<AuthZoneClearInput>
    + Invokable<AuthZoneDrainInput>
    + Invokable<ProofGetAmountInput>
    + Invokable<ProofGetNonFungibleIdsInput>
    + Invokable<ProofGetResourceAddressInput>
    + Invokable<ProofCloneInput>
    + Invokable<WorktopPutInput>
    + Invokable<WorktopTakeAmountInput>
    + Invokable<WorktopTakeAllInput>
    + Invokable<WorktopTakeNonFungiblesInput>
    + Invokable<WorktopAssertContainsInput>
    + Invokable<WorktopAssertContainsAmountInput>
    + Invokable<WorktopAssertContainsNonFungiblesInput>
    + Invokable<WorktopDrainInput>
    + Invokable<VaultTakeInput>
    + Invokable<VaultPutInput>
    + Invokable<VaultLockFeeInput>
    + Invokable<VaultTakeNonFungiblesInput>
    + Invokable<VaultGetAmountInput>
    + Invokable<VaultGetResourceAddressInput>
    + Invokable<VaultGetNonFungibleIdsInput>
    + Invokable<VaultCreateProofInput>
    + Invokable<VaultCreateProofByAmountInput>
    + Invokable<VaultCreateProofByIdsInput>
    + Invokable<ComponentAddAccessCheckInput>
    + Invokable<ResourceManagerBurnInput>
    + Invokable<ResourceManagerUpdateAuthInput>
    + Invokable<ResourceManagerLockAuthInput>
    + Invokable<ResourceManagerCreateVaultInput>
    + Invokable<ResourceManagerCreateBucketInput>
    + Invokable<ResourceManagerMintInput>
    + Invokable<ResourceManagerGetMetadataInput>
    + Invokable<ResourceManagerGetResourceTypeInput>
    + Invokable<ResourceManagerGetTotalSupplyInput>
    + Invokable<ResourceManagerUpdateMetadataInput>
    + Invokable<ResourceManagerUpdateNonFungibleDataInput>
    + Invokable<ResourceManagerNonFungibleExistsInput>
    + Invokable<ResourceManagerGetNonFungibleInput>
    + Invokable<ResourceManagerSetResourceAddressInput>
    + Invokable<EpochManagerGetCurrentEpochInput>
{
}

// TODO: This should be cleaned up
pub enum NativeInvocationInfo {
    Function(NativeFunction, CallFrameUpdate),
    Method(NativeMethod, RENodeId, CallFrameUpdate),
}

impl<N: NativeExecutable> Invocation for N {
    type Output = <N as NativeExecutable>::Output;
}

pub struct NativeResolver;

impl<N: NativeInvocation> Resolver<N> for NativeResolver {
    type Exec = NativeExecutor<N>;

    fn resolve<D: MethodDeref>(
        invocation: N,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let info = invocation.info();
        let (actor, call_frame_update) = match info {
            NativeInvocationInfo::Method(method, receiver, mut call_frame_update) => {
                // TODO: Move this logic into kernel
                let resolved_receiver = if let Some(derefed) = deref.deref(receiver)? {
                    ResolvedReceiver::derefed(derefed, receiver)
                } else {
                    ResolvedReceiver::new(receiver)
                };
                let resolved_node_id = resolved_receiver.receiver;
                call_frame_update.node_refs_to_copy.insert(resolved_node_id);

                let actor = REActor::Method(ResolvedMethod::Native(method), resolved_receiver);
                (actor, call_frame_update)
            }
            NativeInvocationInfo::Function(native_function, call_frame_update) => {
                let actor = REActor::Function(ResolvedFunction::Native(native_function));
                (actor, call_frame_update)
            }
        };

        let input = ScryptoValue::from_typed(&invocation);
        let executor = NativeExecutor(invocation, input);
        Ok((actor, call_frame_update, executor))
    }
}

pub trait NativeInvocation: NativeExecutable + Encode + Debug {
    fn info(&self) -> NativeInvocationInfo;
}

pub trait NativeExecutable: Invocation {
    type Output: Debug;

    fn execute<'s, 'a, Y, R>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNative<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve;
}

pub struct NativeExecutor<N: NativeExecutable>(pub N, pub ScryptoValue);

impl<N: NativeExecutable> Executor for NativeExecutor<N> {
    type Output = <N as Invocation>::Output;

    fn args(&self) -> &ScryptoValue {
        &self.1
    }

    fn execute<'s, 'a, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNative<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        N::execute(self.0, system_api)
    }
}

pub struct NativeMethodExecutor(pub NativeMethod, pub ResolvedReceiver, pub ScryptoValue);

impl Executor for NativeMethodExecutor {
    type Output = ScryptoValue;

    fn args(&self) -> &ScryptoValue {
        &self.2
    }

    fn execute<'s, 'a, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNative<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let output = match (self.1.receiver, self.0) {
            (RENodeId::AuthZoneStack(..), NativeMethod::AuthZone(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::Bucket(..), NativeMethod::Bucket(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::Proof(..), NativeMethod::Proof(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::Worktop, NativeMethod::Worktop(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::Vault(..), NativeMethod::Vault(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::Component(..), NativeMethod::Component(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::ResourceManager(..), NativeMethod::ResourceManager(..)) => {
                panic!("Unexpected")
            }
            (RENodeId::EpochManager(component_id), NativeMethod::EpochManager(method)) => {
                EpochManager::main(component_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (receiver, native_method) => {
                return Err(RuntimeError::KernelError(
                    KernelError::MethodReceiverNotMatch(native_method, receiver),
                ));
            }
        }?;

        let update = CallFrameUpdate {
            node_refs_to_copy: output
                .global_references()
                .into_iter()
                .map(|a| RENodeId::Global(a))
                .collect(),
            nodes_to_move: output.node_ids().into_iter().collect(),
        };

        Ok((output, update))
    }
}
