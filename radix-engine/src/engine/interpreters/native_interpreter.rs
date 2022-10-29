use transaction::model::Instruction;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};

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

pub trait NativeFunctionActor<I, O, E> {
    fn run<'s, Y, R>(input: I, system_api: &mut Y) -> Result<O, InvokeError<E>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve;
}

pub struct NativeFunctionExecutor(pub NativeFunction, pub ScryptoValue);

impl Executor<ScryptoValue> for NativeFunctionExecutor {
    fn args(&self) -> &ScryptoValue {
        &self.1
    }

    fn execute<'s, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation, ScryptoValue>
            + Invokable<NativeFunctionInvocation, ScryptoValue>
            + Invokable<EpochManagerCreateInput, ScryptoValue>
            + Invokable<NativeMethodInvocation, ScryptoValue>,
        R: FeeReserve,
    {
        let output = match self.0 {
            NativeFunction::TransactionProcessor(func) => {
                TransactionProcessor::static_main(func, self.1, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            NativeFunction::Package(func) => Package::static_main(func, self.1, system_api)
                .map_err::<RuntimeError, _>(|e| e.into()),
            NativeFunction::ResourceManager(func) => {
                ResourceManager::static_main(func, self.1, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            NativeFunction::EpochManager(func) => match func {
                EpochManagerFunction::Create => {
                    let input: EpochManagerCreateInput =
                        scrypto_decode(&self.1.raw).map_err(|_| {
                            RuntimeError::InterpreterError(
                                InterpreterError::InvalidNativeFunctionInput,
                            )
                        })?;
                    Self::run(input, system_api)
                        .map(|rtn| ScryptoValue::from_typed(&rtn))
                        .map_err::<RuntimeError, _>(|e| e.into())
                }
            },
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


pub struct EpochManagerCreateExecutor(EpochManagerCreateInput, ScryptoValue);
impl Executor<ScryptoValue> for EpochManagerCreateExecutor {
    fn args(&self) -> &ScryptoValue {
        &self.1
    }

    fn execute<'s, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation, ScryptoValue>
            + Invokable<NativeFunctionInvocation, ScryptoValue>
            + Invokable<NativeMethodInvocation, ScryptoValue>,
            R: FeeReserve,
    {

        let node_id =
            system_api.create_node(RENode::EpochManager(EpochManagerSubstate { epoch: 0 }))?;

        let global_node_id = system_api.create_node(RENode::Global(
            GlobalAddressSubstate::System(node_id.into()),
        ))?;

        let system_address: SystemAddress = global_node_id.into();
        let output = ScryptoValue::from_typed(&system_address);

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

impl<'g, 's, W, I, R>
InvocationResolver<EpochManagerCreateInput, EpochManagerCreateExecutor, ScryptoValue>
for Kernel<'g, 's, W, I, R>
    where
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
{
    fn resolve(
        &mut self,
        invocation: EpochManagerCreateInput,
    ) -> Result<(EpochManagerCreateExecutor, REActor, CallFrameUpdate), RuntimeError> {
        let input = ScryptoValue::from_typed(&invocation);
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::EpochManager(EpochManagerFunction::Create)));
        Ok((
            EpochManagerCreateExecutor(invocation, input),
            actor,
            CallFrameUpdate::empty(),
        ))
    }
}


impl<'g, 's, W, I, R>
InvocationResolver<NativeFunctionInvocation, NativeFunctionExecutor, ScryptoValue>
for Kernel<'g, 's, W, I, R>
    where
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
{
    fn resolve(
        &mut self,
        native_function: NativeFunctionInvocation,
    ) -> Result<(NativeFunctionExecutor, REActor, CallFrameUpdate), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();
        let actor = REActor::Function(ResolvedFunction::Native(native_function.0));
        for global_address in native_function.args().global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        // TODO: This can be refactored out once any type in sbor is implemented
        let maybe_txn: Result<TransactionProcessorRunInput, DecodeError> =
            scrypto_decode(&native_function.args().raw);
        if let Ok(input) = maybe_txn {
            for instruction in input.instructions.as_ref() {
                match instruction {
                    Instruction::CallFunction { args, .. }
                    | Instruction::CallMethod { args, .. }
                    | Instruction::CallNativeFunction { args, .. }
                    | Instruction::CallNativeMethod { args, .. } => {
                        let scrypto_value =
                            ScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                        for global_address in scrypto_value.global_references() {
                            node_refs_to_copy.insert(RENodeId::Global(global_address));
                        }
                    }
                    _ => {}
                }
            }
        }

        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));

        Ok((
            NativeFunctionExecutor(native_function.0, native_function.1.clone()),
            actor,
            CallFrameUpdate {
                nodes_to_move: native_function.args().node_ids().into_iter().collect(),
                node_refs_to_copy,
            },
        ))
    }
}


pub struct NativeMethodExecutor(pub NativeMethod, pub ResolvedReceiver, pub ScryptoValue);

impl Executor<ScryptoValue> for NativeMethodExecutor {
    fn args(&self) -> &ScryptoValue {
        &self.2
    }

    fn execute<'s, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation, ScryptoValue>
            + Invokable<NativeFunctionInvocation, ScryptoValue>
            + Invokable<NativeMethodInvocation, ScryptoValue>,
        R: FeeReserve,
    {
        let output = match (self.1.receiver, self.0) {
            (RENodeId::AuthZoneStack(auth_zone_id), NativeMethod::AuthZone(method)) => {
                AuthZoneStack::main(auth_zone_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Bucket(bucket_id), NativeMethod::Bucket(method)) => {
                Bucket::main(bucket_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Proof(proof_id), NativeMethod::Proof(method)) => {
                Proof::main(proof_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Worktop, NativeMethod::Worktop(method)) => {
                Worktop::main(method, self.2, system_api).map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Vault(vault_id), NativeMethod::Vault(method)) => {
                Vault::main(vault_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Component(component_id), NativeMethod::Component(method)) => {
                Component::main(component_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (
                RENodeId::ResourceManager(resource_address),
                NativeMethod::ResourceManager(method),
            ) => ResourceManager::main(resource_address, method, self.2, system_api)
                .map_err::<RuntimeError, _>(|e| e.into()),
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
