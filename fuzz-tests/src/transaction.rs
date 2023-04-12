#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;
#[cfg(feature = "libfuzzer-sys")]
use once_cell::sync::Lazy;

#[cfg(feature = "afl")]
use afl::fuzz;
#[cfg(feature = "afl")]
use std::panic::AssertUnwindSafe;

#[cfg(feature = "simple-fuzzer")]
mod simple_fuzzer;

//use radix_engine::types::{ComponentAddress, EcdsaSecp256k1PublicKey, ResourceAddress};
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerCreateValidatorInput, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::manifest::manifest_decode;
use radix_engine_stores::interface::SubstateDatabase;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rand_chacha;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use scrypto_unit::TestRunner;
use scrypto_unit::{TestRunner, TestRunnerSnapshot};
use strum::EnumCount;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::manifest::ast;
use transaction::model::Instruction;
use transaction::model::TransactionManifest;

#[derive(Debug, Clone)]
struct Account {
    public_key: EcdsaSecp256k1PublicKey,
    //_private_key: EcdsaSecp256k1PrivateKey,
    address: ComponentAddress,
    fungibles: Container<ResourceAddress>,
    non_fungibles: Container<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Container<T> {
    rng: ChaCha8Rng,
    elems: Vec<T>,
}

impl<T: Copy> Container<T> {
    fn new(elems: Vec<T>) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);
        Self { rng, elems }
    }

    fn get_idx(&mut self, idx: usize, drop: bool) -> Option<T> {
        let len = self.elems.len();
        if idx < len {
            if drop {
                Some(self.elems.remove(idx))
            } else {
                Some(self.elems[idx])
            }
        } else {
            None
        }
    }

    fn get_random(&mut self, drop: bool) -> Option<T> {
        let len = self.elems.len();
        if len > 0 {
            let idx = self.rng.gen_range(0usize..len);
            if drop {
                Some(self.elems.remove(idx))
            } else {
                Some(self.elems[idx])
            }
        } else {
            None
        }
    }

    fn push(&mut self, elem: T) {
        self.elems.push(elem)
    }
}

struct Fuzzer {
    rng: ChaCha8Rng,
    runner: TestRunner,
    snapshot: TestRunnerSnapshot,
    accounts: Vec<Account>,
}

impl Fuzzer {
    fn new() -> Self {
        let mut runner = TestRunner::builder().without_trace().build();
        let mut accounts: Vec<Account> = (0..2)
            .map(|_| {
                let acc = runner.new_account(false);
                let fungibles = Container::<ResourceAddress>::new(vec![
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                    runner.create_fungible_resource(10000.into(), 18, acc.2),
                ]);
                let non_fungibles = Container::<ResourceAddress>::new(vec![
                    runner.create_non_fungible_resource(acc.2),
                    runner.create_non_fungible_resource(acc.2),
                ]);
                println!("addr = {:?}", acc.2);
                Account {
                    public_key: acc.0,
                    //_private_key: acc.1,
                    address: acc.2,
                    fungibles,
                    non_fungibles,
                }
            })
            .collect();
        let snapshot = runner.get_snapshot();
        let rng = ChaCha8Rng::seed_from_u64(1234);

        Self {
            rng,
            runner,
            snapshot,
            accounts,
        }
    }

    fn reset_runner(&mut self) {
        self.runner.restore_snapshot(self.snapshot.clone());
    }

    fn get_random_string(&mut self, max_len: usize) -> String {
        let len = self.rng.gen_range(0usize..max_len);

        let rand_string: String = self
            .rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();
        rand_string
    }

    fn get_random_decimal(&mut self, max: Option<u128>) -> Decimal {
        let max = max.unwrap_or(10_000_000_000_000u128);
        let d = self.rng.gen_range(0u128..max);
        Decimal::from(d)
    }

    fn get_random_vault(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Option<NodeId> {
        let vaults = self
            .runner
            .get_component_vaults(component_address, resource_address);
        if vaults.len() > 0 {
            let idx = self.rng.gen_range(0usize..vaults.len());
            Some(vaults[idx])
        } else {
            None
        }
    }

    fn get_random_account(
        &mut self,
    ) -> (Account, ResourceAddress, ResourceAddress, ResourceAddress) {
        let account_idx = self.rng.gen_range(0usize..self.accounts.len());
        let mut account = self.accounts[account_idx].clone();

        let fungible_address = account.fungibles.get_random(false).unwrap();
        let non_fungible_address = account.non_fungibles.get_random(false).unwrap();
        let some_resource_address = match self.rng.gen_range(0..2) {
            0 => account.fungibles.get_random(false).unwrap(),
            1 => account.non_fungibles.get_random(false).unwrap(),
            _ => todo!(),
        };
        (
            account,
            fungible_address,
            non_fungible_address,
            some_resource_address,
        )
    }

    // return BTreeSet of NonFungibleLocalIds from the preallocated pool
    fn get_non_fungible_local_id(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Option<BTreeSet<NonFungibleLocalId>> {
        let vaults = self
            .runner
            .get_component_vaults(component_address, resource_address);

        for vault_id in vaults {
            if let output = self.runner.substate_db().get_substate(
                &vault_id,
                SysModuleId::ObjectState.into(),
                &NonFungibleVaultOffset::LiquidNonFungible.into(),
            ) {
                if !vault_id.is_internal_fungible_vault() {
                    return self.runner.inspect_non_fungible_vault(vault_id);
                }
            } else {
                return None;
            }
        }
        return None;
    }

    // pick account from the preallocated pool basing on the input data
    fn get_account_for_mutation(&mut self, data: &[u8]) -> Option<ComponentAddress> {
        let len = data.len();
        if len >= 2 && data[len - 2] % 2 == 0 {
            let idx = *data.last().unwrap() as usize % self.accounts.len();
            return Some(self.accounts[idx].address);
        }
        None
    }

    // pick resource from the preallocated pool basing on the input data
    fn get_resource_for_mutation(&mut self, data: &[u8]) -> Option<ResourceAddress> {
        let len = data.len();
        if len >= 2 && data[len - 2] % 2 == 0 {
            let idx = *data.last().unwrap() as usize % self.accounts.len();
            self.accounts[idx].fungibles.get_idx(idx, false)
        } else {
            None
        }
    }

    // Smartly replace some data in the manifest using some preallocated resources.
    // This is to let fuzzing go "deeper" into the manifest instructions and not to reject the
    // transaction on the very early stage
    #[cfg(feature = "smart_mutate")]
    fn smart_mutate_manifest(&mut self, manifest: &mut TransactionManifest) {
        for i in &mut manifest.instructions {
            match i {
                Instruction::CallMethod {
                    component_address, ..
                }
                | Instruction::SetComponentRoyaltyConfig {
                    component_address, ..
                }
                | Instruction::ClaimComponentRoyalty { component_address } => {
                    if let Some(address) =
                        self.get_account_for_mutation(&component_address.as_ref())
                    {
                        *component_address = address;
                    }
                }
                Instruction::TakeFromWorktop { resource_address }
                | Instruction::TakeFromWorktopByAmount {
                    resource_address, ..
                }
                | Instruction::TakeFromWorktopByIds {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContains { resource_address }
                | Instruction::AssertWorktopContainsByAmount {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContainsByIds {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZone { resource_address }
                | Instruction::CreateProofFromAuthZoneByAmount {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZoneByIds {
                    resource_address, ..
                }
                | Instruction::MintFungible {
                    resource_address, ..
                }
                | Instruction::MintNonFungible {
                    resource_address, ..
                }
                | Instruction::MintUuidNonFungible {
                    resource_address, ..
                } => {
                    if let Some(address) =
                        self.get_resource_for_mutation(&resource_address.as_ref())
                    {
                        *resource_address = address;
                    }
                }
                _ => {}
            }
        }
    }

    fn fuzz_tx_manifest(&mut self, data: &[u8]) -> TxStatus {
        let result = manifest_decode::<TransactionManifest>(data);
        match result {
            #[allow(unused_mut)]
            Ok(mut manifest) => {
                #[cfg(feature = "smart_mutate")]
                self.smart_mutate_manifest(&mut manifest);

                let receipt = self.runner.execute_manifest(
                    manifest,
                    vec![],
                    /*
                                        vec![NonFungibleGlobalId::from_public_key(
                                            &self.accounts[0].public_key,
                                        )],
                    */
                );
                if receipt.is_commit_success() {
                    TxStatus::CommitSuccess
                } else {
                    println!("commit failure receipt = {:?}", receipt);
                    receipt.expect_commit_success();
                    TxStatus::CommitFailure
                }
            }
            Err(_err) => TxStatus::DecodeError,
        }
    }
    /*
       using ast::Instrunction since not all instructions are present in Instruction:
           CreateAccessController
           CreateAccount
           CreateAccountAdvanced
           CreateFungibleResource
           CreateFungibleResourceWithInitialSupply
           CreateIdentity
           CreateIdentityAdvanced
           CreateNonFungibleResource
           CreateNonFungibleResourceWithInitialSupply
           CreateValidator
    */
    fn gen_tx_manifest(&mut self) -> TransactionManifest {
        let mut builder = ManifestBuilder::new();
        let instruction_count = self.rng.gen_range(0u32..4u32);
        let mut buckets = Container::<ManifestBucket>::new(vec![]);
        let mut proof_ids = Container::<ManifestProof>::new(vec![]);

        let (account, fungible_address, non_fungible_address, _) = self.get_random_account();
        let (_, _, _, resource_address) = self.get_random_account();
        let component_address = account.address;

        let non_fungible_ids = self
            .get_non_fungible_local_id(component_address, non_fungible_address)
            .unwrap_or(BTreeSet::new());

        // what about lock_fee
        let fee = Decimal::from(100);
        match self.rng.gen_range(0..1) {
            0 => {
                println!("fee = {} ", fee);
                builder.lock_fee(self.runner.faucet_component(), fee);
                //builder.lock_fee(component_address, fee);
            }
            1 => {
                let d = self.get_random_decimal(Some(100u128));
                println!("fee = {} d = {}", fee, d);
                builder.lock_fee_and_withdraw(component_address, fee, fungible_address, d);
            }
            2 => {
                println!("fee = {} ids = {:?}", fee, non_fungible_ids);
                builder.lock_fee_and_withdraw_non_fungibles(
                    component_address,
                    fee,
                    non_fungible_address,
                    non_fungible_ids,
                );
            }
            _ => todo!(),
        };
        /*
                builder.set_metadata(
                        non_fungible_address.into(),
                        self.get_random_string(10),
                        MetadataEntry::Value(MetadataValue::String(
                                self.get_random_string(10)))
                    );
        */
        let manifest = builder.build();
        return manifest;
        /*

                for _ in 0..instruction_count {
                    let next = self.rng.gen_range(0usize..ast::Instruction::COUNT);

                    let instruction = match next {
                        // AssertWorktopContains
                        0 => Some(Instruction::AssertWorktopContains {
                            resource_address: fungible_address,
                        }),
                        // AssertWorktopContainsByAmount
                        1 => {
                            let d = self.rng.gen_range(0u32..10_000_000u32);
                            let amount = Decimal::from(d);
                            Some(Instruction::AssertWorktopContainsByAmount {
                                amount,
                                resource_address: fungible_address,
                            })
                        }
                        // AssertWorktopContainsByIds
                        2 => {
                            let ids: BTreeSet<NonFungibleLocalId> = self
                                .get_non_fungible_local_id(component_address, non_fungible_address)
                                .unwrap_or(BTreeSet::new());

                            Some(Instruction::AssertWorktopContainsByIds {
                                ids,
                                resource_address: non_fungible_address,
                            })
                        }
                        // BurnResource
                        3 => buckets
                            .get_random(true)
                            .map(|bucket_id| Instruction::BurnResource { bucket_id }),
                        // CallFunction
                        4 => {
                            // TODO
                            None
                        }
                        // CallMethod
                        5 => {
                            // TODO
                            None
                        }
                        // ClaimComponentRoyalty
                        6 => Some(Instruction::ClaimComponentRoyalty { component_address }),
                        // ClaimPackageRoyalty
                        7 => {
                            // TODO - use other package address?
                            Some(Instruction::ClaimPackageRoyalty {
                                package_address: ACCOUNT_PACKAGE,
                            })
                        }
                        // ClearAuthZone
                        8 => Some(Instruction::ClearAuthZone),
                        // ClearSignatureProofs
                        9 => Some(Instruction::ClearSignatureProofs),
                        // CloneProof
                        10 => proof_ids
                            .get_random(false)
                            .map(|proof_id| Instruction::CloneProof {
                                proof_id: proof_id.clone(),
                            }),
                        // CreateAccessController
                        11 => buckets.get_random(true).map(|controlled_asset| {
                            let primary_role = AccessRule::AllowAll;
                            let recovery_role = AccessRule::AllowAll;
                            let confirmation_role = AccessRule::AllowAll;
                            let timed_recovery_delay_in_minutes = Some(self.rng.gen_range(0u32..1_000u32));

                            Instruction::CallFunction {
                                package_address: ACCESS_CONTROLLER_PACKAGE,
                                blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
                                function_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
                                args: manifest_args!(
                                    controlled_asset,
                                    RuleSet {
                                        primary_role,
                                        recovery_role,
                                        confirmation_role,
                                    },
                                    timed_recovery_delay_in_minutes
                                ),
                            }
                        }),
                        // CreateAccount
                        12 => Some(Instruction::CallFunction {
                            package_address: ACCOUNT_PACKAGE,
                            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                            function_name: ACCOUNT_CREATE_IDENT.to_string(),
                            args: to_manifest_value(&AccountCreateInput {}),
                        }),
                        // CreateAccountAdvanced
                        13 => {
                            let config = AccessRulesConfig::new()
                                .default(AccessRule::AllowAll, AccessRule::AllowAll);

                            Some(Instruction::CallFunction {
                                package_address: ACCOUNT_PACKAGE,
                                blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                                function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
                                args: to_manifest_value(&AccountCreateAdvancedInput { config }),
                            })
                        }
                        // CreateFungibleResource
                        14 => None,
                        // CreateFungibleResourceWithInitialSupply
                        15 => None,
                        // CreateIdentity
                        16 => None,
                        // CreateIdentityAdvanced
                        17 => None,
                        // CreateNonFungibleResource
                        18 => Some(Instruction::CallFunction {
                            package_address: RESOURCE_MANAGER_PACKAGE,
                            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                            args: to_manifest_value(&NonFungibleResourceManagerCreateInput {
                                id_type: NonFungibleIdType::Integer,
                                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                                access_rules: BTreeMap::from([
                                    (
                                        ResourceMethodAuthKey::Withdraw,
                                        (AccessRule::AllowAll, AccessRule::DenyAll),
                                    ),
                                    (
                                        ResourceMethodAuthKey::Deposit,
                                        (AccessRule::AllowAll, AccessRule::DenyAll),
                                    ),
                                ]),
                            }),
                        }),
                        // CreateNonFungibleResourceWithInitialSupply
                        19 => {
                            let mut entries = BTreeMap::new();
                            let entries_len = self.rng.gen_range(0usize..100usize);
                            for _i in 0..entries_len {
                                entries.insert(
                                    NonFungibleLocalId::integer(self.rng.gen_range(0u64..1000u64)),
                                    (to_manifest_value(&(
                                        self.get_random_string(1000),
                                        self.get_random_decimal(None),
                                    )),),
                                );
                            }
                            Some(Instruction::CallFunction {
                                package_address: RESOURCE_MANAGER_PACKAGE,
                                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                                function_name:
                                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                                        .to_string(),
                                args: to_manifest_value(
                                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                                        id_type: NonFungibleIdType::Integer,
                                        non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                                        metadata: BTreeMap::from([(
                                            "name".to_string(),
                                            "Token".to_string(),
                                        )]),
                                        access_rules: BTreeMap::from([
                                            (
                                                ResourceMethodAuthKey::Withdraw,
                                                (AccessRule::AllowAll, AccessRule::DenyAll),
                                            ),
                                            (
                                                ResourceMethodAuthKey::Deposit,
                                                (AccessRule::AllowAll, AccessRule::DenyAll),
                                            ),
                                        ]),
                                        entries,
                                    },
                                ),
                            })
                        }
                        // CreateProofFromAuthZone
                        20 => Some(Instruction::CreateProofFromAuthZone {
                            resource_address: fungible_address,
                        }),
                        // CreateProofFromAuthZoneByAmount
                        21 => Some(Instruction::CreateProofFromAuthZoneByAmount {
                            amount: self.get_random_decimal(None),
                            resource_address: fungible_address,
                        }),
                        // CreateProofFromAuthZoneByIds
                        22 => {
                            let ids: BTreeSet<NonFungibleLocalId> = self
                                .get_non_fungible_local_id(self.accounts[0].address, fungible_address)
                                .unwrap_or(BTreeSet::new());
                            Some(Instruction::CreateProofFromAuthZoneByIds {
                                ids,
                                resource_address: fungible_address,
                            })
                        }
                        // CreateProofFromBucket
                        23 => buckets
                            .get_random(false)
                            .map(|bucket_id| Instruction::CreateProofFromBucket { bucket_id }),
                        // CreateValidator
                        24 => Some(Instruction::CallMethod {
                            component_address: EPOCH_MANAGER,
                            method_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                            args: to_manifest_value(&EpochManagerCreateValidatorInput {
                                key: account.public_key,
                            }),
                        }),
                        // DropAllProofs
                        25 => Some(Instruction::DropAllProofs),
                        // DropProof
                        26 => proof_ids
                            .get_random(true)
                            .map(|proof_id| Instruction::DropProof { proof_id }),
                        // MintFungible
                        27 => Some(Instruction::MintFungible {
                            resource_address: fungible_address,
                            amount: self.get_random_decimal(None),
                        }),
                        // MintNonFungible
                        28 => {
                            let mut entries = BTreeMap::new();
                            let entries_len = self.rng.gen_range(0usize..100usize);
                            for _i in 0..entries_len {
                                entries.insert(
                                    NonFungibleLocalId::integer(self.rng.gen_range(0u64..1000u64)),
                                    (to_manifest_value(&(
                                        self.get_random_string(1000),
                                        self.get_random_decimal(None),
                                    )),),
                                );
                            }

                            Some(Instruction::MintNonFungible {
                                resource_address: non_fungible_address,
                                args: to_manifest_value(&NonFungibleResourceManagerMintManifestInput {
                                    entries,
                                }),
                            })
                        }
                        // MintUuidNonFungible
                        29 => None,
                        // PopFromAuthZone
                        30 => Some(Instruction::PopFromAuthZone {}),
                        // PublishPackage
                        31 => None,
                        // PublishPackageAdvanced
                        32 => None,
                        // PushToAuthZone
                        33 => proof_ids
                            .get_random(true)
                            .map(|proof_id| Instruction::PushToAuthZone { proof_id }),
                        // RecallResource
                        34 => self
                            .get_random_vault(component_address, resource_address)
                            .map(|vault_id| Instruction::RecallResource {
                                vault_id: LocalAddress::new_unchecked(vault_id.into()),
                                amount: self.get_random_decimal(None),
                            }),
                        // RemoveMetadata
                        35 => None,
                        // ReturnToWorktop
                        36 => buckets
                            .get_random(true)
                            .map(|bucket_id| Instruction::ReturnToWorktop { bucket_id }),
                        // SetComponentRoyaltyConfig
                        37 => {
                            let mut royalty_config = RoyaltyConfigBuilder::new();
                            let rules_len = self.rng.gen_range(0usize..100usize);
                            for _i in 0..rules_len {
                                royalty_config = royalty_config.add_rule(
                                    &self.get_random_string(1000),
                                    self.rng.gen_range(0u32..1_000_000_u32),
                                );
                            }
                            let royalty_config = royalty_config.default(1);

                            Some(Instruction::SetComponentRoyaltyConfig {
                                component_address,
                                royalty_config,
                            })
                        }
                        // SetMetadata
                        38 => Some(Instruction::SetMetadata {
                            entity_address: GlobalAddress::from(component_address),
                            key: self.get_random_string(1000),
                            value: MetadataEntry::Value(MetadataValue::String(
                                self.get_random_string(1000),
                            )),
                        }),
                        // SetMethodAccessRule
                        39 => Some(Instruction::SetMethodAccessRule {
                            entity_address: GlobalAddress::from(component_address),
                            key: MethodKey::new(
                                SysModuleId::ObjectState,
                                ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
                            ),
                            rule: AccessRule::AllowAll,
                        }),
                        // SetPackageRoyaltyConfig
                        40 => {
                            let mut royalty_config = RoyaltyConfigBuilder::new();
                            let rules_len = self.rng.gen_range(0usize..100usize);
                            for _i in 0..rules_len {
                                royalty_config = royalty_config.add_rule(
                                    &self.get_random_string(1000),
                                    self.rng.gen_range(0u32..1_000_000_u32),
                                );
                            }
                            let royalty_config =
                                BTreeMap::from([(self.get_random_string(1000), royalty_config.default(1))]);

                            Some(Instruction::SetPackageRoyaltyConfig {
                                package_address: RESOURCE_MANAGER_PACKAGE,
                                royalty_config,
                            })
                        }
                        // TakeFromWorktop
                        41 => Some(Instruction::TakeFromWorktop {
                            resource_address: fungible_address,
                        }),
                        // TakeFromWorktopByAmount
                        42 => {
                            let d = self.rng.gen_range(0u32..1_000u32);
                            let amount = Decimal::from(d);
                            Some(Instruction::TakeFromWorktopByAmount {
                                amount,
                                resource_address: fungible_address,
                            })
                        }
                        // TakeFromWorktopByIds
                        43 => {
                            let ids: BTreeSet<NonFungibleLocalId> = self
                                .get_non_fungible_local_id(component_address, non_fungible_address)
                                .unwrap_or(BTreeSet::new());

                            Some(Instruction::TakeFromWorktopByIds {
                                ids: ids.clone(),
                                resource_address: non_fungible_address,
                            })
                        }
                        _ => unreachable!(
                            "Not all instructions (current count is {}) covered by this match",
                            ast::Instruction::COUNT
                        ),
                    };

                    match instruction {
                        Some(instruction) => {
                            let (_, bucket_id, proof_id) = builder.add_instruction(instruction);
                            match bucket_id {
                                Some(bucket_id) => buckets.push(bucket_id),
                                None => {}
                            }
                            match proof_id {
                                Some(proof_id) => proof_ids.push(proof_id),
                                None => {}
                            }
                        }
                        None => {}
                    }
                }
                let manifest = builder.build();
                manifest
        */
    }
}

#[derive(Debug)]
pub enum TxStatus {
    // Transaction commit success
    CommitSuccess,
    // Transaction commit failure
    CommitFailure,
    // Transaction manifest parse error
    DecodeError,
}

#[cfg(feature = "dump_manifest_to_file")]
fn dump_manifest_to_file(m: &TransactionManifest) {
    let bytes = manifest_encode(m).unwrap();
    let m_hash = hash(&bytes);
    let path = format!("manifest_{:?}.raw", m_hash);
    std::fs::write(&path, bytes).unwrap();
    println!("manifest dumped to file {}", &path);
}

// This test generates random transactions manifests and tries to execute them.
// It stops when given number of successful transactions is reached.
#[test]
fn test_gen_tx_manifest() {
    let needed_good_cnt = 1;
    let mut i = 0;
    let mut curr_good_cnt = 0;

    let mut fuzzer = Fuzzer::new();
    while curr_good_cnt < needed_good_cnt {
        let m = fuzzer.gen_tx_manifest();
        if matches!(
            fuzzer.fuzz_tx_manifest(&manifest_encode(&m).unwrap()),
            TxStatus::CommitSuccess
        ) && m.instructions.len() >= 1
        {
            curr_good_cnt += 1;
            println!("good instructions={} {:?} ", m.instructions.len(), m);

            #[cfg(feature = "dump_manifest_to_file")]
            dump_manifest_to_file(&m);
        }
        i += 1;
        //println!("{}/{} instructions={} {:?}", i, needed_good_cnt, m.instructions.len(), m);
        println!(
            "{} {}/{} instructions={} ",
            i,
            curr_good_cnt,
            needed_good_cnt,
            m.instructions.len()
        );
    }
}

#[test]
fn test_call_method_with_all_resources_doesnt_drop_auth_zone_proofs() {
    // Arrange
    let mut fuzzer = Fuzzer::new();
    let account = &fuzzer.accounts[1];
    let (public_key, account) = (account.public_key, account.address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(fuzzer.runner.faucet_component(), dec!("10"))
        //        .create_proof_from_account(account, RADIX_TOKEN)
        /*
                .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
                    builder.push_to_auth_zone(proof_id)
                })
                .call_method(
                    account,
                    "deposit_batch",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
                    builder.push_to_auth_zone(proof_id)
                })
                .call_method(
                    account,
                    "deposit_batch",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
                    builder.push_to_auth_zone(proof_id)
                })
                .call_method(
                    account,
                    "deposit_batch",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
        */
        .build();
    let receipt = fuzzer.runner.execute_manifest(
        manifest,
        vec![],
        //        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}

#[test]
// This test verifies whether it is still possible to parse manifest raw files and execute them.
// If it fails with TxStatus::DecodeError then most likely that manifest format has changed and
// input files shall be recreated.
fn test_fuzz_tx() {
    let mut fuzzer = Fuzzer::new();
    let data = std::fs::read(
        "fuzz_input/transaction/manifest_01995e0d6005c34ad99fba993ebe1443ef55c4db71ed037de12afb3eb28bbfae.raw",
    )
    .unwrap();
    assert!(matches!(
        fuzzer.fuzz_tx_manifest(&data),
        TxStatus::CommitSuccess
    ));

    let data = std::fs::read(
        "fuzz_input/transaction//manifest_0113970c0a72935c8c27ddd97a9396d1839f0173bf9ed091f9706aa61db8417e.raw",
    )
    .unwrap();
    assert!(matches!(
        fuzzer.fuzz_tx_manifest(&data),
        TxStatus::CommitFailure
    ));
}

// Initialize static objects outside the fuzzing loop to assure deterministic instrumentation
// output across runs.
fn init_statics() {
    // Following code initializes secp256k1::SECP256K1 global static context
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(100).unwrap();
    let _public_key = private_key.public_key();
}

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    unsafe {
        static mut FUZZER: Lazy<Fuzzer> = Lazy::new(|| Fuzzer::new());

        FUZZER.reset_runner();
        FUZZER.fuzz_tx_manifest(data);
    }
});

#[cfg(feature = "afl")]
fn main() {
    init_statics();

    // fuzz! uses `catch_unwind` and it requires RefUnwindSafe trait, which is not auto-implemented by
    // Fuzzer members (TestRunner mainly). `AssertUnwindSafe` annotates the variable is indeed
    // unwind safe
    let mut fuzzer = AssertUnwindSafe(Fuzzer::new());

    fuzz!(|data: &[u8]| {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    init_statics();

    let mut fuzzer = Fuzzer::new();

    simple_fuzzer::fuzz(|data: &[u8]| -> TxStatus {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data)
    });
}
