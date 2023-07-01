use crate::internal_prelude::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::ResourceAction::{Burn, Mint};
use radix_engine_interface::blueprints::resource::*;

/// Utility for building transaction manifest.
pub struct ManifestBuilderV2 {
    registrar: ManifestNameRegistrar,
    /// Instructions generated.
    instructions: Vec<InstructionV1>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

impl ManifestBuilderV2 {
    /// Starts a new transaction builder. Returns a namer and a builder
    pub fn new() -> (ManifestNamer, Self) {
        let (namer, registrar) = ManifestNamer::new();
        let builder = Self {
            registrar,
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
        };
        (namer, builder)
    }

    pub fn add_blob(&mut self, blob: Vec<u8>) -> ManifestBlobRef {
        let hash = hash(&blob);
        self.blobs.insert(hash, blob);
        ManifestBlobRef(hash.0)
    }

    fn add_instruction(mut self, instruction: InstructionV1) -> Self {
        self.instructions.push(instruction);
        self
    }

    /// Takes resource from worktop.
    pub fn take_all_from_worktop(
        self,
        resource_address: ResourceAddress,
        new_bucket: NewManifestBucket,
    ) -> Self {
        self.registrar.register_bucket(new_bucket);
        self.add_instruction(InstructionV1::TakeAllFromWorktop { resource_address })
    }

    /// Takes resource from worktop, by amount.
    pub fn take_from_worktop(
        self,
        resource_address: ResourceAddress,
        amount: Decimal,
        new_bucket: NewManifestBucket,
    ) -> Self {
        self.registrar.register_bucket(new_bucket);
        self.add_instruction(InstructionV1::TakeFromWorktop {
            amount,
            resource_address,
        })
    }

    /// Takes resource from worktop, by non-fungible ids.
    pub fn take_non_fungibles_from_worktop(
        self,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        new_bucket: NewManifestBucket,
    ) -> Self {
        self.registrar.register_bucket(new_bucket);
        self.add_instruction(InstructionV1::TakeNonFungiblesFromWorktop {
            ids: ids.into_iter().collect(),
            resource_address,
        })
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(self, bucket: ManifestBucket) -> Self {
        self.registrar.consume_bucket(bucket);
        self.add_instruction(InstructionV1::ReturnToWorktop { bucket_id: bucket })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(
        self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Self {
        self.add_instruction(InstructionV1::AssertWorktopContains {
            amount,
            resource_address,
        })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_non_fungibles(
        self,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        self.add_instruction(InstructionV1::AssertWorktopContainsNonFungibles {
            ids: ids.clone().into_iter().collect(),
            resource_address,
        })
    }

    /// Pops the most recent proof from auth zone.
    pub fn pop_from_auth_zone<F>(self, new_proof: NewManifestProof) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::PopFromAuthZone {})
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(self, proof: ManifestProof) -> Self {
        self.registrar.consume_proof(proof);
        self.add_instruction(InstructionV1::PushToAuthZone { proof_id: proof })
    }

    /// Clears the auth zone.
    pub fn clear_auth_zone(self) -> Self {
        self.add_instruction(InstructionV1::ClearAuthZone)
    }

    /// Creates proof from the auth zone.
    pub fn create_proof_from_auth_zone<F>(
        self,
        resource_address: ResourceAddress,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromAuthZone { resource_address })
    }

    /// Creates proof from the auth zone by amount.
    pub fn create_proof_from_auth_zone_of_amount<F>(
        self,
        resource_address: ResourceAddress,
        amount: Decimal,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfAmount {
            amount,
            resource_address,
        })
    }

    /// Creates proof from the auth zone by non-fungible ids.
    pub fn create_proof_from_auth_zone_of_non_fungibles<F>(
        self,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
            ids: ids.clone().into_iter().collect(),
            resource_address,
        })
    }

    /// Creates proof from the auth zone
    pub fn create_proof_from_auth_zone_of_all<F>(
        self,
        resource_address: ResourceAddress,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfAll { resource_address })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket<F>(
        self,
        bucket: ManifestBucket,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromBucket { bucket_id: bucket })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_amount<F>(
        self,
        bucket: ManifestBucket,
        amount: Decimal,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfAmount {
            bucket_id: bucket,
            amount,
        })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_non_fungibles<F>(
        self,
        bucket: ManifestBucket,
        ids: BTreeSet<NonFungibleLocalId>,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfNonFungibles {
            bucket_id: bucket,
            ids: ids.into_iter().collect(),
        })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_all<F>(
        self,
        bucket: ManifestBucket,
        new_proof: NewManifestProof,
    ) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfAll { bucket_id: bucket })
    }

    /// Clones a proof.
    pub fn clone_proof<F>(self, proof: ManifestProof, new_proof: NewManifestProof) -> Self {
        self.registrar.register_proof(new_proof);
        self.add_instruction(InstructionV1::CloneProof { proof_id: proof })
    }

    pub fn allocate_global_address<F>(
        self,
        blueprint_id: BlueprintId,
        new_address_reservation: NewManifestAddressReservation,
        new_address: NewManifestNamedAddress,
    ) -> Self {
        self.registrar
            .register_address_reservation(new_address_reservation);
        self.registrar.register_named_address(new_address);
        self.add_instruction(InstructionV1::AllocateGlobalAddress {
            package_address: blueprint_id.package_address,
            blueprint_name: blueprint_id.blueprint_name,
        })
    }

    /// Drops a proof.
    pub fn drop_proof(self, proof: ManifestProof) -> Self {
        self.registrar.consume_proof(proof);
        self.add_instruction(InstructionV1::DropProof { proof_id: proof })
    }

    /// Drops all proofs.
    pub fn drop_all_proofs(self) -> Self {
        self.registrar.consume_all_proofs();
        self.add_instruction(InstructionV1::DropAllProofs)
    }

    /// Drops all virtual proofs.
    pub fn clear_signature_proofs(self) -> Self {
        self.add_instruction(InstructionV1::ClearSignatureProofs)
    }

    /// Creates a fungible resource
    pub fn create_fungible_resource<R: Into<AccessRule>>(
        self,
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        metadata: ModuleConfig<MetadataInit>,
        access_rules: BTreeMap<ResourceAction, (AccessRule, R)>,
        initial_supply: Option<Decimal>,
    ) -> Self {
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();
        let instruction = if let Some(initial_supply) = initial_supply {
            InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role,
                        divisibility,
                        track_total_supply,
                        metadata,
                        access_rules,
                        initial_supply,
                        address_reservation: None,
                    }
                ),
            }
        } else {
            InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&FungibleResourceManagerCreateManifestInput {
                    owner_role,
                    divisibility,
                    track_total_supply,
                    metadata,
                    access_rules,
                    address_reservation: None,
                }),
            }
        };
        self.add_instruction(instruction)
    }

    /// Creates a new non-fungible resource
    pub fn create_non_fungible_resource<R, T, V>(
        self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        metadata: ModuleConfig<MetadataInit>,
        access_rules: BTreeMap<ResourceAction, (AccessRule, R)>,
        initial_supply: Option<T>,
    ) -> Self
    where
        R: Into<AccessRule>,
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode + NonFungibleData,
    {
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();

        let instruction = if let Some(initial_supply) = initial_supply {
            let entries = initial_supply
                .into_iter()
                .map(|(id, e)| (id, (to_manifest_value_and_unwrap!(&e),)))
                .collect();

            InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role,
                        id_type,
                        track_total_supply,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        metadata,
                        access_rules,
                        entries,
                        address_reservation: None,
                    }
                ),
            }
        } else {
            InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateManifestInput {
                        owner_role,
                        id_type,
                        track_total_supply,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        access_rules,
                        metadata,
                        address_reservation: None,
                    }
                ),
            }
        };

        self.add_instruction(instruction)
    }

    pub fn create_identity_advanced(self, owner_rule: OwnerRole) -> Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateAdvancedInput { owner_rule }),
        })
    }

    pub fn create_identity(self) -> Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateInput {}),
        })
    }

    pub fn create_validator(
        self,
        key: Secp256k1PublicKey,
        fee_factor: Decimal,
        xrd_payment: ManifestBucket,
    ) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: manifest_args!(key, fee_factor, xrd_payment),
        })
    }

    pub fn register_validator(self, validator_address: ComponentAddress) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_REGISTER_IDENT.to_string(),
            args: manifest_args!(),
        })
    }

    pub fn unregister_validator(self, validator_address: ComponentAddress) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_UNREGISTER_IDENT.to_string(),
            args: manifest_args!(),
        })
    }

    pub fn signal_protocol_update_readiness(
        self,
        validator_address: ComponentAddress,
        protocol_version_name: &str,
    ) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS.to_string(),
            args: manifest_args!(protocol_version_name.to_string()),
        })
    }

    pub fn stake_validator_as_owner(
        self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    pub fn stake_validator(
        self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_STAKE_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    pub fn unstake_validator(
        self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_UNSTAKE_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    pub fn claim_xrd(self, validator_address: ComponentAddress, bucket: ManifestBucket) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function<A: TryInto<DynamicPackageAddress, Error = E>, E: Debug>(
        self,
        package_address: A,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        args: ManifestValue,
    ) -> Self {
        let package_address = package_address
            .try_into()
            .expect("Package address was not valid");
        self.registrar.check_address_exists(package_address);
        self.add_instruction(InstructionV1::CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args: to_manifest_value_and_unwrap!(&args),
        })
    }

    /// Calls a scrypto method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug>(
        self,
        address: A,
        method_name: impl Into<String>,
        args: ManifestValue,
    ) -> Self {
        let address = address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallMethod {
            address,
            method_name: method_name.into(),
            args,
        })
    }

    pub fn claim_package_royalties<A: TryInto<DynamicPackageAddress, Error = E>, E: Debug>(
        self,
        package_address: A,
    ) -> Self {
        let address = package_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackageClaimRoyaltiesInput {}),
        })
    }

    pub fn set_component_royalty<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        component_address: A,
        method: impl Into<String>,
        amount: RoyaltyAmount,
    ) -> Self {
        let address = component_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentSetRoyaltyInput {
                method: method.into(),
                amount,
            }),
        })
    }

    pub fn lock_component_royalty<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        component_address: A,
        method: impl Into<String>,
    ) -> Self {
        let address = component_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentLockRoyaltyInput {
                method: method.into(),
            }),
        })
    }

    pub fn claim_component_royalties(self, component_address: ComponentAddress) -> Self {
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: component_address.into(),
            method_name: COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentClaimRoyaltiesInput {}),
        })
    }

    pub fn set_owner_role<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug>(
        self,
        address: A,
        rule: AccessRule,
    ) -> Self {
        let address = address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_SET_OWNER_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesSetOwnerRoleInput { rule }),
        })
    }

    pub fn update_role<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug>(
        self,
        address: A,
        module: ObjectModuleId,
        role_key: RoleKey,
        rule: AccessRule,
    ) -> Self {
        let address = address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_SET_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesSetRoleInput {
                module,
                role_key,
                rule,
            }),
        })
    }

    pub fn lock_role<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug>(
        self,
        address: A,
        module: ObjectModuleId,
        role_key: RoleKey,
    ) -> Self {
        let address = address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_LOCK_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesLockRoleInput { module, role_key }),
        })
    }

    pub fn set_metadata<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug>(
        self,
        address: A,
        key: impl Into<String>,
        value: MetadataValue,
    ) -> Self {
        let address = address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_SET_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataSetInput {
                key: key.into(),
                value
            }),
        })
    }

    pub fn freeze_metadata<A: TryInto<DynamicGlobalAddress, Error = E>, E: Debug>(
        self,
        address: A,
        key: impl Into<String>,
    ) -> Self {
        let address = address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_LOCK_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataLockInput { key: key.into() }),
        })
    }

    /// Publishes a package.
    pub fn publish_package_advanced(
        mut self,
        address_reservation: Option<ManifestAddressReservation>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: impl Into<MetadataInit>,
        owner_role: OwnerRole,
    ) -> Self {
        if let Some(consumed) = address_reservation.clone() {
            self.registrar.consume_address_reservation(consumed);
        }
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                code: ManifestBlobRef(code_hash.0),
                setup: definition,
                metadata: metadata.into(),
                package_address: address_reservation,
                owner_role,
            }),
        })
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package(mut self, code: Vec<u8>, definition: PackageDefinition) -> Self {
        let code_blob_ref = self.add_blob(code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmManifestInput {
                code: code_blob_ref,
                setup: definition,
                metadata: metadata_init!(),
            }),
        })
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package_with_owner(
        mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        owner_badge: NonFungibleGlobalId,
    ) -> Self {
        let code_blob_ref = self.add_blob(code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                package_address: None,
                code: code_blob_ref,
                setup: definition,
                metadata: metadata_init!(),
                owner_role: OwnerRole::Fixed(rule!(require(owner_badge.clone()))),
            }),
        })
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        minter_rule: AccessRule,
    ) -> Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(owner_role, true, 18, metadata, access_rules, initial_supply)
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Decimal,
    ) -> Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(
            owner_role,
            true,
            18,
            metadata,
            access_rules,
            Some(initial_supply),
        )
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        minter_rule: AccessRule,
    ) -> Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(owner_role, false, 0, metadata, access_rules, initial_supply)
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Decimal,
    ) -> Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(
            owner_role,
            false,
            0,
            metadata,
            access_rules,
            Some(initial_supply),
        )
    }

    pub fn burn_bucket(self, bucket: ManifestBucket) -> Self {
        self.registrar.consume_bucket(bucket);
        self.add_instruction(InstructionV1::BurnResource { bucket_id: bucket })
    }

    pub fn burn_from_worktop(self, amount: Decimal, resource_address: ResourceAddress) -> Self {
        let (new_bucket, bucket) = self.registrar.new_named_bucket_pair("to_burn");
        self.take_from_worktop(resource_address, amount, new_bucket)
            .burn_bucket(bucket)
    }

    pub fn burn_all_from_worktop(self, resource_address: ResourceAddress) -> Self {
        let (new_bucket, bucket) = self.registrar.new_named_bucket_pair("to_burn");
        self.take_all_from_worktop(resource_address, new_bucket)
            .burn_bucket(bucket)
    }

    pub fn burn_non_fungible_from_worktop(
        self,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let ids = btreeset!(non_fungible_global_id.local_id().clone());
        let resource_address = non_fungible_global_id.resource_address().clone();
        let (new_bucket, bucket) = self.registrar.new_named_bucket_pair("to_burn");

        self.take_non_fungibles_from_worktop(resource_address, ids, new_bucket)
            .burn_bucket(bucket)
    }

    pub fn mint_fungible<A: TryInto<DynamicResourceAddress, Error = E>, E: Debug>(
        self,
        resource_address: A,
        amount: Decimal,
    ) -> Self {
        let address = resource_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&FungibleResourceManagerMintInput { amount }),
        })
    }

    pub fn mint_non_fungible<A, E, T, V>(self, resource_address: A, entries: T) -> Self
    where
        A: TryInto<DynamicResourceAddress, Error = E>,
        E: Debug,
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode,
    {
        let address = resource_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let entries = entries
            .into_iter()
            .map(|(id, e)| (id, (to_manifest_value_and_unwrap!(&e),)))
            .collect();

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintManifestInput {
                entries
            }),
        })
    }

    pub fn mint_ruid_non_fungible<A, E, T, V>(self, resource_address: A, entries: T) -> Self
    where
        A: TryInto<DynamicResourceAddress, Error = E>,
        E: Debug,
        T: IntoIterator<Item = V>,
        V: ManifestEncode,
    {
        let address = resource_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let entries = entries
            .into_iter()
            .map(|e| (to_manifest_value_and_unwrap!(&e),))
            .collect();

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintRuidManifestInput {
                entries
            }),
        })
    }

    pub fn recall(self, vault_id: InternalAddress, amount: Decimal) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_RECALL_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultRecallInput { amount }),
        })
    }

    pub fn freeze_withdraw(self, vault_id: InternalAddress) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::WITHDRAW,
            }),
        })
    }

    pub fn unfreeze_withdraw(self, vault_id: InternalAddress) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::WITHDRAW,
            }),
        })
    }

    pub fn freeze_deposit(self, vault_id: InternalAddress) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::DEPOSIT,
            }),
        })
    }

    pub fn unfreeze_deposit(self, vault_id: InternalAddress) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::DEPOSIT,
            }),
        })
    }

    pub fn freeze_burn(self, vault_id: InternalAddress) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::BURN,
            }),
        })
    }

    pub fn unfreeze_burn(self, vault_id: InternalAddress) -> Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::BURN,
            }),
        })
    }

    /// Creates an account.
    pub fn new_account_advanced(self, owner_role: OwnerRole) -> Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccountCreateAdvancedInput { owner_role }),
        })
    }

    pub fn lock_fee_and_withdraw<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawInput {
            resource_address,
            amount,
            amount_to_lock,
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            args,
        })
    }

    pub fn lock_fee_and_withdraw_non_fungibles<
        A: TryInto<DynamicComponentAddress, Error = E>,
        E: Debug,
    >(
        self,
        account_address: A,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawNonFungiblesInput {
            amount_to_lock,
            resource_address,
            ids,
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        amount: Decimal,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            args,
        })
    }

    pub fn lock_contingent_fee<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        amount: Decimal,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountLockContingentFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            args,
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountWithdrawInput {
            resource_address,
            amount,
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_WITHDRAW_IDENT.to_string(),
            args,
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw_non_fungibles_from_account<
        A: TryInto<DynamicComponentAddress, Error = E>,
        E: Debug,
    >(
        self,
        account_address: A,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountWithdrawNonFungiblesInput {
            ids: ids.clone(),
            resource_address,
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    /// Withdraws resource from an account.
    pub fn burn_in_account<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountBurnInput {
            resource_address,
            amount
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_BURN_IDENT.to_string(),
            args,
        })
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        resource_address: ResourceAddress,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountCreateProofInput { resource_address });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            args,
        })
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_amount<
        A: TryInto<DynamicComponentAddress, Error = E>,
        E: Debug,
    >(
        self,
        account_address: A,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountCreateProofOfAmountInput {
            resource_address,
            amount,
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            args,
        })
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_non_fungibles<
        A: TryInto<DynamicComponentAddress, Error = E>,
        E: Debug,
    >(
        self,
        account_address: A,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        let args = to_manifest_value_and_unwrap!(&AccountCreateProofOfNonFungiblesInput {
            resource_address,
            ids: ids.clone(),
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
    }

    pub fn deposit<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        bucket: ManifestBucket,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        self.registrar.consume_bucket(bucket);

        self.call_method(address, ACCOUNT_DEPOSIT_IDENT, manifest_args!(bucket))
    }

    pub fn deposit_batch<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        self.registrar.consume_all_buckets();

        self.call_method(
            address,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn try_deposit_or_abort<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        bucket: ManifestBucket,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        self.registrar.consume_bucket(bucket);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
            manifest_args!(bucket),
        )
    }

    pub fn try_deposit_batch_or_abort<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        self.registrar.consume_all_buckets();

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn try_deposit_or_refund<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
        bucket: ManifestBucket,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        self.registrar.consume_bucket(bucket);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
            manifest_args!(bucket),
        )
    }

    pub fn try_deposit_batch_or_refund<A: TryInto<DynamicComponentAddress, Error = E>, E: Debug>(
        self,
        account_address: A,
    ) -> Self {
        let address = account_address.try_into().expect("Address was not valid");
        self.registrar.check_address_exists(address);

        self.registrar.consume_all_buckets();

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn create_access_controller(
        self,
        controlled_asset: ManifestBucket,
        primary_role: AccessRule,
        recovery_role: AccessRule,
        confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> Self {
        self.registrar.consume_bucket(controlled_asset);
        self.add_instruction(InstructionV1::CallFunction {
            package_address: ACCESS_CONTROLLER_PACKAGE.into(),
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
        })
    }

    /// Builds a transaction manifest.
    pub fn build(self) -> TransactionManifestV1 {
        let manifest = TransactionManifestV1 {
            instructions: self.instructions,
            blobs: self.blobs,
        };
        #[cfg(feature = "dump_manifest_to_file")]
        {
            let bytes = manifest_encode(&manifest).unwrap();
            let manifest_hash = hash(&bytes);
            let path = format!("manifest_{:?}.raw", manifest_hash);
            std::fs::write(&path, bytes).unwrap();
            println!("manifest dumped to file {}", &path);
        }
        manifest
    }
}
