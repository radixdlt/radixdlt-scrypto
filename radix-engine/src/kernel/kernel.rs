use crate::blueprints::account::AccountSubstate;
use crate::blueprints::auth_zone::AuthZoneStackSubstate;
use crate::blueprints::identity::Identity;
use crate::blueprints::resource::{
    BucketInfoSubstate, FungibleProof, NonFungibleProof, ProofInfoSubstate, WorktopSubstate,
};
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::system::kernel_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_properties::VisibilityProperties;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::access_rules::AccessRulesObject;
use native_sdk::metadata::Metadata;
use native_sdk::resource::SysProof;
use radix_engine_interface::address::*;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::node_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use radix_engine_interface::api::package::{PackageCodeSubstate, PACKAGE_LOADER_BLUEPRINT};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientObjectApi;
// TODO: clean this up!
use crate::kernel::kernel_api::TemporaryResolvedInvocation;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, LockHandle, ProofOffset, RENodeId, SubstateId, SubstateOffset,
    WorktopOffset,
};
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_BLUEPRINT;
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_DEPOSIT_BATCH_IDENT, ACCOUNT_DEPOSIT_IDENT,
};
use radix_engine_interface::blueprints::auth_zone::AUTH_ZONE_BLUEPRINT;
use radix_engine_interface::blueprints::clock::CLOCK_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::{
    EPOCH_MANAGER_BLUEPRINT, VALIDATOR_BLUEPRINT,
};
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::blueprints::logger::LOGGER_BLUEPRINT;
use radix_engine_interface::blueprints::resource::{
    require, AccessRule, AccessRules, LiquidFungibleResource, LiquidNonFungibleResource, MethodKey,
    ResourceType, BUCKET_BLUEPRINT, PROOF_BLUEPRINT, RESOURCE_MANAGER_BLUEPRINT, VAULT_BLUEPRINT,
    WORKTOP_BLUEPRINT,
};
use radix_engine_interface::blueprints::transaction_runtime::TRANSACTION_RUNTIME_BLUEPRINT;
use radix_engine_interface::rule;
use sbor::rust::mem;

use super::actor::{Actor, ExecutionMode};
use super::call_frame::{CallFrame, RENodeVisibilityOrigin};
use super::heap::{Heap, HeapRENode};
use super::id_allocator::IdAllocator;
use super::interpreters::ScryptoInterpreter;
use super::kernel_api::{
    ExecutableInvocation, Executor, Invokable, KernelApi, KernelInternalApi, KernelModuleApi,
    KernelNodeApi, KernelSubstateApi, KernelWasmApi, LockInfo,
};
use super::module::KernelModule;
use super::module_mixer::KernelModuleMixer;
use super::track::{Track, TrackError};

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
> where
    W: WasmEngine,
{
    /// Current execution mode, specifies permissions into state/invocations
    execution_mode: ExecutionMode,
    /// Stack
    current_frame: CallFrame,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    prev_frame_stack: Vec<CallFrame>,
    /// Heap
    heap: Heap,
    /// Store
    track: &'g mut Track<'s>,

    /// ID allocator
    id_allocator: &'g mut IdAllocator,
    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g ScryptoInterpreter<W>,
    /// Kernel module mixer
    module: KernelModuleMixer,
}

impl<'g, 's, W> Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    pub fn new(
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: KernelModuleMixer,
    ) -> Self {
        Self {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            module,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
            KernelModuleMixer::on_init(api)
        })
    }

    // TODO: Josh holds some concern about this interface; will look into this again.
    pub fn teardown<T>(
        mut self,
        previous_result: Result<T, RuntimeError>,
    ) -> (KernelModuleMixer, Result<T, RuntimeError>) {
        let new_result = match previous_result {
            Ok(output) => {
                // Sanity check call frame
                assert!(self.prev_frame_stack.is_empty());

                // Tear down kernel modules
                match self
                    .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
                        KernelModuleMixer::on_teardown(api)
                    }) {
                    Ok(_) => Ok(output),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        };

        (self.module, new_result)
    }

    fn create_virtual_account(
        &mut self,
        global_node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {

        // TODO: This should move into the appropriate place once virtual manager is implemented
        self.current_frame.add_ref(
            RENodeId::GlobalResourceManager(ECDSA_SECP256K1_TOKEN),
            RENodeVisibilityOrigin::Normal,
        );
        self.current_frame.add_ref(
            RENodeId::GlobalResourceManager(EDDSA_ED25519_TOKEN),
            RENodeVisibilityOrigin::Normal,
        );

        let access_rule = rule!(require(non_fungible_global_id));
        let component_id = {
            let kv_store_id = {
                let node_id = self.kernel_allocate_node_id(RENodeType::KeyValueStore)?;
                let node = RENodeInit::KeyValueStore;
                self.kernel_create_node(node_id, node, BTreeMap::new())?;
                node_id
            };

            let node_id = {
                let node_modules = btreemap!(
                    NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: ACCOUNT_PACKAGE,
                        blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                        global: false
                    })
                );

                let account_substate = AccountSubstate {
                    vaults: Own::KeyValueStore(kv_store_id.into()),
                };

                let node_id = self.kernel_allocate_node_id(RENodeType::Account)?;
                let node = RENodeInit::Account(account_substate);
                self.kernel_create_node(node_id, node, node_modules)?;
                node_id
            };
            node_id
        };

        let access_rules = {
            let mut access_rules = AccessRules::new();
            access_rules.set_access_rule_and_mutability(
                MethodKey::new(NodeModuleId::SELF, ACCOUNT_DEPOSIT_IDENT.to_string()),
                AccessRule::AllowAll,
                AccessRule::DenyAll,
            );
            access_rules.set_access_rule_and_mutability(
                MethodKey::new(NodeModuleId::SELF, ACCOUNT_DEPOSIT_BATCH_IDENT.to_string()),
                AccessRule::AllowAll,
                AccessRule::DenyAll,
            );
            access_rules.default(access_rule.clone(), access_rule)
        };

        let access_rules = AccessRulesObject::sys_new(access_rules, self)?;

        let metadata = Metadata::sys_new(self)?;

        self.globalize_with_address(
            component_id,
            btreemap!(
                NodeModuleId::AccessRules => scrypto_encode(&access_rules).unwrap(),
                NodeModuleId::Metadata => scrypto_encode(&metadata).unwrap(),
            ),
            global_node_id.into(),
        )?;

        Ok(())
    }

    fn create_virtual_identity(
        &mut self,
        global_node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        // TODO: This should move into the appropriate place once virtual manager is implemented
        self.current_frame.add_ref(
            RENodeId::GlobalResourceManager(ECDSA_SECP256K1_TOKEN),
            RENodeVisibilityOrigin::Normal,
        );
        self.current_frame.add_ref(
            RENodeId::GlobalResourceManager(EDDSA_ED25519_TOKEN),
            RENodeVisibilityOrigin::Normal,
        );

        let access_rule = rule!(require(non_fungible_global_id));
        let (local_id, access_rules) = Identity::create(access_rule, self)?;

        let access_rules = AccessRulesObject::sys_new(access_rules, self)?;
        let metadata = Metadata::sys_new(self)?;

        self.globalize_with_address(
            local_id,
            btreemap!(
                NodeModuleId::AccessRules => scrypto_encode(&access_rules).unwrap(),
                NodeModuleId::Metadata => scrypto_encode(&metadata).unwrap(),
            ),
            global_node_id.into(),
        )?;

        Ok(())
    }

    fn try_virtualize(
        &mut self,
        node_id: RENodeId,
        _module_id: NodeModuleId,
        _offset: &SubstateOffset,
    ) -> Result<bool, RuntimeError> {
        match node_id {
            // TODO: Need to have a schema check in place before this in order to not create virtual components when accessing illegal substates
            RENodeId::GlobalComponent(component_address) => {
                // Lazy create component if missing
                match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualAccount(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualIdentity(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
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

    fn drop_node_internal(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::DropNode, |api| match node_id {
            RENodeId::AuthZoneStack => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                    LockFlags::MUTABLE,
                )?;
                let substate_ref: &AuthZoneStackSubstate = api.kernel_get_substate_ref(handle)?;
                let mut auth_zone_stack = substate_ref.clone();
                loop {
                    if let Some(mut auth_zone) = auth_zone_stack.pop_auth_zone() {
                        for p in auth_zone.drain() {
                            p.sys_drop(api)?;
                        }
                    } else {
                        break;
                    }
                }
                api.kernel_drop_lock(handle)?;
                api.current_frame.remove_node(&mut api.heap, node_id)
            }
            RENodeId::Proof(..) => {
                let proof_info = ProofInfoSubstate::of(node_id, api)?;
                if proof_info.resource_type.is_fungible() {
                    let handle = api.kernel_lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::Proof(ProofOffset::Fungible),
                        LockFlags::read_only(),
                    )?;
                    let proof: &FungibleProof = api.kernel_get_substate_ref(handle)?;
                    let proof = proof.clone();
                    proof.drop_proof(api)?;
                    api.kernel_drop_lock(handle)?;
                } else {
                    let handle = api.kernel_lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::Proof(ProofOffset::NonFungible),
                        LockFlags::read_only(),
                    )?;
                    let proof: &NonFungibleProof = api.kernel_get_substate_ref(handle)?;
                    let proof = proof.clone();
                    proof.drop_proof(api)?;
                    api.kernel_drop_lock(handle)?;
                }

                api.current_frame.remove_node(&mut api.heap, node_id)
            }
            RENodeId::Worktop => {
                let mut node = api.current_frame.remove_node(&mut api.heap, node_id)?;

                let substate = node
                    .substates
                    .remove(&(
                        NodeModuleId::SELF,
                        SubstateOffset::Worktop(WorktopOffset::Worktop),
                    ))
                    .unwrap();
                let worktop: WorktopSubstate = substate.into();
                if !worktop.resources.is_empty() {
                    return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                        RENodeId::Worktop,
                    )));
                }

                return Ok(node);
            }
            RENodeId::Logger
            | RENodeId::TransactionRuntime
            | RENodeId::Bucket(..)
            | RENodeId::Clock(..)
            | RENodeId::EpochManager(..)
            | RENodeId::Account(..)
            | RENodeId::Validator(..)
            | RENodeId::Component(..)
            | RENodeId::AccessController(..)
            | RENodeId::Identity(..) => api.current_frame.remove_node(&mut api.heap, node_id),
            _ => Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id,
            ))),
        })
    }

    fn kernel_drop_node_recursively(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        let node = self.kernel_drop_node(node_id)?;
        for (_, substate) in &node.substates {
            let (_, child_nodes) = substate.to_ref().references_and_owned_nodes();
            for child_node in child_nodes {
                self.kernel_drop_node_recursively(child_node)?;
            }
        }

        Ok(())
    }

    fn drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let mut worktops = Vec::new();
        let owned_nodes = self.current_frame.owned_nodes();

        // Need to go through api so that access rules can be caught
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AutoDrop, |api| {
            for node_id in owned_nodes {
                if let RENodeId::Worktop = node_id {
                    worktops.push(node_id);
                } else {
                    api.kernel_drop_node_recursively(node_id)?;
                }
            }
            for worktop_id in worktops {
                api.kernel_drop_node_recursively(worktop_id)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        resolved: TemporaryResolvedInvocation<X>,
    ) -> Result<X::Output, RuntimeError> {
        let executor = resolved.executor;
        let actor = resolved.resolved_actor;
        let args = resolved.args;
        let mut call_frame_update = resolved.update;

        let caller = self.current_frame.actor.clone();

        // Before push call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::before_push_frame(
                    api,
                    &Some(actor.clone()),
                    &mut call_frame_update,
                    &args,
                )
            })?;
        }

        // Push call frame
        {
            self.id_allocator.push();

            let frame = CallFrame::new_child_from_parent(
                &mut self.current_frame,
                actor.clone(),
                call_frame_update,
            )?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, update) = {
            // Handle execution start
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::on_execution_start(api, &caller)
            })?;

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            // Run
            let (output, mut update) =
                self.execute_in_mode(ExecutionMode::Client, |api| executor.execute(args, api))?;

            // Handle execution finish
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::on_execution_finish(api, &caller, &mut update)
            })?;

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            (output, update)
        };

        // Pop call frame
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();

            // Move resource
            CallFrame::update_upstream(&mut self.current_frame, &mut parent, update)?;

            // drop proofs and check resource leak
            self.drop_nodes_in_frame()?;

            // Restore previous frame
            self.current_frame = parent;

            self.id_allocator.pop()?;
        }

        // After pop call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::after_pop_frame(api)
            })?;
        }

        Ok(output)
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    fn invoke_internal<X: Executor>(
        &mut self,
        resolved: TemporaryResolvedInvocation<X>,
    ) -> Result<X::Output, RuntimeError> {
        let depth = self.current_frame.depth;
        // TODO: Move to higher layer
        if depth == 0 {
            for node_id in &resolved.update.node_refs_to_copy {
                match node_id {
                    RENodeId::GlobalResourceManager(..) => {
                        if self.current_frame.get_node_visibility(node_id).is_none() {
                            let offset = SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo);
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, NodeModuleId::TypeInfo, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, NodeModuleId::TypeInfo, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    RENodeId::GlobalPackage(package_address) => {
                        // TODO: Cleanup
                        {
                            match *package_address {
                                PACKAGE_LOADER
                                | RESOURCE_MANAGER_PACKAGE
                                | IDENTITY_PACKAGE
                                | EPOCH_MANAGER_PACKAGE
                                | CLOCK_PACKAGE
                                | ACCOUNT_PACKAGE
                                | ACCESS_CONTROLLER_PACKAGE => {
                                    self.current_frame
                                        .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }
                                _ => {
                                    if self.current_frame.get_node_visibility(node_id).is_none() {
                                        let offset =
                                            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo);
                                        self.track
                                            .acquire_lock(
                                                SubstateId(
                                                    *node_id,
                                                    NodeModuleId::TypeInfo,
                                                    offset.clone(),
                                                ),
                                                LockFlags::read_only(),
                                            )
                                            .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                                        self.track
                                            .release_lock(
                                                SubstateId(
                                                    *node_id,
                                                    NodeModuleId::TypeInfo,
                                                    offset,
                                                ),
                                                false,
                                            )
                                            .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                                        self.current_frame
                                            .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    }
                                }
                            }
                        }
                    }
                    RENodeId::GlobalComponent(global_address) => {
                        if matches!(
                            global_address,
                            ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                        ) || matches!(
                            global_address,
                            ComponentAddress::EddsaEd25519VirtualAccount(..)
                        ) || matches!(
                            global_address,
                            ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                        ) || matches!(
                            global_address,
                            ComponentAddress::EddsaEd25519VirtualIdentity(..)
                        ) {
                            // For virtual accounts and native packages, create a reference directly
                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                            continue;
                        }

                        if self.current_frame.get_node_visibility(node_id).is_none() {
                            let offset = SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo);
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, NodeModuleId::TypeInfo, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, NodeModuleId::TypeInfo, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    RENodeId::Vault(..) => {
                        if self.current_frame.get_node_visibility(node_id).is_none() {
                            let offset = SubstateOffset::Vault(VaultOffset::Info);
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
                                .add_ref(*node_id, RENodeVisibilityOrigin::DirectAccess);
                        }
                    }
                    _ => {}
                }
            }
        }

        let output = self.run(resolved)?;

        Ok(output)
    }

    fn execute_in_mode<X, RTN, E>(
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

impl<'g, 's, W> KernelNodeApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        KernelModuleMixer::before_drop_node(self, &node_id)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if let Some(actor) = &self.current_frame.actor {
            if !VisibilityProperties::check_drop_node_visibility(current_mode, actor, node_id) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidDropNodeAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                        node_id,
                    },
                ));
            }
        }

        let node = self.drop_node_internal(node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        KernelModuleMixer::after_drop_node(self)?;

        Ok(node)
    }

    fn kernel_allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        // TODO: Add costing
        let node_id = self.id_allocator.allocate_node_id(node_type)?;

        Ok(node_id)
    }

    fn kernel_create_node(
        &mut self,
        node_id: RENodeId,
        re_node: RENodeInit,
        mut module_init: BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        KernelModuleMixer::before_create_node(self, &node_id, &re_node, &module_init)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if let Some(actor) = &self.current_frame.actor {
            if !VisibilityProperties::check_create_node_access(
                current_mode,
                actor,
                &re_node,
                &module_init,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidCreateNodeAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                    },
                ));
            }
        }

        match (node_id, &re_node) {
            (RENodeId::GlobalComponent(..), RENodeInit::GlobalComponent(..)) => {}
            (RENodeId::Component(..), RENodeInit::Component(..)) => {}
            (RENodeId::KeyValueStore(..), RENodeInit::KeyValueStore) => {}
            (RENodeId::NonFungibleStore(..), RENodeInit::NonFungibleStore(..)) => {}
            (RENodeId::Component(..), RENodeInit::Metadata(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: METADATA_PACKAGE,
                        blueprint_name: METADATA_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Component(..), RENodeInit::ComponentRoyalty(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: ROYALTY_PACKAGE,
                        blueprint_name: COMPONENT_ROYALTY_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Component(..), RENodeInit::AccessRules(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: ACCESS_RULES_PACKAGE,
                        blueprint_name: ACCESS_RULES_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::AuthZoneStack, RENodeInit::AuthZoneStack(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: AUTH_ZONE_PACKAGE,
                        blueprint_name: AUTH_ZONE_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::TransactionRuntime, RENodeInit::TransactionRuntime(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: TRANSACTION_RUNTIME_PACKAGE,
                        blueprint_name: TRANSACTION_RUNTIME_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Logger, RENodeInit::Logger(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: LOGGER_PACKAGE,
                        blueprint_name: LOGGER_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Worktop, RENodeInit::Worktop(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: RESOURCE_MANAGER_PACKAGE,
                        blueprint_name: WORKTOP_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Bucket(..), RENodeInit::FungibleBucket(..))
            | (RENodeId::Bucket(..), RENodeInit::NonFungibleBucket(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: RESOURCE_MANAGER_PACKAGE,
                        blueprint_name: BUCKET_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Proof(..), RENodeInit::FungibleProof(..))
            | (RENodeId::Proof(..), RENodeInit::NonFungibleProof(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: RESOURCE_MANAGER_PACKAGE,
                        blueprint_name: PROOF_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Vault(..), RENodeInit::FungibleVault(..))
            | (RENodeId::Vault(..), RENodeInit::NonFungibleVault(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: RESOURCE_MANAGER_PACKAGE,
                        blueprint_name: VAULT_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::GlobalPackage(..), RENodeInit::GlobalPackage(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: PACKAGE_LOADER,
                        blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
                        global: true,
                    }),
                );
            }
            (RENodeId::GlobalResourceManager(..), RENodeInit::GlobalResourceManager(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: RESOURCE_MANAGER_PACKAGE,
                        blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        global: true,
                    }),
                );
            }
            (RENodeId::EpochManager(..), RENodeInit::EpochManager(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: EPOCH_MANAGER_PACKAGE,
                        blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Validator(..), RENodeInit::Validator(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: EPOCH_MANAGER_PACKAGE,
                        blueprint_name: VALIDATOR_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Clock(..), RENodeInit::Clock(..)) => { }
            (RENodeId::Identity(..), RENodeInit::Identity(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: IDENTITY_PACKAGE,
                        blueprint_name: IDENTITY_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::AccessController(..), RENodeInit::AccessController(..)) => {
                module_init.insert(
                    NodeModuleId::TypeInfo,
                    RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: ACCESS_CONTROLLER_PACKAGE,
                        blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
                        global: false,
                    }),
                );
            }
            (RENodeId::Account(..), RENodeInit::Account(..)) => { }
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
        }

        let push_to_store = match re_node {
            RENodeInit::GlobalComponent(..)
            | RENodeInit::GlobalPackage(..)
            | RENodeInit::GlobalResourceManager(..) => true,
            _ => false,
        };

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame.create_node(
            node_id,
            re_node,
            module_init,
            &mut self.heap,
            &mut self.track,
            push_to_store,
        )?;

        // Restore current mode
        self.execution_mode = current_mode;

        KernelModuleMixer::after_create_node(self, &node_id)?;

        Ok(())
    }
}

impl<'g, 's, W> KernelInternalApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_get_node_visibility_origin(
        &self,
        node_id: RENodeId,
    ) -> Option<RENodeVisibilityOrigin> {
        let visibility = self.current_frame.get_node_visibility(&node_id)?;
        Some(visibility)
    }

    fn kernel_get_module_state(&mut self) -> &mut KernelModuleMixer {
        &mut self.module
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth
    }

    fn kernel_get_current_actor(&self) -> Option<Actor> {
        self.current_frame.actor.clone()
    }

    fn kernel_read_bucket(&mut self, bucket_id: BucketId) -> Option<BucketSnapshot> {
        if let Ok(substate) = self.heap.get_substate(
            RENodeId::Bucket(bucket_id),
            NodeModuleId::SELF,
            &SubstateOffset::Bucket(BucketOffset::Info),
        ) {
            let info: &BucketInfoSubstate = substate.into();
            let info = info.clone();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            RENodeId::Bucket(bucket_id),
                            NodeModuleId::SELF,
                            &SubstateOffset::Bucket(BucketOffset::LiquidFungible),
                        )
                        .unwrap();
                    let liquid: &LiquidFungibleResource = substate.into();

                    Some(BucketSnapshot::Fungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        liquid: liquid.amount(),
                    })
                }
                ResourceType::NonFungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            RENodeId::Bucket(bucket_id),
                            NodeModuleId::SELF,
                            &SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                        )
                        .unwrap();
                    let liquid: &LiquidNonFungibleResource = substate.into();

                    Some(BucketSnapshot::NonFungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        liquid: liquid.ids().clone(),
                    })
                }
            }
        } else {
            None
        }
    }

    fn kernel_read_proof(&mut self, proof_id: BucketId) -> Option<ProofSnapshot> {
        if let Ok(substate) = self.heap.get_substate(
            RENodeId::Proof(proof_id),
            NodeModuleId::SELF,
            &SubstateOffset::Proof(ProofOffset::Info),
        ) {
            let info: &ProofInfoSubstate = substate.into();
            let info = info.clone();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            RENodeId::Proof(proof_id),
                            NodeModuleId::SELF,
                            &SubstateOffset::Proof(ProofOffset::Fungible),
                        )
                        .unwrap();
                    let proof: &FungibleProof = substate.into();

                    Some(ProofSnapshot::Fungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: info.restricted,
                        total_locked: proof.amount(),
                    })
                }
                ResourceType::NonFungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            RENodeId::Proof(proof_id),
                            NodeModuleId::SELF,
                            &SubstateOffset::Proof(ProofOffset::NonFungible),
                        )
                        .unwrap();
                    let proof: &NonFungibleProof = substate.into();

                    Some(ProofSnapshot::NonFungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: info.restricted,
                        total_locked: proof.non_fungible_local_ids().clone(),
                    })
                }
            }
        } else {
            None
        }
    }
}

impl<'g, 's, W> KernelSubstateApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_lock_substate(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        KernelModuleMixer::before_lock_substate(self, &node_id, &module_id, &offset, &flags)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // TODO: Check if valid offset for node_id

        // Authorization
        if let Some(actor) = &self.current_frame.actor {
            if !VisibilityProperties::check_substate_access(
                current_mode,
                actor,
                node_id,
                offset.clone(),
                flags,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidSubstateAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                        node_id,
                        offset,
                        flags,
                    },
                ));
            }
        }

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            &mut self.track,
            node_id,
            module_id,
            offset.clone(),
            flags,
        );

        let lock_handle = match maybe_lock_handle {
            Ok(lock_handle) => lock_handle,
            Err(RuntimeError::KernelError(KernelError::TrackError(TrackError::NotFound(
                SubstateId(node_id, module_id, ref offset),
            )))) => {
                if self.try_virtualize(node_id, module_id, &offset)? {
                    self.current_frame.acquire_lock(
                        &mut self.heap,
                        &mut self.track,
                        node_id,
                        module_id,
                        offset.clone(),
                        flags,
                    )?
                } else {
                    return maybe_lock_handle;
                }
            }
            Err(err) => {
                match &err {
                    // TODO: This is a hack to allow for package imports to be visible
                    // TODO: Remove this once we are able to get this information through the Blueprint ABI
                    RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(
                        RENodeId::GlobalPackage(package_address),
                    )) => {
                        let node_id = RENodeId::GlobalPackage(*package_address);
                        let module_id = NodeModuleId::SELF;
                        self.track
                            .acquire_lock(
                                SubstateId(node_id, module_id, offset.clone()),
                                LockFlags::read_only(),
                            )
                            .map_err(|_| err.clone())?;
                        self.track
                            .release_lock(SubstateId(node_id, module_id, offset.clone()), false)
                            .map_err(|_| err)?;
                        self.current_frame
                            .add_ref(node_id, RENodeVisibilityOrigin::Normal);
                        self.current_frame.acquire_lock(
                            &mut self.heap,
                            &mut self.track,
                            node_id,
                            module_id,
                            offset.clone(),
                            flags,
                        )?
                    }
                    _ => return Err(err),
                }
            }
        };

        // Restore current mode
        self.execution_mode = current_mode;

        // TODO: pass the right size
        KernelModuleMixer::after_lock_substate(self, lock_handle, 0)?;

        Ok(lock_handle)
    }

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame.get_lock_info(lock_handle)
    }

    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        KernelModuleMixer::on_drop_lock(self, lock_handle)?;

        self.current_frame
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;

        Ok(())
    }

    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.

        KernelModuleMixer::on_read_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref.to_scrypto_value())
    }

    fn kernel_get_substate_ref<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a S, RuntimeError>
    where
        &'a S: From<SubstateRef<'a>>,
        'b: 'a,
    {
        KernelModuleMixer::on_read_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref.into())
    }

    fn kernel_get_substate_ref_mut<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a mut S, RuntimeError>
    where
        &'a mut S: From<SubstateRefMut<'a>>,
        'b: 'a,
    {
        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        KernelModuleMixer::on_write_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref_mut.into())
    }
}

impl<'g, 's, W> KernelWasmApi<W> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_create_wasm_instance(
        &mut self,
        package_address: PackageAddress,
        handle: LockHandle,
    ) -> Result<W::WasmInstance, RuntimeError> {
        KernelModuleMixer::on_read_substate(
            self, handle, 0, //  TODO: pass the right size
        )?;
        let substate_ref = self
            .current_frame
            .get_ref(handle, &mut self.heap, &mut self.track)?;
        let code: &PackageCodeSubstate = substate_ref.into();
        let instance = self
            .scrypto_interpreter
            .create_instance(package_address, &code.code);
        Ok(instance)
    }
}

impl<'g, 's, W, N> Invokable<N, RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
    N: ExecutableInvocation,
{
    fn kernel_invoke(&mut self, invocation: N) -> Result<<N as Invocation>::Output, RuntimeError> {
        KernelModuleMixer::before_invoke(
            self,
            &invocation.debug_identifier(),
            invocation.payload_size(),
        )?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Resolver;
        let resolved = invocation.resolve(self)?;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke_internal(resolved)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        KernelModuleMixer::after_invoke(
            self, 0, // TODO: Pass the right size
        )?;

        Ok(rtn)
    }
}

impl<'g, 's, W> KernelApi<W, RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}

impl<'g, 's, W> KernelModuleApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}
