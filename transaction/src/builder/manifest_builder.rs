use radix_engine_interface::abi::*;
use radix_engine_interface::api::types::{GlobalAddress, VaultId};
use radix_engine_interface::constants::*;
use radix_engine_interface::crypto::{hash, EcdsaSecp256k1PublicKey, Hash};
use radix_engine_interface::data::types::*;
use radix_engine_interface::data::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use radix_engine_interface::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use crate::model::*;
use crate::validation::*;

/// Utility for building transaction manifest.
pub struct ManifestBuilder {
    /// ID validator for calculating transaction object id
    id_allocator: ManifestIdAllocator,
    /// Instructions generated.
    instructions: Vec<BasicInstruction>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

impl ManifestBuilder {
    /// Starts a new transaction builder.
    pub fn new() -> Self {
        Self {
            id_allocator: ManifestIdAllocator::new(),
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
        }
    }

    /// Adds a raw instruction.
    pub fn add_instruction(
        &mut self,
        inst: BasicInstruction,
    ) -> (&mut Self, Option<ManifestBucket>, Option<ManifestProof>) {
        let mut new_bucket_id: Option<ManifestBucket> = None;
        let mut new_proof_id: Option<ManifestProof> = None;

        match &inst {
            BasicInstruction::TakeFromWorktop { .. }
            | BasicInstruction::TakeFromWorktopByAmount { .. }
            | BasicInstruction::TakeFromWorktopByIds { .. } => {
                new_bucket_id = Some(self.id_allocator.new_bucket_id().unwrap());
            }
            BasicInstruction::PopFromAuthZone { .. }
            | BasicInstruction::CreateProofFromAuthZone { .. }
            | BasicInstruction::CreateProofFromAuthZoneByAmount { .. }
            | BasicInstruction::CreateProofFromAuthZoneByIds { .. }
            | BasicInstruction::CreateProofFromBucket { .. }
            | BasicInstruction::CloneProof { .. } => {
                new_proof_id = Some(self.id_allocator.new_proof_id().unwrap());
            }
            _ => {}
        }

        self.instructions.push(inst);

        (self, new_bucket_id, new_proof_id)
    }

    /// Takes resource from worktop.
    pub fn take_from_worktop<F>(&mut self, resource_address: ResourceAddress, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) =
            self.add_instruction(BasicInstruction::TakeFromWorktop { resource_address });
        then(builder, bucket_id.unwrap())
    }

    /// Takes resource from worktop, by amount.
    pub fn take_from_worktop_by_amount<F>(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) =
            self.add_instruction(BasicInstruction::TakeFromWorktopByAmount {
                amount,
                resource_address,
            });
        then(builder, bucket_id.unwrap())
    }

    /// Takes resource from worktop, by non-fungible ids.
    pub fn take_from_worktop_by_ids<F>(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) =
            self.add_instruction(BasicInstruction::TakeFromWorktopByIds {
                ids: ids.clone(),
                resource_address,
            });
        then(builder, bucket_id.unwrap())
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(&mut self, bucket_id: ManifestBucket) -> &mut Self {
        self.add_instruction(BasicInstruction::ReturnToWorktop { bucket_id })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(&mut self, resource_address: ResourceAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::AssertWorktopContains { resource_address })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        })
        .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::AssertWorktopContainsByIds {
            ids: ids.clone(),
            resource_address,
        })
        .0
    }

    /// Pops the most recent proof from auth zone.
    pub fn pop_from_auth_zone<F>(&mut self, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(BasicInstruction::PopFromAuthZone {});
        then(builder, proof_id.unwrap())
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(&mut self, proof_id: ManifestProof) -> &mut Self {
        self.add_instruction(BasicInstruction::PushToAuthZone { proof_id });
        self
    }

    /// Clears the auth zone.
    pub fn clear_auth_zone(&mut self) -> &mut Self {
        self.add_instruction(BasicInstruction::ClearAuthZone).0
    }

    /// Creates proof from the auth zone.
    pub fn create_proof_from_auth_zone<F>(
        &mut self,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(BasicInstruction::CreateProofFromAuthZone { resource_address });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone by amount.
    pub fn create_proof_from_auth_zone_by_amount<F>(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(BasicInstruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone by non-fungible ids.
    pub fn create_proof_from_auth_zone_by_ids<F>(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(BasicInstruction::CreateProofFromAuthZoneByIds {
                ids: ids.clone(),
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from a bucket.
    pub fn create_proof_from_bucket<F>(&mut self, bucket_id: &ManifestBucket, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(BasicInstruction::CreateProofFromBucket {
                bucket_id: bucket_id.clone(),
            });
        then(builder, proof_id.unwrap())
    }

    /// Clones a proof.
    pub fn clone_proof<F>(&mut self, proof_id: &ManifestProof, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(BasicInstruction::CloneProof {
            proof_id: proof_id.clone(),
        });
        then(builder, proof_id.unwrap())
    }

    /// Drops a proof.
    pub fn drop_proof(&mut self, proof_id: ManifestProof) -> &mut Self {
        self.add_instruction(BasicInstruction::DropProof { proof_id })
            .0
    }

    /// Drops all proofs.
    pub fn drop_all_proofs(&mut self) -> &mut Self {
        self.add_instruction(BasicInstruction::DropAllProofs).0
    }

    /// Creates a fungible resource
    pub fn create_fungible_resource<R: Into<AccessRule>>(
        &mut self,
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, R)>,
        initial_supply: Option<Decimal>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CreateFungibleResource {
            divisibility,
            metadata,
            access_rules: access_rules
                .into_iter()
                .map(|(k, v)| (k, (v.0, v.1.into())))
                .collect(),
            initial_supply: initial_supply,
        });

        self
    }

    /// Creates a fungible resource with an owner badge
    pub fn create_fungible_resource_with_owner(
        &mut self,
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        owner_badge: NonFungibleGlobalId,
        initial_supply: Option<Decimal>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CreateFungibleResourceWithOwner {
            divisibility,
            metadata,
            owner_badge,
            initial_supply,
        });
        self
    }

    /// Creates a new non-fungible resource
    pub fn create_non_fungible_resource<R, T, V>(
        &mut self,
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, R)>,
        initial_supply: Option<T>,
    ) -> &mut Self
    where
        R: Into<AccessRule>,
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: NonFungibleData,
    {
        let initial_supply = initial_supply.map(|entries| {
            entries
                .into_iter()
                .map(|(id, e)| (id, (e.immutable_data().unwrap(), e.mutable_data().unwrap())))
                .collect()
        });
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();
        self.add_instruction(BasicInstruction::CreateNonFungibleResource {
            id_type,
            metadata,
            access_rules,
            initial_supply,
        });
        self
    }

    /// Creates a new non-fungible resource with an owner badge
    pub fn create_non_fungible_resource_with_owner<T, V>(
        &mut self,
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        owner_badge: NonFungibleGlobalId,
        initial_supply: Option<T>,
    ) -> &mut Self
    where
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: NonFungibleData,
    {
        let initial_supply = initial_supply.map(|entries| {
            entries
                .into_iter()
                .map(|(id, e)| (id, (e.immutable_data().unwrap(), e.mutable_data().unwrap())))
                .collect()
        });
        self.add_instruction(BasicInstruction::CreateNonFungibleResourceWithOwner {
            id_type,
            metadata,
            owner_badge,
            initial_supply,
        });
        self
    }

    pub fn create_identity(&mut self, access_rule: AccessRule) -> &mut Self {
        self.add_instruction(BasicInstruction::CreateIdentity { access_rule });
        self
    }

    pub fn create_validator(
        &mut self,
        key: EcdsaSecp256k1PublicKey,
        owner_access_rule: AccessRule,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CreateValidator {
            key,
            owner_access_rule,
        });
        self
    }

    pub fn register_validator(&mut self, validator_address: ComponentAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: validator_address,
            method_name: "register".to_string(),
            args: args!(),
        });
        self
    }

    pub fn unregister_validator(&mut self, validator_address: ComponentAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: validator_address,
            method_name: "unregister".to_string(),
            args: args!(),
        });
        self
    }

    pub fn stake_validator(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: validator_address,
            method_name: "stake".to_string(),
            args: args!(bucket),
        });
        self
    }

    pub fn unstake_validator(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: validator_address,
            method_name: "unstake".to_string(),
            args: args!(bucket),
        });
        self
    }

    pub fn claim_xrd(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: validator_address,
            method_name: "claim_xrd".to_string(),
            args: args!(bucket),
        });
        self
    }

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallFunction {
            package_address,
            blueprint_name: blueprint_name.to_string(),
            function_name: function_name.to_string(),
            args,
        });
        self
    }

    /// Calls a scrypto method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        method_name: &str,
        args: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address,
            method_name: method_name.to_owned(),
            args,
        });
        self
    }

    pub fn set_package_royalty_config(
        &mut self,
        package_address: PackageAddress,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetPackageRoyaltyConfig {
            package_address,
            royalty_config,
        })
        .0
    }

    pub fn set_component_royalty_config(
        &mut self,
        component_address: ComponentAddress,
        royalty_config: RoyaltyConfig,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        })
        .0
    }

    pub fn claim_package_royalty(&mut self, package_address: PackageAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::ClaimPackageRoyalty { package_address })
            .0
    }

    pub fn claim_component_royalty(&mut self, component_address: ComponentAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::ClaimComponentRoyalty { component_address })
            .0
    }

    pub fn set_method_access_rule(
        &mut self,
        entity_address: GlobalAddress,
        index: u32,
        key: AccessRuleKey,
        rule: AccessRule,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetMethodAccessRule {
            entity_address,
            index,
            key,
            rule,
        })
        .0
    }

    pub fn set_metadata(
        &mut self,
        entity_address: GlobalAddress,
        key: String,
        value: String,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetMetadata {
            entity_address,
            key,
            value,
        })
        .0
    }

    /// Publishes a package.
    pub fn publish_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRules,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let abi = scrypto_encode(&abi).unwrap();
        let abi_hash = hash(&abi);
        self.blobs.insert(abi_hash, abi);

        self.add_instruction(BasicInstruction::PublishPackage {
            code: ManifestBlobRef(code_hash),
            abi: ManifestBlobRef(abi_hash),
            royalty_config,
            metadata,
            access_rules,
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        owner_badge: NonFungibleGlobalId,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let abi = scrypto_encode(&abi).unwrap();
        let abi_hash = hash(&abi);
        self.blobs.insert(abi_hash, abi);

        self.add_instruction(BasicInstruction::PublishPackageWithOwner {
            code: ManifestBlobRef(code_hash),
            abi: ManifestBlobRef(abi_hash),
            owner_badge,
        });
        self
    }

    /// Builds a transaction manifest.
    /// TODO: consider using self
    pub fn build(&self) -> TransactionManifest {
        TransactionManifest {
            instructions: self.instructions.clone(),
            blobs: self.blobs.values().cloned().collect(),
        }
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: BTreeMap<String, String>,
        minter_rule: AccessRule,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(18, metadata, access_rules, initial_supply)
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        &mut self,
        metadata: BTreeMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(18, metadata, access_rules, Some(initial_supply))
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        &mut self,
        metadata: BTreeMap<String, String>,
        minter_rule: AccessRule,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(0, metadata, access_rules, initial_supply)
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        &mut self,
        metadata: BTreeMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(0, metadata, access_rules, Some(initial_supply))
    }

    pub fn burn(&mut self, amount: Decimal, resource_address: ResourceAddress) -> &mut Self {
        self.take_from_worktop_by_amount(amount, resource_address, |builder, bucket_id| {
            builder
                .add_instruction(BasicInstruction::BurnResource { bucket_id })
                .0
        })
    }

    pub fn mint_fungible(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::MintFungible {
            resource_address,
            amount,
        });
        self
    }

    pub fn mint_non_fungible<T, V>(
        &mut self,
        resource_address: ResourceAddress,
        entries: T,
    ) -> &mut Self
    where
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: NonFungibleData,
    {
        let entries = entries
            .into_iter()
            .map(|(id, e)| (id, (e.immutable_data().unwrap(), e.mutable_data().unwrap())))
            .collect();
        self.add_instruction(BasicInstruction::MintNonFungible {
            resource_address,
            entries,
        });
        self
    }

    pub fn mint_uuid_non_fungible<T, V>(
        &mut self,
        resource_address: ResourceAddress,
        entries: T,
    ) -> &mut Self
    where
        T: IntoIterator<Item = V>,
        V: NonFungibleData,
    {
        let entries = entries
            .into_iter()
            .map(|e| (e.immutable_data().unwrap(), e.mutable_data().unwrap()))
            .collect();
        self.add_instruction(BasicInstruction::MintUuidNonFungible {
            resource_address,
            entries,
        });
        self
    }

    pub fn recall(&mut self, vault_id: VaultId, amount: Decimal) -> &mut Self {
        self.add_instruction(BasicInstruction::RecallResource { vault_id, amount });
        self
    }

    pub fn burn_non_fungible(&mut self, non_fungible_global_id: NonFungibleGlobalId) -> &mut Self {
        let mut ids = BTreeSet::new();
        ids.insert(non_fungible_global_id.local_id().clone());
        self.take_from_worktop_by_ids(
            &ids,
            non_fungible_global_id.resource_address().clone(),
            |builder, bucket_id| {
                builder
                    .add_instruction(BasicInstruction::BurnResource { bucket_id })
                    .0
            },
        )
    }

    /// Creates an account.
    pub fn new_account(&mut self, withdraw_auth: &AccessRuleNode) -> &mut Self {
        self.add_instruction(BasicInstruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_owned(),
            function_name: "new".to_string(),
            args: args!(withdraw_auth.clone()),
        })
        .0
    }

    /// Creates an account with some initial resource.
    pub fn new_account_with_resource(
        &mut self,
        withdraw_auth: &AccessRule,
        bucket_id: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_owned(),
            function_name: "new_with_resource".to_string(),
            args: args!(withdraw_auth.clone(), bucket_id),
        })
        .0
    }

    pub fn lock_fee_and_withdraw(
        &mut self,
        account: ComponentAddress,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee_and_withdraw".to_string(),
            args: args!(amount, resource_address),
        })
        .0
    }

    pub fn lock_fee_and_withdraw_by_amount(
        &mut self,
        account: ComponentAddress,
        amount_to_lock: Decimal,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee_and_withdraw_by_amount".to_string(),

            args: args!(amount_to_lock, amount, resource_address),
        })
        .0
    }

    pub fn lock_fee_and_withdraw_by_ids(
        &mut self,
        account: ComponentAddress,
        amount_to_lock: Decimal,
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee_and_withdraw_by_ids".to_string(),

            args: args!(amount_to_lock, ids, resource_address),
        })
        .0
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee".to_string(),

            args: args!(amount),
        })
        .0
    }

    pub fn lock_contingent_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_contingent_fee".to_string(),

            args: args!(amount),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "withdraw".to_string(),

            args: args!(resource_address),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account_by_amount(
        &mut self,
        account: ComponentAddress,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "withdraw_by_amount".to_string(),

            args: args!(amount, resource_address),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account_by_ids(
        &mut self,
        account: ComponentAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "withdraw_by_ids".to_string(),

            args: args!(ids.clone(), resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "create_proof".to_string(),

            args: args!(resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_amount(
        &mut self,
        account: ComponentAddress,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "create_proof_by_amount".to_string(),

            args: args!(amount, resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_ids(
        &mut self,
        account: ComponentAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "create_proof_by_ids".to_string(),

            args: args!(ids.clone(), resource_address),
        })
        .0
    }

    pub fn create_access_controller(
        &mut self,
        controlled_asset: ManifestBucket,
        primary_role: AccessRule,
        recovery_role: AccessRule,
        confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CreateAccessController {
            controlled_asset,
            primary_role,
            recovery_role,
            confirmation_role,
            timed_recovery_delay_in_minutes,
        })
        .0
    }

    pub fn assert_access_rule(&mut self, access_rule: AccessRule) -> &mut Self {
        self.add_instruction(BasicInstruction::AssertAccessRule { access_rule })
            .0
    }

    pub fn borrow_mut<F, E>(&mut self, handler: F) -> Result<&mut Self, E>
    where
        F: FnOnce(&mut Self) -> Result<&mut Self, E>,
    {
        handler(self)
    }
}
