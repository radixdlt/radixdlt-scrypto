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
use radix_engine_interface::blueprints::resource::*;

/// Utility for building transaction manifest.
pub struct ManifestBuilder {
    registrar: ManifestNameRegistrar,
    /// Instructions generated.
    instructions: Vec<InstructionV1>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

pub struct NewSymbols {
    pub new_bucket: Option<ManifestBucket>,
    pub new_proof: Option<ManifestProof>,
    pub new_address_reservation: Option<ManifestAddressReservation>,
    pub new_address_id: Option<u32>,
}

/// A manifest builder - making use of a paired namer.
/// The namer is for creating / using buckets, proofs, address reservations
/// and named addresses. EG:
/// ```
/// # use transaction::prelude::*;
/// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let (builder, namer) = ManifestBuilder::new_with_namer();
/// let manifest = builder
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), namer.new_bucket("xrd"))
///     .try_deposit_or_abort(to_account_address, namer.bucket("xrd"))
///     .build();
/// ```
impl ManifestBuilder {
    /// Starts a new transaction builder, with paired namer.`
    pub fn new_with_namer() -> (Self, ManifestNamer) {
        let builder = Self::new();
        let namer = builder.namer();
        (builder, namer)
    }

    /// Starts a new transaction builder. Returns a builder, but no namer.
    /// You can later create a namer by calling `let namer = builder.namer();`
    pub fn new() -> Self {
        Self {
            registrar: ManifestNameRegistrar::new(),
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
        }
    }

    pub fn namer(&self) -> ManifestNamer {
        self.registrar.namer()
    }

    pub fn then(self, next: impl FnOnce(Self) -> Self) -> Self {
        next(self)
    }

    pub fn with_namer(self, next: impl FnOnce(Self, ManifestNamer) -> Self) -> Self {
        let namer = self.namer();
        next(self, namer)
    }

    pub fn with_bucket(
        self,
        bucket: impl ExistingManifestBucket,
        next: impl FnOnce(Self, ManifestBucket) -> Self,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        next(self, bucket)
    }

    /// This is intended to be called at the start, before the builder
    /// is used in a chained fashion, eg:
    /// ```
    /// # use transaction::prelude::*;
    /// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// # let package_address = FAUCET_PACKAGE; // Just so it compiles
    /// let (mut builder, namer) = ManifestBuilder::new();
    /// let code_blob_ref = builder.add_blob(vec![]);
    /// let manifest = builder
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     // ...
    ///     .call_function(
    ///         package_address,
    ///         "my_blueprint",
    ///         "func_name",
    ///         manifest_args!(code_blob_ref),
    ///     )
    ///     .build();
    /// ```
    pub fn add_blob(&mut self, blob: Vec<u8>) -> ManifestBlobRef {
        let hash = hash(&blob);
        self.blobs.insert(hash, blob);
        ManifestBlobRef(hash.0)
    }

    /// An internal method which is used by other methods - the callers are expected to handle
    /// registering buckets/proofs/etc and consuming them
    fn add_instruction(mut self, instruction: InstructionV1) -> Self {
        self.instructions.push(instruction);
        self
    }

    /// Only for use in advanced use cases.
    /// Returns all the created symbols as part of the instruction
    pub fn add_instruction_advanced(self, instruction: InstructionV1) -> (Self, NewSymbols) {
        let mut new_bucket = None;
        let mut new_proof = None;
        let mut new_address_reservation = None;
        let mut new_address_id = None;

        let namer = self.namer();

        match &instruction {
            InstructionV1::TakeAllFromWorktop { .. }
            | InstructionV1::TakeFromWorktop { .. }
            | InstructionV1::TakeNonFungiblesFromWorktop { .. } => {
                let (bucket_name, named_bucket) = namer.new_collision_free_bucket("bucket");
                self.registrar.register_bucket(named_bucket);
                new_bucket = Some(namer.bucket(bucket_name));
            }
            InstructionV1::PopFromAuthZone { .. }
            | InstructionV1::CreateProofFromAuthZone { .. }
            | InstructionV1::CreateProofFromAuthZoneOfAmount { .. }
            | InstructionV1::CreateProofFromAuthZoneOfNonFungibles { .. }
            | InstructionV1::CreateProofFromAuthZoneOfAll { .. }
            | InstructionV1::CreateProofFromBucket { .. }
            | InstructionV1::CreateProofFromBucketOfAmount { .. }
            | InstructionV1::CreateProofFromBucketOfNonFungibles { .. }
            | InstructionV1::CreateProofFromBucketOfAll { .. }
            | InstructionV1::CloneProof { .. } => {
                let (proof_name, named_proof) = namer.new_collision_free_proof("proof");
                self.registrar.register_proof(named_proof);
                new_proof = Some(namer.proof(proof_name));
            }
            InstructionV1::AllocateGlobalAddress { .. } => {
                let (reservation_name, named_reservation) =
                    namer.new_collision_free_address_reservation("reservation");
                self.registrar
                    .register_address_reservation(named_reservation);

                let (address_name, named_address) =
                    namer.new_collision_free_named_address("address");
                self.registrar.register_named_address(named_address);
                new_address_reservation = Some(namer.address_reservation(reservation_name));
                new_address_id = Some(namer.named_address_id(address_name));
            }
            _ => {}
        }

        (
            self.add_instruction(instruction),
            NewSymbols {
                new_bucket,
                new_proof,
                new_address_reservation,
                new_address_id,
            },
        )
    }

    /// Takes resource from worktop.
    pub fn take_all_from_worktop(
        self,
        resource_address: impl ResolvableResourceAddress,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_bucket.register(&self.registrar);
        self.add_instruction(InstructionV1::TakeAllFromWorktop { resource_address })
    }

    /// Takes resource from worktop, by amount.
    pub fn take_from_worktop(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        new_bucket.register(&self.registrar);
        self.add_instruction(InstructionV1::TakeFromWorktop {
            amount,
            resource_address,
        })
    }

    /// Takes resource from worktop, by non-fungible ids.
    pub fn take_non_fungibles_from_worktop(
        self,
        resource_address: impl ResolvableResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_bucket.register(&self.registrar);
        self.add_instruction(InstructionV1::TakeNonFungiblesFromWorktop {
            ids: ids.into_iter().collect(),
            resource_address,
        })
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(self, bucket: impl ExistingManifestBucket) -> Self {
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::ReturnToWorktop { bucket_id: bucket })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        self.add_instruction(InstructionV1::AssertWorktopContains {
            amount,
            resource_address,
        })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_any(
        self,
        resource_address: impl ResolvableResourceAddress,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        self.add_instruction(InstructionV1::AssertWorktopContainsAny { resource_address })
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_non_fungibles(
        self,
        resource_address: impl ResolvableResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        self.add_instruction(InstructionV1::AssertWorktopContainsNonFungibles {
            ids: ids.clone().into_iter().collect(),
            resource_address,
        })
    }

    /// Pops the most recent proof from auth zone.
    pub fn pop_from_auth_zone(self, new_proof: impl NewManifestProof) -> Self {
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::PopFromAuthZone {})
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(self, proof: impl ExistingManifestProof) -> Self {
        let proof = proof.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::PushToAuthZone { proof_id: proof })
    }

    /// Clears the auth zone.
    pub fn clear_auth_zone(self) -> Self {
        self.add_instruction(InstructionV1::ClearAuthZone)
    }

    /// Creates proof from the auth zone.
    pub fn create_proof_from_auth_zone(
        self,
        resource_address: impl ResolvableResourceAddress,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromAuthZone { resource_address })
    }

    /// Creates proof from the auth zone by amount.
    pub fn create_proof_from_auth_zone_of_amount(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfAmount {
            amount,
            resource_address,
        })
    }

    /// Creates proof from the auth zone by non-fungible ids.
    pub fn create_proof_from_auth_zone_of_non_fungibles(
        self,
        resource_address: impl ResolvableResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
            ids: ids.into_iter().collect(),
            resource_address,
        })
    }

    /// Creates proof from the auth zone
    pub fn create_proof_from_auth_zone_of_all(
        self,
        resource_address: impl ResolvableResourceAddress,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfAll { resource_address })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket(
        self,
        bucket: impl ExistingManifestBucket,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromBucket { bucket_id: bucket })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_amount(
        self,
        bucket: impl ExistingManifestBucket,
        amount: impl ResolvableDecimal,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        let amount = amount.resolve();
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfAmount {
            bucket_id: bucket,
            amount,
        })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_non_fungibles(
        self,
        bucket: impl ExistingManifestBucket,
        ids: BTreeSet<NonFungibleLocalId>,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfNonFungibles {
            bucket_id: bucket,
            ids: ids.into_iter().collect(),
        })
    }

    /// Creates proof from a bucket. The bucket is not consumed by this process.
    pub fn create_proof_from_bucket_of_all(
        self,
        bucket: impl ExistingManifestBucket,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfAll { bucket_id: bucket })
    }

    /// Clones a proof.
    pub fn clone_proof(
        self,
        proof: impl ExistingManifestProof,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let proof = proof.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CloneProof { proof_id: proof })
    }

    pub fn allocate_global_address(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        new_address_reservation: NamedManifestAddressReservation,
        new_address: NamedManifestAddress,
    ) -> Self {
        let package_address = package_address.resolve_static(&self.registrar);
        let blueprint_name = blueprint_name.into();

        self.registrar
            .register_address_reservation(new_address_reservation);
        self.registrar.register_named_address(new_address);
        self.add_instruction(InstructionV1::AllocateGlobalAddress {
            package_address,
            blueprint_name,
        })
    }

    /// Drops a proof.
    pub fn drop_proof(self, proof: impl ExistingManifestProof) -> Self {
        let proof = proof.mark_consumed(&self.registrar);
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
    pub fn create_fungible_resource(
        self,
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        resource_roles: FungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Option<Decimal>,
    ) -> Self {
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
                        resource_roles,
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
                    resource_roles,
                    address_reservation: None,
                }),
            }
        };
        self.add_instruction(instruction)
    }

    /// Creates a new non-fungible resource
    pub fn create_non_fungible_resource<T, V>(
        self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Option<T>,
    ) -> Self
    where
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode + NonFungibleData,
    {
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
                        resource_roles,
                        metadata,
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
                        resource_roles,
                        metadata,
                        address_reservation: None,
                    }
                ),
            }
        };

        self.add_instruction(instruction)
    }

    pub fn create_ruid_non_fungible_resource<T, V>(
        self,
        owner_role: OwnerRole,
        track_total_supply: bool,
        metadata: ModuleConfig<MetadataInit>,
        resource_roles: NonFungibleResourceRoles,
        initial_supply: Option<T>,
    ) -> Self
    where
        T: IntoIterator<Item = V>,
        V: ManifestEncode + NonFungibleData,
    {
        let instruction = if let Some(initial_supply) = initial_supply {
            let entries = initial_supply
                .into_iter()
                .map(|e| (to_manifest_value_and_unwrap!(&e),))
                .collect();

            InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                        owner_role,
                        track_total_supply,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        resource_roles,
                        metadata,
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
                    &NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                        owner_role,
                        track_total_supply,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        resource_roles,
                        metadata,
                        entries: vec![],
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
        fee_factor: impl ResolvableDecimal,
        xrd_payment: impl ExistingManifestBucket,
    ) -> Self {
        let fee_factor = fee_factor.resolve();
        let xrd_payment = xrd_payment.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: manifest_args!(key, fee_factor, xrd_payment),
        })
    }

    pub fn register_validator(self, validator_address: impl ResolvableComponentAddress) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_REGISTER_IDENT.to_string(),
            args: manifest_args!(),
        })
    }

    pub fn unregister_validator(self, validator_address: impl ResolvableComponentAddress) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_UNREGISTER_IDENT.to_string(),
            args: manifest_args!(),
        })
    }

    pub fn signal_protocol_update_readiness(
        self,
        validator_address: impl ResolvableComponentAddress,
        protocol_version_name: &str,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS.to_string(),
            args: manifest_args!(protocol_version_name.to_string()),
        })
    }

    pub fn stake_validator_as_owner(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    pub fn stake_validator(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_STAKE_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    pub fn unstake_validator(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_UNSTAKE_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    pub fn claim_xrd(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            args: manifest_args!(bucket),
        })
    }

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        args: ManifestValue,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args: to_manifest_value_and_unwrap!(&args),
        })
    }

    /// Calls a scrypto method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        args: ManifestValue,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address,
            method_name: method_name.into(),
            args,
        })
    }

    pub fn claim_package_royalties(self, package_address: impl ResolvablePackageAddress) -> Self {
        let address = package_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackageClaimRoyaltiesInput {}),
        })
    }

    pub fn set_component_royalty(
        self,
        component_address: impl ResolvableComponentAddress,
        method: impl Into<String>,
        amount: RoyaltyAmount,
    ) -> Self {
        let address = component_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentSetRoyaltyInput {
                method: method.into(),
                amount,
            }),
        })
    }

    pub fn lock_component_royalty(
        self,
        component_address: impl ResolvableComponentAddress,
        method: impl Into<String>,
    ) -> Self {
        let address = component_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentLockRoyaltyInput {
                method: method.into(),
            }),
        })
    }

    pub fn claim_component_royalties(
        self,
        component_address: impl ResolvableComponentAddress,
    ) -> Self {
        let address = component_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: address.into(),
            method_name: COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentClaimRoyaltiesInput {}),
        })
    }

    pub fn set_owner_role(self, address: impl ResolvableGlobalAddress, rule: AccessRule) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_SET_OWNER_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesSetOwnerRoleInput { rule }),
        })
    }

    pub fn update_role(
        self,
        address: impl ResolvableGlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
        rule: AccessRule,
    ) -> Self {
        let address = address.resolve(&self.registrar);
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

    pub fn lock_role(
        self,
        address: impl ResolvableGlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_LOCK_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesLockRoleInput { module, role_key }),
        })
    }

    pub fn lock_owner_role(self, address: impl ResolvableGlobalAddress) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_LOCK_OWNER_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesLockOwnerRoleInput {}),
        })
    }

    pub fn get_role(
        self,
        address: impl ResolvableGlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_GET_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesGetRoleInput { module, role_key }),
        })
    }

    pub fn set_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
        value: MetadataValue,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_SET_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataSetInput {
                key: key.into(),
                value
            }),
        })
    }

    pub fn lock_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        let key = key.into();
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_LOCK_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataLockInput { key }),
        })
    }

    pub fn freeze_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
    ) -> Self {
        let address = address.resolve(&self.registrar);
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
        metadata: ModuleConfig<MetadataInit>,
        owner_rule: AccessRule,
    ) -> Self {
        self.create_fungible_resource(
            OwnerRole::Fixed(owner_rule),
            true,
            18,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => OWNER, locked;
                    minter_updater => OWNER, locked;
                },
                burn_roles: burn_roles! {
                    burner => OWNER, locked;
                    burner_updater => OWNER, locked;
                },
                ..Default::default()
            },
            metadata,
            None,
        )
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: impl ResolvableDecimal,
    ) -> Self {
        let initial_supply = initial_supply.resolve();
        self.create_fungible_resource(
            owner_role,
            true,
            18,
            FungibleResourceRoles::default(),
            metadata,
            Some(initial_supply),
        )
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        self,
        metadata: ModuleConfig<MetadataInit>,
        owner_rule: AccessRule,
    ) -> Self {
        self.create_fungible_resource(
            OwnerRole::Fixed(owner_rule),
            false,
            0,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => OWNER, locked;
                    minter_updater => OWNER, locked;
                },
                burn_roles: burn_roles! {
                    burner => OWNER, locked;
                    burner_updater => OWNER, locked;
                },
                ..Default::default()
            },
            metadata,
            None,
        )
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: impl ResolvableDecimal,
    ) -> Self {
        let initial_supply = initial_supply.resolve();
        self.create_fungible_resource(
            owner_role,
            false,
            0,
            FungibleResourceRoles::default(),
            metadata,
            Some(initial_supply),
        )
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_non_fungible_badge_fixed(
        self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: impl ResolvableDecimal,
    ) -> Self {
        let initial_supply = initial_supply.resolve();
        self.create_fungible_resource(
            owner_role,
            false,
            0,
            FungibleResourceRoles::default(),
            metadata,
            Some(initial_supply),
        )
    }

    pub fn burn_resource(self, bucket: impl ExistingManifestBucket) -> Self {
        let bucket = bucket.mark_consumed(&self.registrar);
        self.add_instruction(InstructionV1::BurnResource { bucket_id: bucket })
    }

    pub fn burn_from_worktop(
        self,
        amount: impl ResolvableDecimal,
        resource_address: impl ResolvableResourceAddress,
    ) -> Self {
        let amount = amount.resolve();
        let resource_address = resource_address.resolve(&self.registrar);

        let namer = self.namer();
        let (bucket_name, new_bucket) = namer.new_collision_free_bucket("to_burn");
        self.take_from_worktop(resource_address, amount, new_bucket)
            .burn_resource(namer.bucket(bucket_name))
    }

    pub fn burn_all_from_worktop(self, resource_address: impl ResolvableResourceAddress) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);

        let namer = self.namer();
        let (bucket_name, new_bucket) = namer.new_collision_free_bucket("to_burn");
        self.take_all_from_worktop(resource_address, new_bucket)
            .burn_resource(namer.bucket(bucket_name))
    }

    pub fn burn_non_fungible_from_worktop(
        self,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let ids = btreeset!(non_fungible_global_id.local_id().clone());
        let resource_address = non_fungible_global_id.resource_address().clone();
        let namer = self.namer();
        let (bucket_name, new_bucket) = namer.new_collision_free_bucket("to_burn");

        self.take_non_fungibles_from_worktop(resource_address, ids, new_bucket)
            .burn_resource(namer.bucket(bucket_name))
    }

    pub fn mint_fungible(
        self,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);
        let amount = amount.resolve();
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&FungibleResourceManagerMintInput { amount }),
        })
    }

    pub fn mint_non_fungible<T: IntoIterator<Item = (NonFungibleLocalId, V)>, V: ManifestEncode>(
        self,
        resource_address: impl ResolvableResourceAddress,
        entries: T,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);

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

    pub fn mint_ruid_non_fungible<T: IntoIterator<Item = V>, V: ManifestEncode>(
        self,
        resource_address: impl ResolvableResourceAddress,
        entries: T,
    ) -> Self {
        let address = resource_address.resolve(&self.registrar);

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

    pub fn recall(self, vault_address: InternalAddress, amount: impl ResolvableDecimal) -> Self {
        let amount = amount.resolve();
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_address,
            method_name: VAULT_RECALL_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultRecallInput { amount }),
        })
    }

    pub fn recall_non_fungibles(
        self,
        vault_address: InternalAddress,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let args = to_manifest_value_and_unwrap!(&NonFungibleVaultRecallNonFungiblesInput {
            non_fungible_local_ids,
        });

        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_address,
            method_name: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
            args,
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

    pub fn new_account(self) -> Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccountCreateInput {}),
        })
    }

    pub fn lock_fee_and_withdraw(
        self,
        account_address: impl ResolvableComponentAddress,
        amount_to_lock: impl ResolvableDecimal,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount_to_lock = amount_to_lock.resolve();
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();

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

    pub fn lock_fee_and_withdraw_non_fungibles(
        self,
        account_address: impl ResolvableComponentAddress,
        amount_to_lock: impl ResolvableDecimal,
        resource_address: impl ResolvableResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount_to_lock = amount_to_lock.resolve();
        let resource_address = resource_address.resolve_static(&self.registrar);

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

    /// Locks a large fee from the faucet.
    pub fn lock_fee_from_faucet(self) -> Self {
        self.lock_standard_test_fee(FAUCET)
    }

    /// Locks a large fee from the XRD vault of an account.
    pub fn lock_standard_test_fee(self, account_address: impl ResolvableComponentAddress) -> Self {
        self.lock_fee(account_address, 5000)
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee(
        self,
        account_address: impl ResolvableComponentAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount = amount.resolve();

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            args,
        })
    }

    pub fn lock_contingent_fee(
        self,
        account_address: impl ResolvableComponentAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount = amount.resolve();
        let args = to_manifest_value_and_unwrap!(&AccountLockContingentFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            args,
        })
    }

    /// Locks a large fee from the faucet.
    pub fn get_free_xrd_from_faucet(self) -> Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: FAUCET.into(),
            method_name: "free".to_string(),
            args: manifest_args!(),
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
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
    pub fn withdraw_non_fungibles_from_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);

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
    pub fn burn_in_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
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
    pub fn create_proof_from_account(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);

        let args = to_manifest_value_and_unwrap!(&AccountCreateProofInput { resource_address });

        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
            method_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            args,
        })
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_amount(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);
        let amount = amount.resolve();
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
    pub fn create_proof_from_account_of_non_fungibles(
        self,
        account_address: impl ResolvableComponentAddress,
        resource_address: impl ResolvableResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let resource_address = resource_address.resolve_static(&self.registrar);

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

    pub fn deposit(
        self,
        account_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        let bucket = bucket.mark_consumed(&self.registrar);

        self.call_method(address, ACCOUNT_DEPOSIT_IDENT, manifest_args!(bucket))
    }

    pub fn deposit_batch(self, account_address: impl ResolvableComponentAddress) -> Self {
        let address = account_address.resolve(&self.registrar);

        self.registrar.consume_all_buckets();

        self.call_method(
            address,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn try_deposit_or_abort(
        self,
        account_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        let bucket = bucket.mark_consumed(&self.registrar);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
            manifest_args!(bucket),
        )
    }

    pub fn try_deposit_batch_or_abort(
        self,
        account_address: impl ResolvableComponentAddress,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        self.registrar.consume_all_buckets();

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn try_deposit_or_refund(
        self,
        account_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        let bucket = bucket.mark_consumed(&self.registrar);

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
            manifest_args!(bucket),
        )
    }

    pub fn try_deposit_batch_or_refund(
        self,
        account_address: impl ResolvableComponentAddress,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);

        self.registrar.consume_all_buckets();

        self.call_method(
            address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn create_access_controller(
        self,
        controlled_asset: impl ExistingManifestBucket,
        primary_role: AccessRule,
        recovery_role: AccessRule,
        confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> Self {
        let controlled_asset = controlled_asset.mark_consumed(&self.registrar);
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
