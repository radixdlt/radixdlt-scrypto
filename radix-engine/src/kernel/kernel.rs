use crate::blueprints::account::AccountSubstate;
use crate::blueprints::identity::Identity;
use crate::blueprints::kv_store::KeyValueStore;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelSubstateApi, LockFlags};
use crate::kernel::module::BaseModule;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate; // TODO: possible clean-up
use crate::system::kernel_modules::auth::auth_module::AuthModule;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::kernel_modules::logger::LoggerModule;
use crate::system::kernel_modules::node_move::NodeMoveModule;
use crate::system::kernel_modules::transaction_runtime::TransactionHashModule;
use crate::system::node_modules::auth::{AccessRulesChainSubstate, AuthZoneStackSubstate}; // TODO: possible clean-up
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::SysBucket;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, GlobalAddress, GlobalOffset, LockHandle, ProofOffset, RENodeId,
    SubstateId, SubstateOffset, WorktopOffset,
};
use radix_engine_interface::blueprints::resource::{
    require, AccessRule, AccessRuleKey, AccessRules, Bucket,
};
use radix_engine_interface::rule;
use sbor::rust::mem;
use transaction::model::AuthZoneParams;

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
    R,  // Fee reserve type
    M,
> where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    /// Current execution mode, specifies permissions into state/invocations
    pub(super) execution_mode: ExecutionMode,
    /// Stack
    pub(super) current_frame: CallFrame,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    pub(super) prev_frame_stack: Vec<CallFrame>,
    /// Heap
    pub(super) heap: Heap,
    /// Store
    pub(super) track: &'g mut Track<'s, R>,

    /// ID allocator
    pub(super) id_allocator: &'g mut IdAllocator,
    /// Interpreter capable of running scrypto programs
    pub(super) scrypto_interpreter: &'g ScryptoInterpreter<W>,
    /// Kernel module
    pub(super) module: &'g mut M,
}

impl<'g, 's, W, R, M> Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    pub fn new(
        auth_zone_params: AuthZoneParams,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s, R>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: &'g mut M,
    ) -> Self {
        let mut kernel = Self {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            module,
        };

        // Initial authzone
        // TODO: Move into module initialization
        kernel
            .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AuthModule, |api| {
                let auth_zone = AuthZoneStackSubstate::new(
                    vec![],
                    auth_zone_params.virtualizable_proofs_resource_addresses,
                    auth_zone_params.initial_proofs.into_iter().collect(),
                );
                let node_id = api.allocate_node_id(RENodeType::AuthZoneStack)?;
                api.create_node(
                    node_id,
                    RENodeInit::AuthZoneStack(auth_zone),
                    BTreeMap::new(),
                )?;
                Ok(())
            })
            .expect("AuthModule failed to initialize");

        kernel
            .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::LoggerModule, |api| {
                LoggerModule::initialize(api)
            })
            .expect("Logger failed to initialize");

        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(SYSTEM_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(ECDSA_SECP256K1_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(EDDSA_ED25519_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Component(CLOCK)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Package(FAUCET_PACKAGE)),
            RENodeVisibilityOrigin::Normal,
        );

        kernel
    }

    fn create_virtual_account(
        &mut self,
        node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        // TODO: Replace with trusted IndexedScryptoValue
        let access_rule = rule!(require(non_fungible_global_id));
        let component_id = {
            let kv_store_id = {
                let node_id = self.allocate_node_id(RENodeType::KeyValueStore)?;
                let node = RENodeInit::KeyValueStore(KeyValueStore::new());
                self.create_node(node_id, node, BTreeMap::new())?;
                node_id
            };

            let access_rules = {
                let mut access_rules = AccessRules::new();
                access_rules.set_access_rule_and_mutability(
                    AccessRuleKey::Native(NativeFn::Account(AccountFn::Balance)),
                    AccessRule::AllowAll,
                    AccessRule::DenyAll,
                );
                access_rules.set_access_rule_and_mutability(
                    AccessRuleKey::Native(NativeFn::Account(AccountFn::Deposit)),
                    AccessRule::AllowAll,
                    AccessRule::DenyAll,
                );
                access_rules.set_access_rule_and_mutability(
                    AccessRuleKey::Native(NativeFn::Account(AccountFn::DepositBatch)),
                    AccessRule::AllowAll,
                    AccessRule::DenyAll,
                );
                access_rules.default(access_rule.clone(), access_rule)
            };

            let node_id = {
                let mut node_modules = BTreeMap::new();
                node_modules.insert(
                    NodeModuleId::Metadata,
                    RENodeModuleInit::Metadata(MetadataSubstate {
                        metadata: BTreeMap::new(),
                    }),
                );
                let access_rules_substate = AccessRulesChainSubstate {
                    access_rules_chain: vec![access_rules],
                };
                node_modules.insert(
                    NodeModuleId::AccessRules,
                    RENodeModuleInit::AccessRulesChain(access_rules_substate),
                );
                let account_substate = AccountSubstate {
                    vaults: Own::KeyValueStore(kv_store_id.into()),
                };

                let node_id = self.allocate_node_id(RENodeType::Account)?;
                let node = RENodeInit::Account(account_substate);
                self.create_node(node_id, node, node_modules)?;
                node_id
            };
            node_id
        };

        // TODO: Use system_api to globalize component when create_node is refactored
        // TODO: to allow for address selection
        let global_substate = GlobalAddressSubstate::Account(component_id.into());

        self.current_frame.create_node(
            node_id,
            RENodeInit::Global(global_substate),
            BTreeMap::new(),
            &mut self.heap,
            &mut self.track,
            true,
        )?;

        Ok(())
    }

    fn create_virtual_identity(
        &mut self,
        node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        let access_rule = rule!(require(non_fungible_global_id));
        let underlying_node_id = Identity::create(access_rule, self)?;

        // TODO: Use system_api to globalize component when create_node is refactored
        // TODO: to allow for address selection
        let global_substate = GlobalAddressSubstate::Identity(underlying_node_id.into());
        self.current_frame.create_node(
            node_id,
            RENodeInit::Global(global_substate),
            BTreeMap::new(),
            &mut self.heap,
            &mut self.track,
            true,
        )?;

        Ok(())
    }

    pub(super) fn try_virtualize(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<bool, RuntimeError> {
        match (node_id, offset) {
            (
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ) => {
                // Lazy create component if missing
                match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualAccount(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualIdentity(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    _ => return Ok(false),
                };

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn drop_node_internal(
        &mut self,
        node_id: RENodeId,
    ) -> Result<HeapRENode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::DropNode, |system_api| {
            match node_id {
                RENodeId::AuthZoneStack => {
                    let handle = system_api.lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                    let auth_zone_stack = substate_ref_mut.auth_zone_stack();
                    auth_zone_stack.clear_all();
                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Proof(..) => {
                    let handle = system_api.lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::Proof(ProofOffset::Proof),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                    let proof = substate_ref_mut.proof();
                    proof.drop();
                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Logger => Ok(()),
                RENodeId::Worktop => {
                    let handle = system_api.lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::Worktop(WorktopOffset::Worktop),
                        LockFlags::MUTABLE,
                    )?;

                    let buckets = {
                        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                        let worktop = substate_ref_mut.worktop();
                        mem::replace(&mut worktop.resources, BTreeMap::new())
                    };
                    for (_, bucket) in buckets {
                        let bucket = Bucket(bucket.bucket_id());
                        if !bucket.sys_is_empty(system_api)? {
                            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                                RENodeId::Worktop,
                            )));
                        }
                    }

                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Bucket(..) => Ok(()),
                RENodeId::TransactionRuntime(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                    node_id,
                ))),
            }
        })?;

        let node = self.current_frame.remove_node(&mut self.heap, node_id)?;
        for (_, substate) in &node.substates {
            let (_, child_nodes) = substate.to_ref().references_and_owned_nodes();
            for child_node in child_nodes {
                // Need to go through system_api so that visibility issues can be caught
                self.drop_node(child_node)?;
            }
        }
        // TODO: REmove
        Ok(node)
    }

    fn drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let mut worktops = Vec::new();
        let owned_nodes = self.current_frame.owned_nodes();

        // Need to go through system_api so that visibility issues can be caught
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Application, |system_api| {
            for node_id in owned_nodes {
                if let RENodeId::Worktop = node_id {
                    worktops.push(node_id);
                } else {
                    system_api.drop_node(node_id)?;
                }
            }
            for worktop_id in worktops {
                system_api.drop_node(worktop_id)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        executor: X,
        actor: ResolvedActor,
        mut call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let derefed_lock = if let Some(ResolvedReceiver {
            derefed_from: Some((_, derefed_lock)),
            ..
        }) = &actor.receiver
        {
            Some(*derefed_lock)
        } else {
            None
        };

        // Filter
        self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
            AuthModule::on_before_frame_start(&actor, system_api)
        })?;

        // New Call Frame pre-processing
        {
            // TODO: Abstract these away
            self.execute_in_mode(ExecutionMode::TransactionModule, |system_api| {
                TransactionHashModule::on_call_frame_enter(
                    &mut call_frame_update,
                    &actor,
                    system_api,
                )
            })?;
            self.execute_in_mode(ExecutionMode::LoggerModule, |system_api| {
                LoggerModule::on_call_frame_enter(&mut call_frame_update, &actor, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
                AuthModule::on_call_frame_enter(&mut call_frame_update, &actor, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::NodeMoveModule, |system_api| {
                NodeMoveModule::on_call_frame_enter(
                    &mut call_frame_update,
                    &actor.identifier,
                    system_api,
                )
            })?;

            self.module
                .pre_execute_invocation(
                    &actor,
                    &call_frame_update,
                    &mut self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                )
                .map_err(RuntimeError::ModuleError)?;
            self.id_allocator.pre_execute_invocation();
        }

        // Call Frame Push
        {
            let frame = CallFrame::new_child_from_parent(
                &mut self.current_frame,
                actor,
                call_frame_update,
            )?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, update) = self.execute_in_mode(ExecutionMode::Application, |system_api| {
            executor.execute(system_api)
        })?;

        // Call Frame post-processing
        {
            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            self.id_allocator.post_execute_invocation()?;
            self.module
                .post_execute_invocation(
                    &self.prev_frame_stack.last().unwrap().actor,
                    &update,
                    &mut self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                )
                .map_err(RuntimeError::ModuleError)?;

            // TODO: Abstract these away
            self.execute_in_mode(ExecutionMode::NodeMoveModule, |system_api| {
                NodeMoveModule::on_call_frame_exit(&update, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
                AuthModule::on_call_frame_exit(system_api)
            })?;

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;
        }

        // Call Frame Pop
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();
            CallFrame::update_upstream(&mut self.current_frame, &mut parent, update)?;

            // drop proofs and check resource leak
            self.drop_nodes_in_frame()?;

            // Restore previous frame
            self.current_frame = parent;
        }

        if let Some(derefed_lock) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, derefed_lock)?;
        }

        Ok(output)
    }

    pub fn node_method_deref(
        &mut self,
        node_id: RENodeId,
    ) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            let derefed =
                self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Deref, |system_api| {
                    let offset = SubstateOffset::Global(GlobalOffset::Global);
                    let handle = system_api.lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        offset,
                        LockFlags::empty(),
                    )?;
                    let substate_ref = system_api.get_ref(handle)?;
                    Ok((substate_ref.global_address().node_deref(), handle))
                })?;

            Ok(Some(derefed))
        } else {
            Ok(None)
        }
    }

    pub fn node_offset_deref(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            if !matches!(offset, SubstateOffset::Global(GlobalOffset::Global)) {
                let derefed = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::Deref,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            node_id,
                            NodeModuleId::SELF,
                            SubstateOffset::Global(GlobalOffset::Global),
                            LockFlags::empty(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        Ok((substate_ref.global_address().node_deref(), handle))
                    },
                )?;

                Ok(Some(derefed))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            (ExecutionMode::Resolver, ExecutionMode::Deref) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    pub(super) fn invoke_internal<X: Executor>(
        &mut self,
        executor: X,
        actor: ResolvedActor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let depth = self.current_frame.depth;
        // TODO: Move to higher layer
        if depth == 0 {
            for node_id in &call_frame_update.node_refs_to_copy {
                match node_id {
                    RENodeId::Global(global_address) => {
                        if self.current_frame.get_node_location(*node_id).is_err() {
                            if matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EddsaEd25519VirtualAccount(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EddsaEd25519VirtualIdentity(..)
                                )
                            ) {
                                self.current_frame
                                    .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                continue;
                            }

                            let offset = SubstateOffset::Global(GlobalOffset::Global);

                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    RENodeId::Vault(..) => {
                        if self.current_frame.get_node_location(*node_id).is_err() {
                            let offset = SubstateOffset::Vault(VaultOffset::Vault);
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;

                            self.current_frame
                                .add_stored_ref(*node_id, RENodeVisibilityOrigin::DirectAccess);
                        }
                    }
                    _ => {}
                }
            }
        }

        let output = self.run(executor, actor, call_frame_update)?;

        // TODO: Move to higher layer
        if depth == 0 {
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;
            self.drop_nodes_in_frame()?;
        }

        Ok(output)
    }

    pub(super) fn execute_in_mode<X, RTN, E>(
        &mut self,
        execution_mode: ExecutionMode,
        execute: X,
    ) -> Result<RTN, RuntimeError>
    where
        RuntimeError: From<E>,
        X: FnOnce(&mut Self) -> Result<RTN, E>,
    {
        Self::verify_valid_mode_transition(&self.execution_mode, &execution_mode)?;

        // Save and replace kernel actor
        let saved = self.execution_mode;
        self.execution_mode = execution_mode;

        let rtn = execute(self)?;

        // Restore old kernel actor
        self.execution_mode = saved;

        Ok(rtn)
    }
}
