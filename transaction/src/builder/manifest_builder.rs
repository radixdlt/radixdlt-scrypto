use crate::internal_prelude::*;
use crate::manifest::decompiler::decompile_with_known_naming;
use crate::manifest::decompiler::ManifestObjectNames;
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

/// A manifest builder for use in tests.
///
/// Note - if you break invariants of the manifest builder (eg resolve a bucket
/// before it's been created, or pass an invalid parameter), this builder will panic.
/// As such, it's only designed for use in test code, or where the inputs are trusted.
///
/// Simple use case:
/// ```
/// # use transaction::prelude::*;
/// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let manifest = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), "xrd")
///     .try_deposit_or_abort(to_account_address, "xrd")
///     .build();
/// ```
///
/// Intermediate use case, where we need to pass a bucket into a component:
/// ```
/// # use transaction::prelude::*;
/// # let package_address = RESOURCE_PACKAGE; // Just some address to get it to compile
/// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let manifest = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), "xrd")
///     .call_function_with_name_lookup(
///         package_address,
///         "SomeBlueprint",
///         "some_function",
///         |lookup| (
///             lookup.bucket("xrd"),
///         ),
///     )
///     .build();
/// ```
///
/// Advanced use case, where we need to generate a collision-free bucket name:
/// ```
/// # use transaction::prelude::*;
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let mut builder = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .get_free_xrd_from_faucet();
/// for _ in 0..32 {
///     // The generate_bucket_name method generates a new bucket name starting with
///     // "transfer" that doesn't collide with any previously used bucket names
///     let bucket_name = builder.generate_bucket_name("transfer");
///     builder = builder
///         .take_from_worktop(XRD, "0.001", &bucket_name)
///         .try_deposit_or_abort(to_account_address, bucket_name);
/// }
/// let manifest = builder.build();
/// ```
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

impl ManifestBuilder {
    /// Starts a new transaction builder.
    pub fn new() -> Self {
        Self {
            registrar: ManifestNameRegistrar::new(),
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
        }
    }

    pub fn name_lookup(&self) -> ManifestNameLookup {
        self.registrar.name_lookup()
    }

    pub fn then(self, next: impl FnOnce(Self) -> Self) -> Self {
        next(self)
    }

    pub fn with_name_lookup(self, next: impl FnOnce(Self, ManifestNameLookup) -> Self) -> Self {
        let lookup = self.name_lookup();
        next(self, lookup)
    }

    pub fn with_bucket(
        self,
        bucket: impl ExistingManifestBucket,
        next: impl FnOnce(Self, ManifestBucket) -> Self,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        next(self, bucket)
    }

    pub fn bucket(&self, name: impl AsRef<str>) -> ManifestBucket {
        self.name_lookup().bucket(name)
    }

    pub fn proof(&self, name: impl AsRef<str>) -> ManifestProof {
        self.name_lookup().proof(name)
    }

    pub fn named_address(&self, name: impl AsRef<str>) -> ManifestAddress {
        self.name_lookup().named_address(name)
    }

    pub fn address_reservation(&self, name: impl AsRef<str>) -> ManifestAddressReservation {
        self.name_lookup().address_reservation(name)
    }

    /// Generates an unused bucket name with the given prefix.
    /// This should be used when you are programatically generating buckets,
    /// and need to generate bucket names which do not clash.
    pub fn generate_bucket_name(&self, prefix: impl Into<String>) -> String {
        self.registrar.new_collision_free_bucket_name(prefix)
    }

    /// Generates an unused proof name with the given prefix.
    /// This should be used when you are programatically generating proofs,
    /// and need to generate names which do not clash.
    pub fn generate_proof_name(&self, prefix: impl Into<String>) -> String {
        self.registrar.new_collision_free_proof_name(prefix)
    }

    /// Generates an unused address reservation name with the given prefix.
    /// This should be used when you are programatically generating address reservations,
    /// and need to generate names which do not clash.
    pub fn generate_address_reservation_name(&self, prefix: impl Into<String>) -> String {
        self.registrar
            .new_collision_free_address_reservation_name(prefix)
    }

    /// Generates an unused address name with the given prefix.
    /// This should be used when you are programatically generating named addresses,
    /// and need to generate names which do not clash.
    pub fn generate_address_name(&self, prefix: impl Into<String>) -> String {
        self.registrar
            .new_collision_free_address_reservation_name(prefix)
    }

    pub fn object_names(&self) -> ManifestObjectNames {
        self.registrar.object_names()
    }

    /// Example usage:
    /// ```
    /// # use transaction::prelude::*;
    /// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// # let package_address = FAUCET_PACKAGE; // Just so it compiles
    ///
    /// let manifest = ManifestBuilder::new()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     // ...
    ///     .then(|mut builder| {
    ///         let code_blob_ref = builder.add_blob(vec![]);
    ///         builder
    ///             .call_function(
    ///                 package_address,
    ///                 "my_blueprint",
    ///                 "func_name",
    ///                 manifest_args!(code_blob_ref),
    ///             )
    ///     })
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

        let registrar = &self.registrar;
        let lookup = self.name_lookup();

        match &instruction {
            InstructionV1::TakeAllFromWorktop { .. }
            | InstructionV1::TakeFromWorktop { .. }
            | InstructionV1::TakeNonFungiblesFromWorktop { .. } => {
                let bucket_name = registrar.new_collision_free_bucket_name("bucket");
                registrar.register_bucket(registrar.new_bucket(&bucket_name));
                new_bucket = Some(lookup.bucket(bucket_name));
            }
            InstructionV1::PopFromAuthZone { .. }
            | InstructionV1::CreateProofFromAuthZoneOfAmount { .. }
            | InstructionV1::CreateProofFromAuthZoneOfNonFungibles { .. }
            | InstructionV1::CreateProofFromAuthZoneOfAll { .. }
            | InstructionV1::CreateProofFromBucketOfAmount { .. }
            | InstructionV1::CreateProofFromBucketOfNonFungibles { .. }
            | InstructionV1::CreateProofFromBucketOfAll { .. }
            | InstructionV1::CloneProof { .. } => {
                let proof_name = registrar.new_collision_free_bucket_name("proof");
                registrar.register_proof(registrar.new_proof(&proof_name));
                new_proof = Some(lookup.proof(proof_name));
            }
            InstructionV1::AllocateGlobalAddress { .. } => {
                let reservation_name =
                    registrar.new_collision_free_address_reservation_name("reservation");
                registrar.register_address_reservation(
                    registrar.new_address_reservation(&reservation_name),
                );

                let address_name = registrar.new_collision_free_address_name("address");
                registrar.register_named_address(registrar.new_named_address(&address_name));
                new_address_reservation = Some(lookup.address_reservation(reservation_name));
                new_address_id = Some(lookup.named_address_id(address_name));
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
        ids: &BTreeSet<NonFungibleLocalId>,
        new_bucket: impl NewManifestBucket,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_bucket.register(&self.registrar);
        self.add_instruction(InstructionV1::TakeNonFungiblesFromWorktop {
            ids: ids.iter().cloned().collect(),
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
        ids: &BTreeSet<NonFungibleLocalId>,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
            ids: ids.iter().cloned().collect(),
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
        ids: &BTreeSet<NonFungibleLocalId>,
        new_proof: impl NewManifestProof,
    ) -> Self {
        let bucket = bucket.resolve(&self.registrar);
        new_proof.register(&self.registrar);
        self.add_instruction(InstructionV1::CreateProofFromBucketOfNonFungibles {
            bucket_id: bucket,
            ids: ids.iter().cloned().collect(),
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
        new_address_reservation_name: impl Into<String>,
        new_address_name: impl Into<String>,
    ) -> Self {
        let package_address = package_address.resolve_static(&self.registrar);
        let blueprint_name = blueprint_name.into();
        let new_address_reservation = self
            .registrar
            .new_address_reservation(new_address_reservation_name);
        let new_named_address = self.registrar.new_named_address(new_address_name);

        self.registrar
            .register_address_reservation(new_address_reservation);
        self.registrar.register_named_address(new_named_address);
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

    pub fn create_identity_advanced(self, owner_role: OwnerRole) -> Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateAdvancedInput { owner_role }),
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
        self.call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
            (key, fee_factor, xrd_payment),
        )
    }

    pub fn register_validator(self, validator_address: impl ResolvableComponentAddress) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.call_method(address, VALIDATOR_REGISTER_IDENT, ())
    }

    pub fn unregister_validator(self, validator_address: impl ResolvableComponentAddress) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.call_method(address, VALIDATOR_UNREGISTER_IDENT, ())
    }

    pub fn signal_protocol_update_readiness(
        self,
        validator_address: impl ResolvableComponentAddress,
        protocol_version_name: &str,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        self.call_method(
            address,
            VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS,
            (protocol_version_name.to_string(),),
        )
    }

    pub fn stake_validator_as_owner(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket: ManifestBucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_STAKE_AS_OWNER_IDENT, (bucket,))
    }

    pub fn stake_validator(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_STAKE_IDENT, (bucket,))
    }

    pub fn unstake_validator(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_UNSTAKE_IDENT, (bucket,))
    }

    pub fn claim_xrd(
        self,
        validator_address: impl ResolvableComponentAddress,
        bucket: impl ExistingManifestBucket,
    ) -> Self {
        let address = validator_address.resolve(&self.registrar);
        let bucket = bucket.mark_consumed(&self.registrar);
        self.call_method(address, VALIDATOR_CLAIM_XRD_IDENT, (bucket,))
    }

    /// Calls a scrypto function where the arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// NOTE: If you need access to named buckets/proofs etc, use `call_function_with_name_lookup`
    /// instead.
    pub fn call_function(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args: arguments.resolve(),
        })
    }

    /// Calls a scrypto function where the arguments are a raw ManifestValue.
    /// The caller is required to ensure the ManifestValue is a Tuple.
    ///
    /// You should prefer `call_function` or `call_function_with_name_lookup` instead.
    pub fn call_function_raw(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments: ManifestValue,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args: arguments,
        })
    }

    /// Calls a scrypto function where the arguments will be created using the given
    /// callback, which takes a `lookup` (allowing for resolving named buckets, proofs, etc)
    /// and returns resolvable arguments.
    ///
    /// The resolvable arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Example:
    /// ```
    /// # use transaction::prelude::*;
    /// # let package_address = RESOURCE_PACKAGE; // Just some address to get it to compile
    /// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// let manifest = ManifestBuilder::new()
    ///     .lock_fee_from_faucet()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     .take_from_worktop(XRD, dec!(1), "xrd_bucket")
    ///     .call_function_with_name_lookup(
    ///         package_address,
    ///         "SomeBlueprint",
    ///         "some_function",
    ///         |lookup| (
    ///             "argument1",
    ///             lookup.bucket("xrd_bucket"),
    ///             dec!("1.3")
    ///         ),
    ///     )
    ///     .build();
    /// ```
    pub fn call_function_with_name_lookup<T: ResolvableArguments>(
        self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments_creator: impl FnOnce(&ManifestNameLookup) -> T,
    ) -> Self {
        let package_address = package_address.resolve(&self.registrar);
        let args = arguments_creator(&self.name_lookup()).resolve();

        self.add_instruction(InstructionV1::CallFunction {
            package_address,
            blueprint_name: blueprint_name.into(),
            function_name: function_name.into(),
            args,
        })
    }

    /// Calls a scrypto method where the arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// NOTE: If you need access to named buckets/proofs etc, use `call_method_with_name_lookup`
    /// instead.
    pub fn call_method(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address,
            method_name: method_name.into(),
            args: arguments.resolve(),
        })
    }

    /// Calls a scrypto method where the arguments are a raw ManifestValue.
    /// The caller is required to ensure the ManifestValue is a Tuple.
    ///
    /// You should prefer `call_function` or `call_function_with_name_lookup` instead.
    pub fn call_method_raw(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments: ManifestValue,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallMethod {
            address,
            method_name: method_name.into(),
            args: arguments,
        })
    }

    /// Calls a scrypto method where the arguments will be created using the given
    /// callback, which takes a `lookup` (allowing for resolving named buckets, proofs, etc)
    /// and returns resolvable arguments.
    ///
    /// The resolvable arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Example:
    /// ```
    /// # use transaction::prelude::*;
    /// # let component_address = GENESIS_HELPER; // Just some address to get it to compile
    /// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
    /// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
    /// # );
    /// let manifest = ManifestBuilder::new()
    ///     .lock_fee_from_faucet()
    ///     .withdraw_from_account(from_account_address, XRD, dec!(1))
    ///     .take_from_worktop(XRD, dec!(1), "xrd_bucket")
    ///     .call_method_with_name_lookup(
    ///         component_address,
    ///         "some_function",
    ///         |lookup| (
    ///             "argument1",
    ///             lookup.bucket("xrd_bucket"),
    ///             dec!("1.3")
    ///         ),
    ///     )
    ///     .build();
    /// ```
    pub fn call_method_with_name_lookup<T: ResolvableArguments>(
        self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        arguments_creator: impl FnOnce(&ManifestNameLookup) -> T,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        let args = arguments_creator(&self.name_lookup()).resolve();

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
        self.add_instruction(InstructionV1::CallRoleAssignmentMethod {
            address: address.into(),
            method_name: ROLE_ASSIGNMENT_SET_OWNER_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&RoleAssignmentSetOwnerInput { rule }),
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
        self.add_instruction(InstructionV1::CallRoleAssignmentMethod {
            address: address.into(),
            method_name: ROLE_ASSIGNMENT_SET_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&RoleAssignmentSetInput {
                module,
                role_key,
                rule,
            }),
        })
    }

    pub fn lock_owner_role(self, address: impl ResolvableGlobalAddress) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallRoleAssignmentMethod {
            address: address.into(),
            method_name: ROLE_ASSIGNMENT_LOCK_OWNER_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&RoleAssignmentLockOwnerInput {}),
        })
    }

    pub fn get_role(
        self,
        address: impl ResolvableGlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        self.add_instruction(InstructionV1::CallRoleAssignmentMethod {
            address: address.into(),
            method_name: ROLE_ASSIGNMENT_GET_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&RoleAssignmentGetInput { module, role_key }),
        })
    }

    pub fn set_metadata(
        self,
        address: impl ResolvableGlobalAddress,
        key: impl Into<String>,
        value: impl ToMetadataEntry,
    ) -> Self {
        let address = address.resolve(&self.registrar);
        match value.to_metadata_entry() {
            Some(value) => self.add_instruction(InstructionV1::CallMetadataMethod {
                address: address.into(),
                method_name: METADATA_SET_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&MetadataSetInput {
                    key: key.into(),
                    value
                }),
            }),
            None => self.add_instruction(InstructionV1::CallMetadataMethod {
                address: address.into(),
                method_name: METADATA_REMOVE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&MetadataRemoveInput { key: key.into() }),
            }),
        }
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
        address_reservation: Option<String>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: impl Into<MetadataInit>,
        owner_role: OwnerRole,
    ) -> Self {
        let address_reservation = if let Some(reservation_name) = address_reservation {
            let reservation = self.name_lookup().address_reservation(reservation_name);
            self.registrar.consume_address_reservation(reservation);
            Some(reservation)
        } else {
            None
        };
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                code: ManifestBlobRef(code_hash.0),
                definition: definition,
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
                definition,
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
                definition,
                metadata: metadata_init!(),
                owner_role: OwnerRole::Fixed(rule!(require(owner_badge.clone()))),
            }),
        })
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        self,
        metadata: ModuleConfig<MetadataInit>,
        owner_role: AccessRule,
    ) -> Self {
        self.create_fungible_resource(
            OwnerRole::Fixed(owner_role),
            true,
            18,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => OWNER;
                    minter_updater => OWNER;
                },
                burn_roles: burn_roles! {
                    burner => OWNER;
                    burner_updater => OWNER;
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
        owner_role: AccessRule,
    ) -> Self {
        self.create_fungible_resource(
            OwnerRole::Fixed(owner_role),
            false,
            0,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => OWNER;
                    minter_updater => OWNER;
                },
                burn_roles: burn_roles! {
                    burner => OWNER;
                    burner_updater => OWNER;
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

        let bucket = self.generate_bucket_name("to_burn");
        self.take_from_worktop(resource_address, amount, &bucket)
            .burn_resource(bucket)
    }

    pub fn burn_all_from_worktop(self, resource_address: impl ResolvableResourceAddress) -> Self {
        let resource_address = resource_address.resolve_static(&self.registrar);

        let bucket = self.generate_bucket_name("to_burn");
        self.take_all_from_worktop(resource_address, &bucket)
            .burn_resource(bucket)
    }

    pub fn burn_non_fungible_from_worktop(
        self,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Self {
        let ids = btreeset!(non_fungible_global_id.local_id().clone());
        let resource_address = non_fungible_global_id.resource_address().clone();
        let bucket = self.generate_bucket_name("to_burn");

        self.take_non_fungibles_from_worktop(resource_address, &ids, &bucket)
            .burn_resource(bucket)
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
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let args = to_manifest_value_and_unwrap!(&NonFungibleVaultRecallNonFungiblesInput {
            non_fungible_local_ids: non_fungible_local_ids.clone(),
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
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let address = account_address.resolve(&self.registrar);
        let amount_to_lock = amount_to_lock.resolve();
        let resource_address = resource_address.resolve_static(&self.registrar);

        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawNonFungiblesInput {
            amount_to_lock,
            resource_address,
            ids: ids.iter().cloned().collect(),
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
        self.call_method(FAUCET, "free", ())
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
        ids: &BTreeSet<NonFungibleLocalId>,
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
        ids: &BTreeSet<NonFungibleLocalId>,
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
        self.call_function(
            ACCESS_CONTROLLER_PACKAGE,
            ACCESS_CONTROLLER_BLUEPRINT,
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
            (
                controlled_asset,
                RuleSet {
                    primary_role,
                    recovery_role,
                    confirmation_role,
                },
                timed_recovery_delay_in_minutes,
            ),
        )
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

    pub fn to_canonical_string(
        &self,
        network_definition: &NetworkDefinition,
    ) -> Result<String, DecompileError> {
        decompile_with_known_naming(&self.instructions, network_definition, self.object_names())
    }
}
