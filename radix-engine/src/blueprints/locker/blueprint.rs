#![allow(clippy::too_many_arguments)]

use super::*;
use crate::internal_prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::locker::*;
use radix_native_sdk::modules::metadata::*;
use radix_native_sdk::modules::role_assignment::*;
use radix_native_sdk::resource::*;
use radix_native_sdk::runtime::*;

pub const STORER_ROLE: &str = "storer";
pub const STORER_UPDATER_ROLE: &str = "storer_updater";
pub const RECOVERER_ROLE: &str = "recoverer";
pub const RECOVERER_UPDATER_ROLE: &str = "recoverer_updater";

pub struct AccountLockerBlueprint;

#[allow(unused_variables)]
impl AccountLockerBlueprint {
    pub fn definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let feature_set = AccountLockerFeatureSet::all_features();
        let state = AccountLockerStateSchemaInit::create_schema_init(&mut aggregator);

        let functions = function_schema! {
            aggregator,
            AccountLocker {
                instantiate: None,
                instantiate_simple: None,
                store: Some(ReceiverInfo::normal_ref_mut()),
                airdrop: Some(ReceiverInfo::normal_ref_mut()),
                recover: Some(ReceiverInfo::normal_ref_mut()),
                recover_non_fungibles: Some(ReceiverInfo::normal_ref_mut()),
                claim: Some(ReceiverInfo::normal_ref_mut()),
                claim_non_fungibles: Some(ReceiverInfo::normal_ref_mut()),
                get_amount: Some(ReceiverInfo::normal_ref()),
                get_non_fungible_local_ids: Some(ReceiverInfo::normal_ref()),
            }
        };

        let events = event_schema! {
            aggregator,
            [
                StoreEvent,
                RecoverEvent,
                ClaimEvent,
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set,
            dependencies: indexset!(),
            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template!(
                    roles {
                        STORER_ROLE => updaters: [STORER_UPDATER_ROLE];
                        STORER_UPDATER_ROLE => updaters: [STORER_UPDATER_ROLE];
                        RECOVERER_ROLE => updaters: [RECOVERER_UPDATER_ROLE];
                        RECOVERER_UPDATER_ROLE => updaters: [RECOVERER_UPDATER_ROLE];
                    },
                    methods {
                        ACCOUNT_LOCKER_STORE_IDENT => [STORER_ROLE];
                        ACCOUNT_LOCKER_AIRDROP_IDENT => [STORER_ROLE];

                        ACCOUNT_LOCKER_RECOVER_IDENT => [RECOVERER_ROLE];
                        ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT => [RECOVERER_ROLE];

                        ACCOUNT_LOCKER_CLAIM_IDENT => MethodAccessibility::Public;
                        ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT => MethodAccessibility::Public;
                        ACCOUNT_LOCKER_GET_AMOUNT_IDENT => MethodAccessibility::Public;
                        ACCOUNT_LOCKER_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => MethodAccessibility::Public;
                    }
                )),
            },
        }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        dispatch! {
            EXPORT_NAME,
            export_name,
            input,
            api,
            AccountLocker,
            [
                instantiate,
                instantiate_simple,
                store,
                airdrop,
                recover,
                recover_non_fungibles,
                claim,
                claim_non_fungibles,
                get_amount,
                get_non_fungible_local_ids,
            ]
        }
    }

    fn instantiate<Y: SystemApi<RuntimeError>>(
        AccountLockerInstantiateInput {
            owner_role,
            storer_role,
            storer_updater_role,
            recoverer_role,
            recoverer_updater_role,
            address_reservation,
        }: AccountLockerInstantiateInput,
        api: &mut Y,
    ) -> Result<AccountLockerInstantiateOutput, RuntimeError> {
        Self::instantiate_internal(
            owner_role,
            storer_role,
            storer_updater_role,
            recoverer_role,
            recoverer_updater_role,
            metadata_init! {
                "admin_badge" => EMPTY, locked;
            },
            address_reservation,
            api,
        )
    }

    fn instantiate_simple<Y: SystemApi<RuntimeError>>(
        AccountLockerInstantiateSimpleInput { allow_recover }: AccountLockerInstantiateSimpleInput,
        api: &mut Y,
    ) -> Result<AccountLockerInstantiateSimpleOutput, RuntimeError> {
        // Two address reservations are needed. One for the badge and another for the account locker
        // that we're instantiating.
        let (locker_reservation, locker_address) = api.allocate_global_address(BlueprintId {
            package_address: LOCKER_PACKAGE,
            blueprint_name: ACCOUNT_LOCKER_BLUEPRINT.into(),
        })?;
        let (badge_reservation, badge_address) = api.allocate_global_address(BlueprintId {
            package_address: RESOURCE_PACKAGE,
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.into(),
        })?;
        let badge_address = ResourceAddress::new_or_panic(badge_address.as_node_id().0);

        let (badge_address, badge) = api
            .call_function(
                RESOURCE_PACKAGE,
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&FungibleResourceManagerCreateWithInitialSupplyInput {
                    owner_role: OwnerRole::Updatable(rule!(require(badge_address))),
                    track_total_supply: true,
                    divisibility: 0,
                    resource_roles: Default::default(),
                    metadata: metadata! {
                        init {
                            "account_locker" => locker_address, locked;
                        }
                    },
                    address_reservation: Some(badge_reservation),
                    initial_supply: dec!(1),
                })
                .unwrap(),
            )
            .map(|rtn| {
                scrypto_decode::<FungibleResourceManagerCreateWithInitialSupplyOutput>(&rtn)
                    .unwrap()
            })?;

        // Preparing all of the roles and rules.
        let rule = rule!(require(badge_address));
        let recoverer_rule = match allow_recover {
            true => rule.clone(),
            false => rule!(deny_all),
        };

        Self::instantiate_internal(
            OwnerRole::Updatable(rule.clone()),
            rule.clone(),
            rule.clone(),
            recoverer_rule.clone(),
            recoverer_rule,
            metadata_init! {
                "admin_badge" => badge_address, locked;
            },
            Some(locker_reservation),
            api,
        )
        .map(|rtn| (rtn, badge))
    }

    fn instantiate_internal<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        storer_role: AccessRule,
        storer_updater_role: AccessRule,
        recoverer_role: AccessRule,
        recoverer_updater_role: AccessRule,
        metadata_init: MetadataInit,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Global<AccountLockerMarker>, RuntimeError> {
        // Main module
        let object_id = api.new_simple_object(ACCOUNT_LOCKER_BLUEPRINT, indexmap! {})?;

        // Role Assignment Module
        let roles = indexmap! {
            ModuleId::Main => roles2! {
                STORER_ROLE => storer_role, updatable;
                STORER_UPDATER_ROLE => storer_updater_role, updatable;
                RECOVERER_ROLE => recoverer_role, updatable;
                RECOVERER_UPDATER_ROLE => recoverer_updater_role, updatable;
            }
        };
        let role_assignment = RoleAssignment::create(owner_role, roles, api)?.0;

        // Metadata Module
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        // Globalize
        let address = api.globalize(
            object_id,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment.0,
                AttachedModuleId::Metadata => metadata.0,
            ),
            address_reservation,
        )?;
        let component_address = ComponentAddress::new_or_panic(address.as_node_id().0);

        Ok(Global::new(component_address))
    }

    fn store<Y: SystemApi<RuntimeError>>(
        AccountLockerStoreInput {
            claimant,
            bucket,
            try_direct_send,
        }: AccountLockerStoreInput,
        api: &mut Y,
    ) -> Result<AccountLockerStoreOutput, RuntimeError> {
        // If we should try to send first then attempt the deposit into the account
        let bucket = if try_direct_send {
            // Getting the node-id of the actor and constructing the non-fungible global id of the
            // global caller.
            let actor_node_id = api.actor_get_node_id(ACTOR_STATE_SELF)?;
            let global_caller_non_fungible_global_id =
                global_caller(GlobalAddress::new_or_panic(actor_node_id.0));

            let bucket = api
                .call_method(
                    claimant.0.as_node_id(),
                    ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                    scrypto_encode(&AccountTryDepositOrRefundInput {
                        bucket,
                        authorized_depositor_badge: Some(global_caller_non_fungible_global_id),
                    })
                    .unwrap(),
                )
                .map(|rtn| scrypto_decode::<AccountTryDepositOrRefundOutput>(&rtn).unwrap())?;
            match bucket {
                Some(bucket) => bucket,
                None => return Ok(()),
            }
        } else {
            bucket
        };

        // Deposit into the account either was not requested or failed. Store the resources into the
        // account locker.

        // Bucket info.
        let (resource_address, resource_specifier) = bucket_to_resource_specifier(&bucket, api)?;

        // Store in the vault.
        Self::with_vault_create_on_traversal(
            claimant.0,
            resource_address,
            api,
            |mut vault, api| vault.put(bucket, api),
        )?;

        // Emit an event with the stored resource
        Runtime::emit_event(
            api,
            StoreEvent {
                claimant,
                resource_address,
                resources: resource_specifier,
            },
        )?;

        Ok(())
    }

    fn airdrop<Y: SystemApi<RuntimeError>>(
        AccountLockerAirdropInput {
            claimants,
            bucket,
            try_direct_send,
        }: AccountLockerAirdropInput,
        api: &mut Y,
    ) -> Result<AccountLockerAirdropOutput, RuntimeError> {
        // Distribute and call `store`
        let resource_address = bucket.resource_address(api)?;
        for (account_address, specifier) in claimants.iter() {
            let claim_bucket = match specifier {
                ResourceSpecifier::Fungible(amount) => bucket.take(*amount, api)?,
                ResourceSpecifier::NonFungible(ids) => {
                    bucket.take_non_fungibles(ids.clone(), api)?.into()
                }
            };

            Self::store(
                AccountLockerStoreInput {
                    claimant: *account_address,
                    bucket: claim_bucket,
                    try_direct_send,
                },
                api,
            )?;
        }

        if bucket.is_empty(api)? {
            bucket.drop_empty(api)?;
            Ok(None)
        } else {
            Ok(Some(bucket))
        }
    }

    fn recover<Y: SystemApi<RuntimeError>>(
        AccountLockerRecoverInput {
            claimant,
            resource_address,
            amount,
        }: AccountLockerRecoverInput,
        api: &mut Y,
    ) -> Result<AccountLockerRecoverOutput, RuntimeError> {
        // Recover the resources from the vault.
        let bucket = Self::with_vault_create_on_traversal(
            claimant.0,
            resource_address,
            api,
            |mut vault, api| vault.take(amount, api),
        )?;

        // Emitting the event
        let (resource_address, resource_specifier) = bucket_to_resource_specifier(&bucket, api)?;
        Runtime::emit_event(
            api,
            RecoverEvent {
                claimant,
                resource_address,
                resources: resource_specifier,
            },
        )?;

        // Return
        Ok(bucket)
    }

    fn recover_non_fungibles<Y: SystemApi<RuntimeError>>(
        AccountLockerRecoverNonFungiblesInput {
            claimant,
            resource_address,
            ids,
        }: AccountLockerRecoverNonFungiblesInput,
        api: &mut Y,
    ) -> Result<AccountLockerRecoverNonFungiblesOutput, RuntimeError> {
        // Recover the resources from the vault.
        let bucket = Self::with_vault_create_on_traversal(
            claimant.0,
            resource_address,
            api,
            |mut vault, api| vault.take_non_fungibles(ids, api),
        )?;

        // Emitting the event
        let (resource_address, resource_specifier) = bucket_to_resource_specifier(&bucket, api)?;
        Runtime::emit_event(
            api,
            RecoverEvent {
                claimant,
                resource_address,
                resources: resource_specifier,
            },
        )?;

        // Return
        Ok(bucket)
    }

    fn claim<Y: SystemApi<RuntimeError>>(
        AccountLockerClaimInput {
            claimant,
            resource_address,
            amount,
        }: AccountLockerClaimInput,
        api: &mut Y,
    ) -> Result<AccountLockerClaimOutput, RuntimeError> {
        // Read and assert against the owner role of the claimant.
        let claimant_owner_role = api
            .call_module_method(
                claimant.0.as_node_id(),
                AttachedModuleId::RoleAssignment,
                ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                scrypto_encode(&RoleAssignmentGetOwnerRoleInput).unwrap(),
            )
            .map(|rtn| scrypto_decode::<RoleAssignmentGetOwnerRoleOutput>(&rtn).unwrap())?;
        Runtime::assert_access_rule(claimant_owner_role.rule, api)?;

        // Recover the resources from the vault.
        let bucket = Self::with_vault_create_on_traversal(
            claimant.0,
            resource_address,
            api,
            |mut vault, api| vault.take(amount, api),
        )?;

        // Emitting the event
        let (resource_address, resource_specifier) = bucket_to_resource_specifier(&bucket, api)?;
        Runtime::emit_event(
            api,
            ClaimEvent {
                claimant,
                resource_address,
                resources: resource_specifier,
            },
        )?;

        // Return
        Ok(bucket)
    }

    fn claim_non_fungibles<Y: SystemApi<RuntimeError>>(
        AccountLockerClaimNonFungiblesInput {
            claimant,
            resource_address,
            ids,
        }: AccountLockerClaimNonFungiblesInput,
        api: &mut Y,
    ) -> Result<AccountLockerClaimNonFungiblesOutput, RuntimeError> {
        // Read and assert against the owner role of the claimant.
        let claimant_owner_role = api
            .call_module_method(
                claimant.0.as_node_id(),
                AttachedModuleId::RoleAssignment,
                ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                scrypto_encode(&RoleAssignmentGetOwnerRoleInput).unwrap(),
            )
            .map(|rtn| scrypto_decode::<RoleAssignmentGetOwnerRoleOutput>(&rtn).unwrap())?;
        Runtime::assert_access_rule(claimant_owner_role.rule, api)?;

        // Recover the resources from the vault.
        let bucket = Self::with_vault_create_on_traversal(
            claimant.0,
            resource_address,
            api,
            |mut vault, api| vault.take_non_fungibles(ids, api),
        )?;

        // Emitting the event
        let (resource_address, resource_specifier) = bucket_to_resource_specifier(&bucket, api)?;
        Runtime::emit_event(
            api,
            ClaimEvent {
                claimant,
                resource_address,
                resources: resource_specifier,
            },
        )?;

        // Return
        Ok(bucket)
    }

    fn get_amount<Y: SystemApi<RuntimeError>>(
        AccountLockerGetAmountInput {
            claimant,
            resource_address,
        }: AccountLockerGetAmountInput,
        api: &mut Y,
    ) -> Result<AccountLockerGetAmountOutput, RuntimeError> {
        Self::with_vault(claimant.0, resource_address, api, |vault, api| {
            vault
                .map(|vault| vault.amount(api))
                .unwrap_or(Ok(Decimal::ZERO))
        })
    }

    fn get_non_fungible_local_ids<Y: SystemApi<RuntimeError>>(
        AccountLockerGetNonFungibleLocalIdsInput {
            claimant,
            resource_address,
            limit,
        }: AccountLockerGetNonFungibleLocalIdsInput,
        api: &mut Y,
    ) -> Result<AccountLockerGetNonFungibleLocalIdsOutput, RuntimeError> {
        Self::with_vault(claimant.0, resource_address, api, |vault, api| {
            vault
                .map(|vault| vault.non_fungible_local_ids(limit, api))
                .unwrap_or(Ok(indexset! {}))
        })
    }

    fn with_vault_create_on_traversal<Y: SystemApi<RuntimeError>, O>(
        account_address: ComponentAddress,
        resource_address: ResourceAddress,
        api: &mut Y,
        handler: impl FnOnce(Vault, &mut Y) -> Result<O, RuntimeError>,
    ) -> Result<O, RuntimeError> {
        // The collection on the blueprint maps an account address to a key value store. We read the
        // node id of that key value store.
        let account_claims_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountLockerCollection::AccountClaimsKeyValue.collection_index(),
            &scrypto_encode(&account_address).unwrap(),
            LockFlags::MUTABLE,
        )?;
        let account_claims = api
            .key_value_entry_get_typed::<VersionedAccountLockerAccountClaims>(
                account_claims_handle,
            )?
            .map(|entry| entry.fully_update_and_into_latest_version());

        let account_claims_kv_store = match account_claims {
            Some(account_claims_kv_store) => account_claims_kv_store,
            None => {
                // Create a new kv-store
                let key_value_store = api
                    .key_value_store_new(
                        KeyValueStoreDataSchema::new_local_without_self_package_replacement::<
                            ResourceAddress,
                            Vault,
                        >(true),
                    )
                    .map(Own)?;
                // Write the kv-store's node id to the collection entry.
                api.key_value_entry_set_typed(
                    account_claims_handle,
                    AccountLockerAccountClaimsVersions::V1(key_value_store).into_versioned(),
                )?;
                // Return the NodeId of the kv-store.
                key_value_store
            }
        };

        // Lock the entry in the key-value store which contains the vault and attempt to get it.
        let vault_entry_handle = api.key_value_store_open_entry(
            account_claims_kv_store.as_node_id(),
            &scrypto_encode(&resource_address).unwrap(),
            LockFlags::MUTABLE,
        )?;

        let vault_entry = api.key_value_entry_get_typed::<Vault>(vault_entry_handle)?;
        let vault = match vault_entry {
            Some(vault) => vault,
            None => {
                // Creating the vault.
                let vault = Vault::create(resource_address, api)?;
                // Writing it to the kv-entry
                api.key_value_entry_set_typed(vault_entry_handle, Vault(vault.0))?;
                // Return the vault.
                vault
            }
        };

        // Call the callback - if the callback fails then the following code will not be executed
        // and the substate locks will not be released. We are making the assumption that a failed
        // callback that returns an `Err(RuntimeError)` can not be recovered from.
        let rtn = handler(vault, api)?;

        // Close the opened kv-entries.
        api.key_value_entry_close(vault_entry_handle)?;
        api.key_value_entry_close(account_claims_handle)?;

        // Return the rtn result
        Ok(rtn)
    }

    fn with_vault<Y: SystemApi<RuntimeError>, O>(
        account_address: ComponentAddress,
        resource_address: ResourceAddress,
        api: &mut Y,
        handler: impl FnOnce(Option<Vault>, &mut Y) -> Result<O, RuntimeError>,
    ) -> Result<O, RuntimeError> {
        // The collection on the blueprint maps an account address to a key value store. We read the
        // node id of that key value store.
        let account_claims_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountLockerCollection::AccountClaimsKeyValue.collection_index(),
            &scrypto_encode(&account_address).unwrap(),
            LockFlags::read_only(),
        )?;
        let account_claims = api
            .key_value_entry_get_typed::<VersionedAccountLockerAccountClaims>(
                account_claims_handle,
            )?
            .map(|entry| entry.fully_update_and_into_latest_version());

        let account_claims_kv_store = match account_claims {
            Some(account_claims_kv_store) => account_claims_kv_store,
            None => {
                // Call the callback function.
                let rtn = handler(None, api)?;
                // Dropping the lock on the collection entry.
                api.key_value_entry_close(account_claims_handle)?;
                // Return the result of the callback.
                return Ok(rtn);
            }
        };

        // Lock the entry in the key-value store which contains the vault and attempt to get it. If
        // we're allowed to create the vault.
        let vault_entry_handle = api.key_value_store_open_entry(
            account_claims_kv_store.as_node_id(),
            &scrypto_encode(&resource_address).unwrap(),
            LockFlags::read_only(),
        )?;

        let vault_entry = api.key_value_entry_get_typed::<Vault>(vault_entry_handle)?;

        // Call the callback - if the callback fails then the following code will not be executed
        // and the substate locks will not be released. We are making the assumption that a failed
        // callback that returns an `Err(RuntimeError)` can not be recovered from.
        let rtn = handler(vault_entry, api)?;

        // Close the opened kv-entries.
        api.key_value_entry_close(vault_entry_handle)?;
        api.key_value_entry_close(account_claims_handle)?;

        // Return the rtn result
        Ok(rtn)
    }
}

fn bucket_to_resource_specifier<Y: SystemApi<RuntimeError>>(
    bucket: &Bucket,
    api: &mut Y,
) -> Result<(ResourceAddress, ResourceSpecifier), RuntimeError> {
    let resource_address = bucket.resource_address(api)?;
    if resource_address.is_fungible() {
        let amount = bucket.amount(api)?;
        Ok((resource_address, ResourceSpecifier::Fungible(amount)))
    } else {
        let ids = bucket.non_fungible_local_ids(api)?;
        Ok((resource_address, ResourceSpecifier::NonFungible(ids)))
    }
}
