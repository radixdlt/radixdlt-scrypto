use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::string::ToString;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::{ScryptoActor, TypeName};
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::validation::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame {
    /// The frame id
    pub depth: usize,
    /// The running actor of this frame
    pub actor: REActor,

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    pub node_refs: HashMap<RENodeId, RENodePointer>, // TODO: reduce fields visibility

    /// Owned Values
    pub owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
    pub auth_zone: Option<AuthZone>,
}

impl CallFrame {
    pub fn new_root<'s, W, I, C, Y>(
        signer_public_keys: Vec<EcdsaPublicKey>,
        is_system: bool,
        system_api: &mut Y,
    ) -> Self
    where
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
        Y: SystemApi<'s, W, I, C>,
    {
        // TODO: Cleanup initialization of authzone
        let signer_non_fungible_ids: BTreeSet<NonFungibleId> = signer_public_keys
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleId::from_bytes(public_key.to_vec()))
            .collect();

        let mut initial_auth_zone_proofs = Vec::new();
        if !signer_non_fungible_ids.is_empty() {
            // Proofs can't be zero amount
            let mut ecdsa_bucket = Bucket::new(ResourceContainer::new_non_fungible(
                ECDSA_TOKEN,
                signer_non_fungible_ids,
            ));
            let ecdsa_proof = ecdsa_bucket.create_proof(ECDSA_TOKEN_BUCKET_ID).unwrap();
            initial_auth_zone_proofs.push(ecdsa_proof);
        }

        if is_system {
            let id = [NonFungibleId::from_u32(0)].into_iter().collect();
            let mut system_bucket =
                Bucket::new(ResourceContainer::new_non_fungible(SYSTEM_TOKEN, id));
            let system_proof = system_bucket
                .create_proof(system_api.id_allocator().new_bucket_id().unwrap())
                .unwrap();
            initial_auth_zone_proofs.push(system_proof);
        }

        Self::new(
            0,
            REActor::Native,
            Some(AuthZone::new_with_proofs(initial_auth_zone_proofs)),
            HashMap::new(),
            HashMap::new(),
        )
    }

    pub fn new(
        depth: usize,
        actor: REActor,
        auth_zone: Option<AuthZone>,
        owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
        node_refs: HashMap<RENodeId, RENodePointer>,
    ) -> Self {
        Self {
            depth,
            actor,
            node_refs,
            owned_heap_nodes,
            auth_zone,
        }
    }

    pub fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
        let values = self
            .owned_heap_nodes
            .drain()
            .map(|(_id, value)| value)
            .collect();
        HeapRENode::drop_nodes(values).map_err(|e| RuntimeError::DropFailure(e))
    }

    pub fn run<'s, W, I, C, Y>(
        &mut self,
        execution_entity: ExecutionEntity,
        fn_ident: &str,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError>
    where
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
        Y: SystemApi<'s, W, I, C>,
    {
        let output = {
            let rtn = match execution_entity {
                ExecutionEntity::Function(type_name) => match type_name {
                    TypeName::TransactionProcessor => TransactionProcessor::static_main(
                        fn_ident, input, system_api,
                    )
                    .map_err(|e| match e {
                        TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                        TransactionProcessorError::InvalidMethod => panic!("Illegal state"),
                        TransactionProcessorError::RuntimeError(e) => e,
                    }),
                    TypeName::Package => ValidatedPackage::static_main(fn_ident, input, system_api)
                        .map_err(RuntimeError::PackageError),
                    TypeName::ResourceManager => {
                        ResourceManager::static_main(fn_ident, input, system_api)
                            .map_err(RuntimeError::ResourceManagerError)
                    }
                    TypeName::Blueprint(package_address, blueprint_name) => {
                        let output = {
                            let package = system_api
                                .track()
                                .read_substate(SubstateId::Package(package_address))
                                .package()
                                .clone(); // TODO: remove copy
                            let wasm_metering_params =
                                system_api.fee_table().wasm_metering_params();
                            let instrumented_code = system_api
                                .wasm_instrumenter()
                                .instrument(package.code(), &wasm_metering_params)
                                .to_vec(); // TODO: remove copy
                            let mut instance =
                                system_api.wasm_engine().instantiate(&instrumented_code);
                            let blueprint_abi = package
                                .blueprint_abi(&blueprint_name)
                                .expect("Blueprint should exist");
                            let export_name = &blueprint_abi
                                .get_fn_abi(fn_ident)
                                .unwrap()
                                .export_name
                                .to_string();
                            let mut runtime: Box<dyn WasmRuntime> =
                                Box::new(RadixEngineWasmRuntime::new(
                                    ScryptoActor::blueprint(
                                        package_address,
                                        blueprint_name.clone(),
                                    ),
                                    system_api,
                                ));
                            instance
                                .invoke_export(&export_name, &input, &mut runtime)
                                .map_err(|e| match e {
                                    // Flatten error code for more readable transaction receipt
                                    InvokeError::RuntimeError(e) => e,
                                    e @ _ => RuntimeError::InvokeError(e.into()),
                                })?
                        };

                        let package = system_api
                            .track()
                            .read_substate(SubstateId::Package(package_address))
                            .package();
                        let blueprint_abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint should exist");
                        let fn_abi = blueprint_abi.get_fn_abi(fn_ident).unwrap();
                        if !fn_abi.output.matches(&output.dom) {
                            Err(RuntimeError::InvalidFnOutput {
                                fn_ident: fn_ident.to_string(),
                                output: output.dom,
                            })
                        } else {
                            Ok(output)
                        }
                    }
                },
                ExecutionEntity::Method(_, state) => match state {
                    ExecutionState::Consumed(node_id) => match node_id {
                        RENodeId::Bucket(..) => {
                            Bucket::consuming_main(node_id, fn_ident, input, system_api)
                                .map_err(RuntimeError::BucketError)
                        }
                        RENodeId::Proof(..) => {
                            Proof::main_consume(node_id, fn_ident, input, system_api)
                                .map_err(RuntimeError::ProofError)
                        }
                        _ => panic!("Unexpected"),
                    },
                    ExecutionState::RENodeRef(node_id) => match node_id {
                        RENodeId::Bucket(bucket_id) => {
                            Bucket::main(bucket_id, fn_ident, input, system_api)
                                .map_err(RuntimeError::BucketError)
                        }
                        RENodeId::Proof(proof_id) => {
                            Proof::main(proof_id, fn_ident, input, system_api)
                                .map_err(RuntimeError::ProofError)
                        }
                        RENodeId::Worktop => Worktop::main(fn_ident, input, system_api)
                            .map_err(RuntimeError::WorktopError),
                        RENodeId::AuthZone => AuthZone::main(fn_ident, input, system_api)
                            .map_err(RuntimeError::AuthZoneError),
                        RENodeId::Vault(vault_id) => {
                            Vault::main(vault_id, fn_ident, input, system_api)
                                .map_err(RuntimeError::VaultError)
                        }
                        RENodeId::Component(component_address) => {
                            Component::main(component_address, fn_ident, input, system_api)
                                .map_err(RuntimeError::ComponentError)
                        }
                        RENodeId::ResourceManager(resource_address) => {
                            ResourceManager::main(resource_address, fn_ident, input, system_api)
                                .map_err(RuntimeError::ResourceManagerError)
                        }
                        RENodeId::System => System::main(fn_ident, input, system_api)
                            .map_err(RuntimeError::SystemError),
                        _ => panic!("Unexpected"),
                    },
                    ExecutionState::Component(
                        package_address,
                        blueprint_name,
                        component_address,
                    ) => {
                        let output = {
                            let package = system_api
                                .track()
                                .read_substate(SubstateId::Package(package_address))
                                .package()
                                .clone(); // TODO: remove copy
                            let wasm_metering_params =
                                system_api.fee_table().wasm_metering_params();
                            let instrumented_code = system_api
                                .wasm_instrumenter()
                                .instrument(package.code(), &wasm_metering_params)
                                .to_vec(); // TODO: remove copy
                            let mut instance =
                                system_api.wasm_engine().instantiate(&instrumented_code);
                            let blueprint_abi = package
                                .blueprint_abi(&blueprint_name)
                                .expect("Blueprint should exist");
                            let export_name = &blueprint_abi
                                .get_fn_abi(fn_ident)
                                .unwrap()
                                .export_name
                                .to_string();
                            let mut runtime: Box<dyn WasmRuntime> =
                                Box::new(RadixEngineWasmRuntime::new(
                                    ScryptoActor::Component(
                                        component_address,
                                        package_address.clone(),
                                        blueprint_name.clone(),
                                    ),
                                    system_api,
                                ));
                            instance
                                .invoke_export(&export_name, &input, &mut runtime)
                                .map_err(|e| match e {
                                    // Flatten error code for more readable transaction receipt
                                    InvokeError::RuntimeError(e) => e,
                                    e @ _ => RuntimeError::InvokeError(e.into()),
                                })?
                        };

                        let package = system_api
                            .track()
                            .read_substate(SubstateId::Package(package_address))
                            .package();
                        let blueprint_abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint should exist");
                        let fn_abi = blueprint_abi.get_fn_abi(fn_ident).unwrap();
                        if !fn_abi.output.matches(&output.dom) {
                            Err(RuntimeError::InvalidFnOutput {
                                fn_ident: fn_ident.to_string(),
                                output: output.dom,
                            })
                        } else {
                            Ok(output)
                        }
                    }
                },
            }?;

            rtn
        };

        // Take values to return
        let values_to_take = output.node_ids();
        let (taken_values, mut missing) = self.take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_value {
            return Err(RuntimeError::RENodeNotFound(missing_node));
        }

        // Check we have valid references to pass back
        for refed_component_address in &output.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            if let Some(RENodePointer::Store(..)) = self.node_refs.get(&node_id) {
                // Only allow passing back global references
            } else {
                return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
            }
        }

        // drop proofs and check resource leak
        if self.auth_zone.is_some() {
            system_api.invoke_method(
                Receiver::AuthZoneRef,
                "clear".to_string(),
                ScryptoValue::from_typed(&AuthZoneClearInput {}),
            )?;
        }
        self.drop_owned_values()?;

        Ok((output, taken_values))
    }

    pub fn take_available_values(
        &mut self,
        node_ids: HashSet<RENodeId>,
        persist_only: bool,
    ) -> Result<(HashMap<RENodeId, HeapRootRENode>, HashSet<RENodeId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in node_ids {
                let maybe = self.owned_heap_nodes.remove(&id);
                if let Some(value) = maybe {
                    value.root().verify_can_move()?;
                    if persist_only {
                        value.root().verify_can_persist()?;
                    }
                    taken_values.insert(id, value);
                } else {
                    missing_values.insert(id);
                }
            }

            (taken_values, missing_values)
        };

        // Moved values must have their references removed
        for (id, value) in &taken {
            self.node_refs.remove(id);
            for (id, ..) in &value.child_nodes {
                self.node_refs.remove(id);
            }
        }

        Ok((taken, missing))
    }
}
