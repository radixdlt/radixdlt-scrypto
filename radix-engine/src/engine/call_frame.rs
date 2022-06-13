use colored::*;
use sbor::path::SborPath;
use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::marker::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::BlueprintAbi;
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::validation::*;

use crate::engine::LoadedSNodeState::{Borrowed, Consumed, Static};
use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame<
    'p, // Parent frame lifetime
    's, // Substate store lifetime
    't, // Track lifetime
    'w, // WASM engine lifetime
    S,  // Substore store type
    W,  // WASM engine type
    I,  // WASM instance type
> where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,

    /// State track
    track: &'t mut Track<'s, S>,
    /// Wasm engine
    wasm_engine: &'w mut W,

    /// Owned Values
    buckets: HashMap<BucketId, Bucket>,
    proofs: HashMap<ProofId, Proof>,
    owned_values: HashMap<StoredValueId, StoredValue>,

    /// Referenced values
    worktop: Option<Worktop>,
    auth_zone: Option<AuthZone>,
    component_state: Option<&'p mut ComponentState>,
    refed_values: HashMap<StoredValueId, ValueRefType>,

    /// Caller's auth zone
    caller_auth_zone: Option<&'p AuthZone>,

    phantom: PhantomData<I>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Owned,
    Ref(ValueRefType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRefType {
    Uncommitted {
        root: KeyValueStoreId,
        ancestors: Vec<KeyValueStoreId>,
    },
    Committed {
        component_address: ComponentAddress,
    },
}

fn stored_value_update(
    old: &ScryptoValue,
    new: &ScryptoValue,
) -> Result<HashSet<StoredValueId>, RuntimeError> {
    let old_ids = old.stored_value_ids();
    let new_ids = new.stored_value_ids();
    for old_id in &old_ids {
        if !new_ids.contains(old_id) {
            return Err(RuntimeError::StoredValueRemoved(old_id.clone()));
        }
    }

    let mut new_value_ids = HashSet::new();
    for new_id in new_ids {
        if !old_ids.contains(&new_id) {
            new_value_ids.insert(new_id);
        }
    }
    Ok(new_value_ids)
}

fn verify_stored_value(value: &ScryptoValue) -> Result<(), RuntimeError> {
    if !value.bucket_ids.is_empty() {
        return Err(RuntimeError::BucketNotAllowed);
    }
    if !value.proof_ids.is_empty() {
        return Err(RuntimeError::ProofNotAllowed);
    }
    Ok(())
}

fn verify_stored_key(value: &ScryptoValue) -> Result<(), RuntimeError> {
    if !value.bucket_ids.is_empty() {
        return Err(RuntimeError::BucketNotAllowed);
    }
    if !value.proof_ids.is_empty() {
        return Err(RuntimeError::ProofNotAllowed);
    }
    if !value.vault_ids.is_empty() {
        return Err(RuntimeError::VaultNotAllowed);
    }
    if !value.kv_store_ids.is_empty() {
        return Err(RuntimeError::KeyValueStoreNotAllowed);
    }
    Ok(())
}

pub enum ConsumedSNodeState {
    Bucket(Bucket),
    Proof(Proof),
}

pub enum BorrowedSNodeState {
    AuthZone(AuthZone),
    Worktop(Worktop),
    Scrypto(
        ScryptoActorInfo,
        BlueprintAbi,
        ValidatedPackage,
        String,
        Option<ComponentState>,
    ),
    Resource(ResourceAddress, ResourceManager),
    Bucket(BucketId, Bucket),
    Proof(ProofId, Proof),
    Vault(VaultId, Vault, ValueType),
}

pub enum StaticSNodeState {
    Package,
    Resource,
    System,
    TransactionProcessor,
}

pub enum LoadedSNodeState {
    Static(StaticSNodeState),
    Consumed(Option<ConsumedSNodeState>),
    Borrowed(BorrowedSNodeState),
}

pub enum SNodeState<'a> {
    Root,
    SystemStatic,
    TransactionProcessorStatic,
    PackageStatic,
    AuthZoneRef(&'a mut AuthZone),
    Worktop(&'a mut Worktop),
    // TODO: use reference to the package
    Scrypto(
        ScryptoActorInfo,
        BlueprintAbi,
        ValidatedPackage,
        String,
        Option<&'a mut ComponentState>,
    ),
    ResourceStatic,
    ResourceRef(ResourceAddress, &'a mut ResourceManager),
    BucketRef(BucketId, &'a mut Bucket),
    Bucket(Bucket),
    ProofRef(ProofId, &'a mut Proof),
    Proof(Proof),
    VaultRef(VaultId, &'a mut Vault),
}

#[derive(Debug)]
pub struct ComponentState {
    pub component_address: ComponentAddress,
    pub component: Component,
    pub initial_value: ScryptoValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveMethod {
    AsReturn,
    AsArgument,
}

impl LoadedSNodeState {
    fn to_snode_state(&mut self) -> SNodeState {
        match self {
            Static(static_state) => match static_state {
                StaticSNodeState::Package => SNodeState::PackageStatic,
                StaticSNodeState::Resource => SNodeState::ResourceStatic,
                StaticSNodeState::System => SNodeState::SystemStatic,
                StaticSNodeState::TransactionProcessor => SNodeState::TransactionProcessorStatic,
            },
            Consumed(ref mut to_consume) => match to_consume.take().unwrap() {
                ConsumedSNodeState::Proof(proof) => SNodeState::Proof(proof),
                ConsumedSNodeState::Bucket(bucket) => SNodeState::Bucket(bucket),
            },
            Borrowed(ref mut borrowed) => match borrowed {
                BorrowedSNodeState::AuthZone(s) => SNodeState::AuthZoneRef(s),
                BorrowedSNodeState::Worktop(s) => SNodeState::Worktop(s),
                BorrowedSNodeState::Scrypto(
                    info,
                    blueprint_abi,
                    package,
                    export_name,
                    component_state,
                ) => SNodeState::Scrypto(
                    info.clone(),
                    blueprint_abi.clone(),
                    package.clone(),
                    export_name.clone(),
                    component_state.as_mut(),
                ),
                BorrowedSNodeState::Resource(addr, s) => SNodeState::ResourceRef(*addr, s),
                BorrowedSNodeState::Bucket(id, s) => SNodeState::BucketRef(*id, s),
                BorrowedSNodeState::Proof(id, s) => SNodeState::ProofRef(*id, s),
                BorrowedSNodeState::Vault(id, vault, ..) => SNodeState::VaultRef(*id, vault),
            },
        }
    }
}

impl<'p, 's, 't, 'w, S, W, I> CallFrame<'p, 's, 't, 'w, S, W, I>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new_root(
        verbose: bool,
        transaction_hash: Hash,
        signer_public_keys: Vec<EcdsaPublicKey>,
        track: &'t mut Track<'s, S>,
        wasm_engine: &'w mut W,
    ) -> Self {
        let signer_public_keys: BTreeSet<NonFungibleId> = signer_public_keys
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleId::from_bytes(public_key.to_vec()))
            .collect();

        let mut initial_auth_zone_proofs = Vec::new();
        if !signer_public_keys.is_empty() {
            // Proofs can't be zero amount
            let mut ecdsa_bucket = Bucket::new(ResourceContainer::new_non_fungible(
                ECDSA_TOKEN,
                signer_public_keys,
            ));
            let ecdsa_proof = ecdsa_bucket.create_proof(ECDSA_TOKEN_BUCKET_ID).unwrap();
            initial_auth_zone_proofs.push(ecdsa_proof);
        }

        Self::new(
            transaction_hash,
            0,
            verbose,
            track,
            wasm_engine,
            Some(AuthZone::new_with_proofs(initial_auth_zone_proofs)),
            Some(Worktop::new()),
            HashMap::new(),
            HashMap::new(),
            None,
        )
    }

    pub fn new(
        transaction_hash: Hash,
        depth: usize,
        trace: bool,
        track: &'t mut Track<'s, S>,
        wasm_engine: &'w mut W,
        auth_zone: Option<AuthZone>,
        worktop: Option<Worktop>,
        buckets: HashMap<BucketId, Bucket>,
        proofs: HashMap<ProofId, Proof>,
        caller_auth_zone: Option<&'p AuthZone>,
    ) -> Self {
        Self {
            transaction_hash,
            depth,
            trace,
            track,
            wasm_engine,
            buckets,
            proofs,
            owned_values: HashMap::new(),
            refed_values: HashMap::new(),
            worktop,
            auth_zone,
            caller_auth_zone,
            component_state: None,
            phantom: PhantomData,
        }
    }

    /// Checks resource leak.
    fn check_resource(&mut self) -> Result<(), RuntimeError> {
        self.sys_log(Level::Info, "Resource check started".to_string());
        let mut success = true;
        let mut resource = ResourceFailure::Unknown;

        for (bucket_id, bucket) in &self.buckets {
            self.sys_log(
                Level::Warn,
                format!("Dangling bucket: {}, {:?}", bucket_id, bucket),
            );
            resource = ResourceFailure::Resource(bucket.resource_address());
            success = false;
        }

        let values: HashMap<StoredValueId, StoredValue> = self.owned_values.drain().collect();
        for (_, value) in values {
            self.sys_log(Level::Warn, format!("Dangling value: {:?}", value));
            resource = match value {
                StoredValue::Vault(_, vault) => ResourceFailure::Resource(vault.resource_address()),
                StoredValue::KeyValueStore(..) => ResourceFailure::UnclaimedKeyValueStore,
            };
            success = false;
        }

        if let Some(worktop) = &self.worktop {
            if !worktop.is_empty() {
                self.sys_log(Level::Warn, "Resource worktop is not empty".to_string());
                resource = ResourceFailure::Resources(worktop.resource_addresses());
                success = false;
            }
        }

        self.sys_log(Level::Info, "Resource check ended".to_string());
        if success {
            Ok(())
        } else {
            Err(RuntimeError::ResourceCheckFailure(resource))
        }
    }

    fn process_call_data(validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(())
    }

    fn process_return_data(
        &mut self,
        from: Option<SNodeRef>,
        validated: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }

        // Allow vaults to be returned from ResourceStatic
        // TODO: Should we allow vaults to be returned by any component?
        if !matches!(from, Some(SNodeRef::ResourceRef(_))) {
            if !validated.vault_ids.is_empty() {
                return Err(RuntimeError::VaultNotAllowed);
            }
        }

        Ok(())
    }

    /// Sends buckets to another component/blueprint, either as argument or return
    fn send_buckets(
        &mut self,
        bucket_ids: &HashMap<BucketId, SborPath>,
    ) -> Result<HashMap<BucketId, Bucket>, RuntimeError> {
        let mut buckets = HashMap::new();
        for (bucket_id, _) in bucket_ids {
            let bucket = self
                .buckets
                .remove(bucket_id)
                .ok_or(RuntimeError::BucketNotFound(*bucket_id))?;
            if bucket.is_locked() {
                return Err(RuntimeError::CantMoveLockedBucket);
            }
            buckets.insert(*bucket_id, bucket);
        }
        Ok(buckets)
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_vaults(
        &mut self,
        vault_ids: &HashSet<VaultId>,
    ) -> Result<HashMap<VaultId, Vault>, RuntimeError> {
        let mut vault_ids_to_take = HashSet::new();
        for vault_id in vault_ids {
            vault_ids_to_take.insert(StoredValueId::VaultId(*vault_id));
        }
        let vaults_to_take = self.take_set(&vault_ids_to_take)?;

        let mut vaults = HashMap::new();
        for vault_to_take in vaults_to_take {
            match vault_to_take {
                StoredValue::Vault(vault_id, vault) => {
                    vaults.insert(vault_id, vault);
                }
                _ => panic!("Expected vault but was {:?}", vault_to_take),
            }
        }

        Ok(vaults)
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_proofs(
        &mut self,
        proof_ids: &HashMap<ProofId, SborPath>,
        method: MoveMethod,
    ) -> Result<HashMap<ProofId, Proof>, RuntimeError> {
        let mut proofs = HashMap::new();
        for (proof_id, _) in proof_ids {
            let mut proof = self
                .proofs
                .remove(proof_id)
                .ok_or(RuntimeError::ProofNotFound(*proof_id))?;
            if proof.is_restricted() {
                return Err(RuntimeError::CantMoveRestrictedProof(*proof_id));
            }
            if matches!(method, MoveMethod::AsArgument) {
                proof.change_to_restricted();
            }
            proofs.insert(*proof_id, proof);
        }
        Ok(proofs)
    }

    pub fn run(
        &mut self,
        snode_ref: Option<SNodeRef>, // TODO: Remove, abstractions between invoke_snode() and run() are a bit messy right now
        snode: SNodeState<'p>,
        fn_ident: &str,
        input: ScryptoValue,
    ) -> Result<
        (
            ScryptoValue,
            HashMap<BucketId, Bucket>,
            HashMap<ProofId, Proof>,
            HashMap<VaultId, Vault>,
        ),
        RuntimeError,
    > {
        let output = match snode {
            SNodeState::Root => {
                panic!("Root is not runnable")
            }
            SNodeState::SystemStatic => {
                System::static_main(fn_ident, input, self).map_err(RuntimeError::SystemError)
            }
            SNodeState::TransactionProcessorStatic => {
                TransactionProcessor::static_main(fn_ident, input, self).map_err(|e| match e {
                    TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                    TransactionProcessorError::InvalidMethod => panic!("Illegal state"),
                    TransactionProcessorError::RuntimeError(e) => e,
                })
            }
            SNodeState::PackageStatic => ValidatedPackage::static_main(fn_ident, input, self)
                .map_err(RuntimeError::PackageError),
            SNodeState::AuthZoneRef(auth_zone) => auth_zone
                .main(fn_ident, input, self)
                .map_err(RuntimeError::AuthZoneError),
            SNodeState::Worktop(worktop) => worktop
                .main(fn_ident, input, self)
                .map_err(RuntimeError::WorktopError),
            SNodeState::Scrypto(actor, blueprint_abi, package, export_name, component_state) => {
                self.component_state = component_state;
                package.invoke(actor, blueprint_abi, export_name, fn_ident, input, self)
            }
            SNodeState::ResourceStatic => ResourceManager::static_main(fn_ident, input, self)
                .map_err(RuntimeError::ResourceManagerError),
            SNodeState::ResourceRef(resource_address, resource_manager) => {
                let return_value = resource_manager
                    .main(resource_address, fn_ident, input, self)
                    .map_err(RuntimeError::ResourceManagerError)?;

                Ok(return_value)
            }
            SNodeState::BucketRef(bucket_id, bucket) => bucket
                .main(bucket_id, fn_ident, input, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::Bucket(bucket) => bucket
                .consuming_main(fn_ident, input, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::ProofRef(_, proof) => proof
                .main(fn_ident, input, self)
                .map_err(RuntimeError::ProofError),
            SNodeState::Proof(proof) => proof
                .main_consume(fn_ident, input)
                .map_err(RuntimeError::ProofError),
            SNodeState::VaultRef(vault_id, vault) => vault
                .main(vault_id, fn_ident, input, self)
                .map_err(RuntimeError::VaultError),
        }?;

        self.process_return_data(snode_ref, &output)?;

        // figure out what buckets and resources to return
        let moving_buckets = self.send_buckets(&output.bucket_ids)?;
        let moving_proofs = self.send_proofs(&output.proof_ids, MoveMethod::AsReturn)?;
        let moving_vaults = self.send_vaults(&output.vault_ids)?;

        // drop proofs and check resource leak
        for (_, proof) in self.proofs.drain() {
            proof.drop();
        }

        if let Some(_) = &mut self.auth_zone {
            self.invoke_snode(
                SNodeRef::AuthZoneRef,
                "clear".to_string(),
                ScryptoValue::from_typed(&AuthZoneClearInput {}),
            )?;
        }
        self.check_resource()?;

        Ok((output, moving_buckets, moving_proofs, moving_vaults))
    }

    fn take_values(
        &mut self,
        value_ids: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let values = self.take_set(value_ids)?;
        for value in &values {
            if let StoredValue::KeyValueStore(_, store) = value {
                for id in store.all_descendants() {
                    self.refed_values.remove(&id);
                }
            }
        }
        Ok(values)
    }

    fn read_kv_store_entry_internal(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: &ScryptoValue,
    ) -> Result<(Option<ScryptoValue>, ValueType), RuntimeError> {
        verify_stored_key(key)?;

        let (maybe_value, value_type) = if self
            .owned_values
            .contains_key(&StoredValueId::KeyValueStoreId(kv_store_id.clone()))
        {
            let store = Self::get_owned_kv_store_mut(&mut self.owned_values, &kv_store_id).unwrap();
            let value = store.store.get(&key.raw).cloned();
            (value, ValueType::Owned)
        } else {
            let maybe_value_ref = self
                .refed_values
                .get(&StoredValueId::KeyValueStoreId(kv_store_id.clone()))
                .cloned();
            let value_ref =
                maybe_value_ref.ok_or(RuntimeError::KeyValueStoreNotFound(kv_store_id.clone()))?;
            let value = match &value_ref {
                ValueRefType::Uncommitted { root, ancestors } => {
                    let root_store = Self::get_owned_kv_store_mut(&mut self.owned_values, root).unwrap();
                    let store = root_store.get_child_kv_store(ancestors, &kv_store_id);
                    store.store.get(&key.raw).cloned()
                }
                ValueRefType::Committed { component_address } => {
                    let substate_value = self.track.read_key_value(
                        Address::KeyValueStore(*component_address, kv_store_id),
                        key.raw.to_vec(),
                    );
                    match substate_value {
                        SubstateValue::KeyValueStoreEntry(v) => v,
                        _ => panic!("Substate value is not a KeyValueStore entry"),
                    }
                    .map(|v| ScryptoValue::from_slice(&v).expect("Expected to decode."))
                }
            };
            (value, ValueType::Ref(value_ref))
        };

        Ok((maybe_value, value_type))
    }

    pub fn take_set(
        &mut self,
        other: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let mut taken_values = Vec::new();

        for id in other {
            let value = self
                .owned_values
                .remove(id)
                .ok_or(RuntimeError::ValueNotFound(*id))?;
            taken_values.push(value);
        }

        Ok(taken_values)
    }

    pub fn get_owned_kv_store_mut<'a>(
        owned_values: &'a mut HashMap<StoredValueId, StoredValue>,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&'a mut PreCommittedKeyValueStore> {
        owned_values
            .get_mut(&StoredValueId::KeyValueStoreId(*kv_store_id))
            .map(|v| match v {
                StoredValue::KeyValueStore(_, store) => store,
                _ => panic!("Expected KV store"),
            })
    }
}

impl<'p, 's, 't, 'w, S, W, I> SystemApi<W, I> for CallFrame<'p, 's, 't, 'w, S, W, I>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn wasm_engine(&mut self) -> &mut W {
        self.wasm_engine
    }

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        self.sys_log(Level::Debug, format!("{:?} {:?}", snode_ref, &fn_ident));

        // Authorization and state load
        let (mut loaded_snode, method_auths) = match &snode_ref {
            SNodeRef::TransactionProcessor => {
                // FIXME: only TransactionExecutor can invoke this function
                Ok((Static(StaticSNodeState::TransactionProcessor), vec![]))
            }
            SNodeRef::PackageStatic => Ok((Static(StaticSNodeState::Package), vec![])),
            SNodeRef::SystemStatic => Ok((Static(StaticSNodeState::System), vec![])),
            SNodeRef::AuthZoneRef => {
                if let Some(auth_zone) = self.auth_zone.take() {
                    Ok((Borrowed(BorrowedSNodeState::AuthZone(auth_zone)), vec![]))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            SNodeRef::WorktopRef => {
                if let Some(worktop) = self.worktop.take() {
                    Ok((Borrowed(BorrowedSNodeState::Worktop(worktop)), vec![]))
                } else {
                    Err(RuntimeError::WorktopDoesNotExist)
                }
            }
            SNodeRef::Scrypto(actor) => {
                match actor {
                    ScryptoActor::Blueprint(package_address, blueprint_name) => {
                        let substate_value = self
                            .track
                            .read_value(package_address.clone())
                            .ok_or(RuntimeError::PackageNotFound(*package_address))?;
                        let package = match substate_value {
                            SubstateValue::Package(package) => package,
                            _ => panic!("Value is not a package"),
                        };
                        let abi = package.blueprint_abi(blueprint_name).ok_or(
                            RuntimeError::BlueprintNotFound(
                                package_address.clone(),
                                blueprint_name.clone(),
                            ),
                        )?;
                        let fn_abi = abi
                            .get_fn_abi(&fn_ident)
                            .ok_or(RuntimeError::MethodDoesNotExist(fn_ident.clone()))?;
                        if !fn_abi.input.matches(&input.dom) {
                            return Err(RuntimeError::InvalidMethodArgument {
                                fn_ident,
                                input: input.dom,
                            });
                        }
                        let export_name = format!("{}_main", blueprint_name);

                        Ok((
                            Borrowed(BorrowedSNodeState::Scrypto(
                                ScryptoActorInfo::blueprint(
                                    package_address.clone(),
                                    blueprint_name.clone(),
                                ),
                                abi.clone(),
                                package.clone(),
                                export_name.clone(),
                                None,
                            )),
                            vec![],
                        ))
                    }
                    ScryptoActor::Component(component_address) => {
                        let component_address = *component_address;

                        let component: Component = self
                            .track
                            .borrow_global_mut_value(component_address)
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ComponentNotFound(component_address)
                                }
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address)
                                }
                            })?
                            .into();
                        let package_address = component.package_address();
                        let blueprint_name = component.blueprint_name().to_string();
                        let export_name = format!("{}_main", blueprint_name);

                        let substate_value = self
                            .track
                            .read_value(package_address)
                            .ok_or(RuntimeError::PackageNotFound(package_address))?;
                        let package = match substate_value {
                            SubstateValue::Package(package) => package,
                            _ => panic!("Value is not a package"),
                        };

                        let abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint not found for existing component");
                        let fn_abi = abi
                            .get_fn_abi(&fn_ident)
                            .ok_or(RuntimeError::MethodDoesNotExist(fn_ident.clone()))?;
                        if !fn_abi.input.matches(&input.dom) {
                            return Err(RuntimeError::InvalidMethodArgument {
                                fn_ident,
                                input: input.dom,
                            });
                        }
                        let (_, method_auths) =
                            component.method_authorization(&abi.structure, &fn_ident);

                        // set up component state
                        let initial_value = ScryptoValue::from_slice(component.state()).unwrap();

                        Ok((
                            Borrowed(BorrowedSNodeState::Scrypto(
                                ScryptoActorInfo::component(
                                    package_address,
                                    blueprint_name,
                                    component_address,
                                ),
                                abi.clone(),
                                package.clone(),
                                export_name,
                                Some(ComponentState {
                                    component_address,
                                    component,
                                    initial_value,
                                }),
                            )),
                            method_auths,
                        ))
                    }
                }
            }
            SNodeRef::ResourceStatic => Ok((Static(StaticSNodeState::Resource), vec![])),
            SNodeRef::ResourceRef(resource_address) => {
                let resource_manager: ResourceManager = self
                    .track
                    .borrow_global_mut_value(resource_address.clone())
                    .map_err(|e| match e {
                        TrackError::NotFound => {
                            RuntimeError::ResourceManagerNotFound(resource_address.clone())
                        }
                        TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
                    })?
                    .into();

                let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                Ok((
                    Borrowed(BorrowedSNodeState::Resource(
                        resource_address.clone(),
                        resource_manager,
                    )),
                    vec![method_auth],
                ))
            }
            SNodeRef::Bucket(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let resource_address = bucket.resource_address();
                let substate_value = self.track.read_value(resource_address.clone()).unwrap();
                let resource_manager = match substate_value {
                    SubstateValue::Resource(resource_manager) => resource_manager,
                    _ => panic!("Value is not a resource manager"),
                };
                let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);
                Ok((
                    Consumed(Some(ConsumedSNodeState::Bucket(bucket))),
                    vec![method_auth.clone()],
                ))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                Ok((
                    Borrowed(BorrowedSNodeState::Bucket(bucket_id.clone(), bucket)),
                    vec![],
                ))
            }
            SNodeRef::ProofRef(proof_id) => {
                let proof = self
                    .proofs
                    .remove(&proof_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                Ok((
                    Borrowed(BorrowedSNodeState::Proof(proof_id.clone(), proof)),
                    vec![],
                ))
            }
            SNodeRef::Proof(proof_id) => {
                let proof = self
                    .proofs
                    .remove(&proof_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                Ok((Consumed(Some(ConsumedSNodeState::Proof(proof))), vec![]))
            }
            SNodeRef::VaultRef(vault_id) => {
                let (value_type, vault) = {
                    if let Some(value) =
                        self.owned_values.remove(&StoredValueId::VaultId(*vault_id))
                    {
                        match value {
                            StoredValue::Vault(_, vault) => (ValueType::Owned, vault),
                            _ => panic!("Expected vault"),
                        }
                    } else {
                        let value_id = StoredValueId::VaultId(*vault_id);
                        let maybe_value_ref = self.refed_values.get(&value_id).cloned();
                        let value_ref =
                            maybe_value_ref.ok_or(RuntimeError::ValueNotFound(value_id.clone()))?;
                        let vault = match &value_ref {
                            ValueRefType::Uncommitted { root, ancestors } => {
                                let root_store = Self::get_owned_kv_store_mut(&mut self.owned_values, root).unwrap();
                                root_store.take_child_vault(ancestors, vault_id)
                            }
                            ValueRefType::Committed { component_address } => self
                                .track
                                .borrow_global_mut_value((*component_address, *vault_id))
                                .map_err(|e| match e {
                                    TrackError::NotFound => panic!("Expected to find vault"),
                                    TrackError::Reentrancy => {
                                        panic!("Vault logic is causing reentrancy")
                                    }
                                })?
                                .into(),
                        };
                        (ValueType::Ref(value_ref), vault)
                    }
                };

                let resource_address = vault.resource_address();
                let substate_value = self.track.read_value(resource_address.clone()).unwrap();
                let resource_manager = match substate_value {
                    SubstateValue::Resource(resource_manager) => resource_manager,
                    _ => panic!("Value is not a resource manager"),
                };

                let method_auth = resource_manager.get_vault_auth(&fn_ident);
                Ok((
                    Borrowed(BorrowedSNodeState::Vault(
                        vault_id.clone(),
                        vault,
                        value_type,
                    )),
                    vec![method_auth.clone()],
                ))
            }
        }?;

        // Authorization check
        if !method_auths.is_empty() {
            let mut auth_zones = Vec::new();
            if let Some(self_auth_zone) = &self.auth_zone {
                auth_zones.push(self_auth_zone);
            }

            match &loaded_snode {
                // Resource auth check includes caller
                Borrowed(BorrowedSNodeState::Resource(_, _))
                | Borrowed(BorrowedSNodeState::Vault(_, _, _))
                | Borrowed(BorrowedSNodeState::Bucket(_, _))
                | Borrowed(BorrowedSNodeState::Scrypto(..))
                | Consumed(Some(ConsumedSNodeState::Bucket(_))) => {
                    if let Some(auth_zone) = self.caller_auth_zone {
                        auth_zones.push(auth_zone);
                    }
                }
                // Extern call auth check
                _ => {}
            };

            for method_auth in method_auths {
                method_auth.check(&auth_zones).map_err(|error| {
                    RuntimeError::AuthorizationError {
                        function: fn_ident.clone(),
                        authorization: method_auth,
                        error,
                    }
                })?;
            }
        }

        // Figure out what buckets and proofs to move from this process
        let mut moving_buckets = HashMap::new();
        let mut moving_proofs = HashMap::new();
        Self::process_call_data(&input)?;
        moving_buckets.extend(self.send_buckets(&input.bucket_ids)?);
        moving_proofs.extend(self.send_proofs(&input.proof_ids, MoveMethod::AsArgument)?);
        self.sys_log(
            Level::Debug,
            format!("Sending buckets: {:?}", moving_buckets),
        );
        self.sys_log(Level::Debug, format!("Sending proofs: {:?}", moving_proofs));

        // start a new frame
        let mut frame = CallFrame::new(
            self.transaction_hash,
            self.depth + 1,
            self.trace,
            self.track,
            self.wasm_engine,
            match loaded_snode {
                Borrowed(BorrowedSNodeState::Scrypto(..))
                | Static(StaticSNodeState::TransactionProcessor) => Some(AuthZone::new()),
                _ => None,
            },
            match loaded_snode {
                Static(StaticSNodeState::TransactionProcessor) => Some(Worktop::new()),
                _ => None,
            },
            moving_buckets,
            moving_proofs,
            self.auth_zone.as_ref(),
        );

        // invoke the main function
        let snode = loaded_snode.to_snode_state();
        let (result, received_buckets, received_proofs, mut received_vaults) =
            frame.run(Some(snode_ref), snode, &fn_ident, input)?;

        // move buckets and proofs to this process.
        self.sys_log(
            Level::Debug,
            format!("Received buckets: {:?}", received_buckets),
        );
        self.sys_log(
            Level::Debug,
            format!("Received proofs: {:?}", received_proofs),
        );
        self.buckets.extend(received_buckets);
        self.proofs.extend(received_proofs);
        for (vault_id, vault) in received_vaults.drain() {
            self.owned_values.insert(
                StoredValueId::VaultId(vault_id.clone()),
                StoredValue::Vault(vault_id, vault),
            );
        }

        // Return borrowed snodes
        if let Borrowed(borrowed) = loaded_snode {
            match borrowed {
                BorrowedSNodeState::AuthZone(auth_zone) => {
                    self.auth_zone = Some(auth_zone);
                }
                BorrowedSNodeState::Worktop(worktop) => {
                    self.worktop = Some(worktop);
                }
                BorrowedSNodeState::Scrypto(actor, _, _, _, component_state) => {
                    if let Some(component_address) = actor.component_address() {
                        self.track.return_borrowed_global_mut_value(
                            component_address,
                            component_state.unwrap().component, // TODO: how about the refs?
                        );
                    }
                }
                BorrowedSNodeState::Resource(resource_address, resource_manager) => {
                    self
                        .track
                        .return_borrowed_global_mut_value(resource_address, resource_manager);
                }
                BorrowedSNodeState::Bucket(bucket_id, bucket) => {
                    self.buckets.insert(bucket_id, bucket);
                }
                BorrowedSNodeState::Proof(proof_id, proof) => {
                    self.proofs.insert(proof_id, proof);
                }
                BorrowedSNodeState::Vault(vault_id, vault, value_type) => match value_type {
                    ValueType::Owned => {
                        self.owned_values.insert(
                            StoredValueId::VaultId(vault_id.clone()),
                            StoredValue::Vault(vault_id, vault),
                        );
                    }
                    ValueType::Ref(ValueRefType::Uncommitted { root, ancestors }) => {
                        let store = Self::get_owned_kv_store_mut(&mut self.owned_values, &root).unwrap();
                        store.put_child_vault(&ancestors, vault_id, vault);
                    }
                    ValueType::Ref(ValueRefType::Committed { component_address }) => {
                        self
                            .track
                            .return_borrowed_global_mut_value((component_address, vault_id), vault);
                    }
                },
            }
        }

        Ok(result)
    }

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible> {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        if let SubstateValue::NonFungible(non_fungible) =
            self.track.read_key_value(parent_address, key)
        {
            non_fungible
        } else {
            panic!("Value is not a non fungible");
        }
    }

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    ) {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        self.track.set_key_value(parent_address, key, non_fungible)
    }

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        self.track
            .borrow_global_mut_value(resource_address.clone())
            .map(|v| v.into())
            .map_err(|e| match e {
                TrackError::NotFound => {
                    RuntimeError::ResourceManagerNotFound(resource_address.clone())
                }
                TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
            })
    }

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    ) {
        self.track
            .return_borrowed_global_mut_value(resource_address, resource_manager)
    }

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError> {
        let proof_id = self.track.new_proof_id();
        self.proofs.insert(proof_id, proof);
        Ok(proof_id)
    }

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError> {
        let proof = self
            .proofs
            .remove(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;

        Ok(proof)
    }

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError> {
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, Bucket::new(container));
        Ok(bucket_id)
    }

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError> {
        let vault_id = self.track.new_vault_id();
        self.owned_values.insert(
            StoredValueId::VaultId(vault_id.clone()),
            StoredValue::Vault(vault_id, Vault::new(container)),
        );
        Ok(vault_id)
    }

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError> {
        self.buckets
            .remove(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))
    }

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress {
        self.track.create_uuid_value(resource_manager).into()
    }

    fn create_package(&mut self, package: ValidatedPackage) -> PackageAddress {
        self.track.create_uuid_value(package).into()
    }

    fn create_component(&mut self, component: Component) -> Result<ComponentAddress, RuntimeError> {
        let value =
            ScryptoValue::from_slice(component.state()).map_err(RuntimeError::DecodeError)?;
        verify_stored_value(&value)?;
        let values = self.take_values(&value.stored_value_ids())?;
        let address = self.track.create_uuid_value(component);
        self.track
            .insert_objects_into_component(values, address.clone().into());
        Ok(address.into())
    }

    fn read_component_state(&mut self, addr: ComponentAddress) -> Result<Vec<u8>, RuntimeError> {
        if let Some(ComponentState {
            component_address,
            component,
            initial_value,
        }) = &mut self.component_state
        {
            if addr.eq(component_address) {
                for value_id in initial_value.stored_value_ids() {
                    self.refed_values.insert(
                        value_id,
                        ValueRefType::Committed {
                            component_address: *component_address,
                        },
                    );
                }
                let state = component.state().to_vec();
                return Ok(state);
            }
        }

        Err(RuntimeError::ComponentNotFound(addr))
    }

    fn write_component_state(
        &mut self,
        addr: ComponentAddress,
        state: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        verify_stored_value(&state)?;

        if let Some(ComponentState {
            component_address,
            component,
            initial_value,
            ..
        }) = &mut self.component_state
        {
            if addr.eq(component_address) {
                let new_value_ids = stored_value_update(initial_value, &state)?;
                component.set_state(state.raw);
                let addr = *component_address;
                let new_values = self.take_values(&new_value_ids)?;
                self.track.insert_objects_into_component(new_values, addr);
                return Ok(());
            }
        }
        Err(RuntimeError::ComponentNotFound(addr))
    }

    fn read_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        verify_stored_key(&key)?;

        let (maybe_value, parent_type) =
            self.read_kv_store_entry_internal(kv_store_id.clone(), &key)?;

        let ref_type = match parent_type {
            ValueType::Owned => ValueRefType::Uncommitted {
                root: kv_store_id,
                ancestors: vec![],
            },
            ValueType::Ref(ValueRefType::Uncommitted { root, ancestors }) => {
                let mut next_ancestors = ancestors.clone();
                next_ancestors.push(kv_store_id);
                ValueRefType::Uncommitted {
                    root: root.clone(),
                    ancestors: next_ancestors,
                }
            }
            ValueType::Ref(ValueRefType::Committed { component_address }) => {
                ValueRefType::Committed { component_address }
            }
        };
        match maybe_value {
            Some(v) => {
                for value_id in v.stored_value_ids() {
                    self.refed_values.insert(value_id, ref_type.clone());
                }

                let value = Value::Option {
                    value: Box::new(Some(v.dom)),
                };
                let encoded = encode_any(&value);
                Ok(ScryptoValue::from_slice(&encoded).unwrap())
            }
            None => {
                let value = Value::Option {
                    value: Box::new(Option::None),
                };
                let encoded = encode_any(&value);
                Ok(ScryptoValue::from_slice(&encoded).unwrap())
            }
        }
    }

    fn write_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: ScryptoValue,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        verify_stored_value(&value)?;

        let (old_value, parent_type) = self.read_kv_store_entry_internal(kv_store_id, &key)?;
        let new_value_ids = match old_value {
            None => value.stored_value_ids(),
            Some(old_scrypto_value) => stored_value_update(&old_scrypto_value, &value)?,
        };
        let new_values = self.take_values(&new_value_ids)?;
        match parent_type {
            ValueType::Owned => {
                let kv_store = Self::get_owned_kv_store_mut(&mut self.owned_values, &kv_store_id)
                    .ok_or(RuntimeError::CyclicKeyValueStore(kv_store_id))?;
                kv_store.store.insert(key.raw, value);
                kv_store.insert_children(new_values)
            }
            ValueType::Ref(ValueRefType::Uncommitted { root, ancestors }) => {
                if let Some(root_store) = Self::get_owned_kv_store_mut(&mut self.owned_values, &root) {
                    let kv_store = root_store.get_child_kv_store(&ancestors, &kv_store_id);
                    kv_store.store.insert(key.raw, value);
                    kv_store.insert_children(new_values)
                } else {
                    return Err(RuntimeError::CyclicKeyValueStore(kv_store_id.clone()));
                }
            }
            ValueType::Ref(ValueRefType::Committed { component_address }) => {
                self.track.set_key_value(
                    Address::KeyValueStore(component_address.clone(), kv_store_id),
                    key.raw,
                    SubstateValue::KeyValueStoreEntry(Some(value.raw)),
                );
                self.track
                    .insert_objects_into_component(new_values, component_address);
            }
        }

        Ok(())
    }

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let substate_value = self
            .track
            .read_value(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;

        if let SubstateValue::Component(component) = substate_value {
            Ok((
                component.package_address(),
                component.blueprint_name().to_owned(),
            ))
        } else {
            panic!("Value is not a component");
        }
    }

    fn create_kv_store(&mut self) -> KeyValueStoreId {
        let kv_store_id = self.track.new_kv_store_id();
        self.owned_values.insert(
            StoredValueId::KeyValueStoreId(kv_store_id.clone()),
            StoredValue::KeyValueStore(kv_store_id, PreCommittedKeyValueStore::new()),
        );
        kv_store_id
    }

    fn get_epoch(&mut self) -> u64 {
        self.track.current_epoch()
    }

    fn get_transaction_hash(&mut self) -> Hash {
        self.track.transaction_hash()
    }

    fn generate_uuid(&mut self) -> u128 {
        self.track.new_uuid()
    }

    fn user_log(&mut self, level: Level, message: String) {
        self.track.add_log(level, message);
    }

    #[allow(unused_variables)]
    fn sys_log(&self, level: Level, message: String) {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), message.red()),
            Level::Warn => ("WARN".yellow(), message.yellow()),
            Level::Info => ("INFO".green(), message.green()),
            Level::Debug => ("DEBUG".cyan(), message.cyan()),
            Level::Trace => ("TRACE".normal(), message.normal()),
        };

        #[cfg(not(feature = "alloc"))]
        if self.trace {
            println!("{}[{:5}] {}", "  ".repeat(self.depth), l, m);
        }
    }

    fn check_access_rule(
        &mut self,
        access_rule: scrypto::resource::AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError> {
        let proofs = proof_ids
            .iter()
            .map(|proof_id| {
                self.proofs
                    .get(&proof_id)
                    .map(Proof::clone)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))
            })
            .collect::<Result<Vec<Proof>, RuntimeError>>()?;
        let mut simulated_auth_zone = AuthZone::new_with_proofs(proofs);

        let method_authorization = convert(&Type::Unit, &Value::Unit, &access_rule);
        let is_authorized = method_authorization.check(&[&simulated_auth_zone]).is_ok();
        simulated_auth_zone
            .main(
                "clear",
                ScryptoValue::from_typed(&AuthZoneClearInput {}),
                self,
            )
            .map_err(RuntimeError::AuthZoneError)?;

        Ok(is_authorized)
    }
}
