use scrypto::core::{FnIdentifier, NativeFnIdentifier, Receiver};
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

pub struct NativeInterpreter;

impl NativeInterpreter {
    pub fn run<'s, Y, W, I, C>(
        receiver: Option<Receiver>,
        auth_zone_frame_id: Option<usize>,
        fn_identifier: NativeFnIdentifier,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    {
        match (receiver, fn_identifier) {
            (None, NativeFnIdentifier::TransactionProcessor(transaction_processor_fn)) => {
                TransactionProcessor::static_main(transaction_processor_fn, input, system_api)
                    .map_err(|e| match e {
                        TransactionProcessorError::InvalidRequestData(_) => {
                            panic!("Illegal state")
                        }
                        TransactionProcessorError::InvalidMethod => {
                            panic!("Illegal state")
                        }
                        TransactionProcessorError::RuntimeError(e) => e,
                    })
            }
            (None, NativeFnIdentifier::Package(package_fn)) => {
                ValidatedPackage::static_main(package_fn, input, system_api)
                    .map_err(RuntimeError::PackageError)
            }
            (None, NativeFnIdentifier::ResourceManager(resource_manager_fn)) => {
                ResourceManager::static_main(resource_manager_fn, input, system_api)
                    .map_err(RuntimeError::ResourceManagerError)
            }
            (Some(Receiver::Consumed(node_id)), NativeFnIdentifier::Bucket(bucket_fn)) => {
                Bucket::consuming_main(node_id, bucket_fn, input, system_api)
                    .map_err(RuntimeError::BucketError)
            }
            (Some(Receiver::Consumed(node_id)), NativeFnIdentifier::Proof(proof_fn)) => {
                Proof::main_consume(node_id, proof_fn, input, system_api)
                    .map_err(RuntimeError::ProofError)
            }
            (Some(Receiver::CurrentAuthZone), NativeFnIdentifier::AuthZone(auth_zone_fn)) => {
                AuthZone::main(
                    auth_zone_frame_id.expect("AuthZone receiver frame id not specified"),
                    auth_zone_fn,
                    input,
                    system_api,
                )
                .map_err(RuntimeError::AuthZoneError)
            }
            (
                Some(Receiver::Ref(RENodeId::Bucket(bucket_id))),
                NativeFnIdentifier::Bucket(bucket_fn),
            ) => Bucket::main(bucket_id, bucket_fn, input, system_api)
                .map_err(RuntimeError::BucketError),
            (
                Some(Receiver::Ref(RENodeId::Proof(proof_id))),
                NativeFnIdentifier::Proof(proof_fn),
            ) => {
                Proof::main(proof_id, proof_fn, input, system_api).map_err(RuntimeError::ProofError)
            }
            (Some(Receiver::Ref(RENodeId::Worktop)), NativeFnIdentifier::Worktop(worktop_fn)) => {
                Worktop::main(worktop_fn, input, system_api).map_err(RuntimeError::WorktopError)
            }
            (
                Some(Receiver::Ref(RENodeId::Vault(vault_id))),
                NativeFnIdentifier::Vault(vault_fn),
            ) => {
                Vault::main(vault_id, vault_fn, input, system_api).map_err(RuntimeError::VaultError)
            }
            (
                Some(Receiver::Ref(RENodeId::Component(component_address))),
                NativeFnIdentifier::Component(component_fn),
            ) => Component::main(component_address, component_fn, input, system_api)
                .map_err(RuntimeError::ComponentError),
            (
                Some(Receiver::Ref(RENodeId::ResourceManager(resource_address))),
                NativeFnIdentifier::ResourceManager(resource_manager_fn),
            ) => ResourceManager::main(resource_address, resource_manager_fn, input, system_api)
                .map_err(RuntimeError::ResourceManagerError),
            (Some(Receiver::Ref(RENodeId::System)), NativeFnIdentifier::System(system_fn)) => {
                System::main(system_fn, input, system_api).map_err(RuntimeError::SystemError)
            }
            _ => {
                return Err(RuntimeError::MethodDoesNotExist(FnIdentifier::Native(
                    fn_identifier.clone(),
                )))
            }
        }
    }
}
