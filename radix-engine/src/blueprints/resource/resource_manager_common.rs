use crate::errors::RuntimeError;
use crate::types::*;
use crate::{method_permissions, permission_entry};
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::MethodPermission::Public;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> (
    Roles, BTreeMap<MethodKey, (MethodPermission, RoleList)>,
    BTreeMap<MethodKey, (MethodPermission, RoleList)>,
    BTreeMap<MethodKey, (MethodPermission, RoleList)>,
    BTreeMap<MethodKey, (MethodPermission, RoleList)>,
) {
    let mut roles = Roles::new();

    let resman_protected_methods = {
        let (mint_access_rule, mint_mutability) = access_rules_map
            .remove(&Mint)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (burn_access_rule, burn_mutability) = access_rules_map
            .remove(&Burn)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) =
            access_rules_map
                .remove(&UpdateNonFungibleData)
                .unwrap_or((AllowAll, rule!(deny_all)));
        let (update_metadata_access_rule, update_metadata_mutability) = access_rules_map
            .remove(&UpdateMetadata)
            .unwrap_or((DenyAll, rule!(deny_all)));

        {
            roles.define_role(
                UPDATE_METADATA_UPDATE_AUTHORITY,
                update_metadata_mutability,
                vec![UPDATE_METADATA_UPDATE_AUTHORITY],
            );

            roles.define_role(
                UPDATE_METADATA_AUTHORITY,
                update_metadata_access_rule,
                vec![UPDATE_METADATA_UPDATE_AUTHORITY],
            );
        }

        // Mint
        {
            roles.define_role(
                MINT_UPDATE_AUTHORITY,
                mint_mutability,
                vec![MINT_UPDATE_AUTHORITY],
            );
            roles.define_role(
                MINT_AUTHORITY,
                mint_access_rule,
                vec![MINT_UPDATE_AUTHORITY],
            );
        }

        // Burn
        {
            roles.define_role(
                BURN_UPDATE_AUTHORITY,
                burn_mutability,
                vec![BURN_UPDATE_AUTHORITY],
            );
            roles.define_role(
                BURN_AUTHORITY,
                burn_access_rule,
                vec![BURN_UPDATE_AUTHORITY],
            );
        }

        // Non Fungible Update data
        {
            roles.define_role(
                UPDATE_NON_FUNGIBLE_DATA_UPDATE_AUTHORITY,
                update_non_fungible_data_mutability,
                vec![UPDATE_NON_FUNGIBLE_DATA_UPDATE_AUTHORITY],
            );

            roles.define_role(
                UPDATE_NON_FUNGIBLE_DATA_AUTHORITY,
                update_non_fungible_data_access_rule,
                vec![UPDATE_NON_FUNGIBLE_DATA_UPDATE_AUTHORITY],
            );
        }

        let resman_protected_methods = method_permissions!(
            MethodKey::metadata(METADATA_GET_IDENT) => [UPDATE_METADATA_AUTHORITY];
            MethodKey::metadata(METADATA_SET_IDENT) => [UPDATE_METADATA_AUTHORITY];
            MethodKey::main(FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT) => [MINT_AUTHORITY];
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT) => [MINT_AUTHORITY];
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT) => [MINT_AUTHORITY];
            MethodKey::main(RESOURCE_MANAGER_BURN_IDENT) => [BURN_AUTHORITY];
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT) => [UPDATE_NON_FUNGIBLE_DATA_AUTHORITY];
            MethodKey::main(RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT) => Public;
            MethodKey::main(RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT) => Public;
            MethodKey::main(RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT) => Public;
            MethodKey::main(RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT) => Public;
            MethodKey::main(RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT) => Public;
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT) => Public;
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT) => Public;
        );

        resman_protected_methods
    };

    let vault_protected_methods = {
        let (deposit_access_rule, deposit_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Deposit)
            .unwrap_or((AllowAll, rule!(deny_all)));
        let (withdraw_access_rule, withdraw_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Withdraw)
            .unwrap_or((AllowAll, rule!(deny_all)));
        let (recall_access_rule, recall_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Recall)
            .unwrap_or((DenyAll, rule!(deny_all)));

        // Withdraw
        {
            roles.define_role(
                WITHDRAW_UPDATE_AUTHORITY,
                withdraw_mutability,
                vec![WITHDRAW_UPDATE_AUTHORITY],
            );
            roles.define_role(
                WITHDRAW_AUTHORITY,
                withdraw_access_rule,
                vec![WITHDRAW_UPDATE_AUTHORITY],
            );
        }

        // Recall
        {
            roles.define_role(
                RECALL_UPDATE_AUTHORITY,
                recall_mutability,
                vec![RECALL_UPDATE_AUTHORITY],
            );
            roles.define_role(
                RECALL_AUTHORITY,
                recall_access_rule,
                vec![RECALL_UPDATE_AUTHORITY],
            );
        }

        // Deposit
        {
            roles.define_role(
                DEPOSIT_UPDATE_AUTHORITY,
                deposit_mutability,
                vec![DEPOSIT_UPDATE_AUTHORITY],
            );

            roles.define_role(
                DEPOSIT_AUTHORITY,
                deposit_access_rule,
                vec![DEPOSIT_UPDATE_AUTHORITY],
            );
        }

        // Internal
        {
            roles.define_role(
                "this_package",
                rule!(require(package_of_direct_caller(RESOURCE_PACKAGE))),
                vec![],
            );
        }

        let vault_protected_methods = method_permissions!(
            MethodKey::main(VAULT_GET_AMOUNT_IDENT) => Public;
            MethodKey::main(VAULT_CREATE_PROOF_IDENT) => Public;
            MethodKey::main(VAULT_CREATE_PROOF_OF_AMOUNT_IDENT) => Public;

            MethodKey::main(VAULT_TAKE_IDENT) => [WITHDRAW_AUTHORITY];
            MethodKey::main(FUNGIBLE_VAULT_LOCK_FEE_IDENT) => [WITHDRAW_AUTHORITY];
            MethodKey::main(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT) => [WITHDRAW_AUTHORITY];
            MethodKey::main(VAULT_RECALL_IDENT) => [RECALL_AUTHORITY];
            MethodKey::main(NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT) => [RECALL_AUTHORITY];
            MethodKey::main(VAULT_PUT_IDENT) => [DEPOSIT_AUTHORITY];
            MethodKey::main(FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT) => ["this_package"];
            MethodKey::main(NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
            MethodKey::main(FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT) => ["this_package"];
            MethodKey::main(NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
        );

        vault_protected_methods
    };

    // Note that if a local reference to a bucket is passed to another actor, the recipient will be able
    // to take resource from the bucket. This is not what Scrypto lib supports/encourages, but can be done
    // theoretically.

    let bucket_protected_methods = {
        let bucket_protected_methods = method_permissions!(
            MethodKey::main(BUCKET_GET_AMOUNT_IDENT) => Public;
            MethodKey::main(BUCKET_GET_RESOURCE_ADDRESS_IDENT) => Public;
            MethodKey::main(BUCKET_CREATE_PROOF_IDENT) => Public;
            MethodKey::main(BUCKET_CREATE_PROOF_OF_ALL_IDENT) => Public;
            MethodKey::main(BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT) => Public;
            MethodKey::main(BUCKET_PUT_IDENT) => Public;
            MethodKey::main(BUCKET_TAKE_IDENT) => Public;
            MethodKey::main(NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT) => Public;
            MethodKey::main(NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT) => Public;

            MethodKey::main(FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT) => ["this_package"];
            MethodKey::main(FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT) => ["this_package"];
            MethodKey::main(NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
            MethodKey::main(NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
        );

        bucket_protected_methods
    };

    let protected_proof_methods = method_permissions!(
        MethodKey::main(PROOF_GET_RESOURCE_ADDRESS_IDENT) => Public;
        MethodKey::main(PROOF_CLONE_IDENT) => Public;
        MethodKey::main(PROOF_DROP_IDENT) => Public;
        MethodKey::main(PROOF_GET_AMOUNT_IDENT) => Public;
        MethodKey::main(NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT) => Public;
    );


    (
        roles,
        resman_protected_methods,
        vault_protected_methods,
        bucket_protected_methods,
        protected_proof_methods
    )
}

pub fn globalize_resource_manager<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    api: &mut Y,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (
        roles,
        protected_resman_methods,
        protected_vault_methods,
        protected_bucket_methods,
        protected_proof_methods,
    ) = build_access_rules(access_rules);

    let (vault_blueprint_name, bucket_blueprint_name, proof_blueprint_name) = if resource_address
        .as_node_id()
        .is_global_fungible_resource_manager()
    {
        (
            FUNGIBLE_VAULT_BLUEPRINT,
            FUNGIBLE_BUCKET_BLUEPRINT,
            FUNGIBLE_PROOF_BLUEPRINT,
        )
    } else {
        (
            NON_FUNGIBLE_VAULT_BLUEPRINT,
            NON_FUNGIBLE_BUCKET_BLUEPRINT,
            NON_FUNGIBLE_PROOF_BLUEPRINT,
        )
    };


    let resman_access_rules = AccessRules::create(
        protected_resman_methods,
        roles,
        btreemap!(
            vault_blueprint_name.to_string() => protected_vault_methods,
            bucket_blueprint_name.to_string() => protected_bucket_methods,
            proof_blueprint_name.to_string() => protected_proof_methods,
        ),
        api,
    )?
    .0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    api.globalize_with_address(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
    )?;

    Ok(())
}

pub fn globalize_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (
        roles,
        protected_resman_methods,
        protected_vault_methods,
        protected_bucket_methods,
        protected_proof_methods,
    ) = build_access_rules(access_rules);

    let resman_access_rules = AccessRules::create(
        protected_resman_methods,
        roles,
        btreemap!(
            FUNGIBLE_VAULT_BLUEPRINT.to_string() => protected_vault_methods,
            FUNGIBLE_BUCKET_BLUEPRINT.to_string() => protected_bucket_methods,
            FUNGIBLE_PROOF_BLUEPRINT.to_string() => protected_proof_methods,
        ),
        api,
    )?
    .0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    let bucket_id = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
        FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidFungibleResource::new(initial_supply)).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
}

pub fn globalize_non_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (
        roles,
        protected_resman_methods,
        protected_vault_methods,
        protected_bucket_methods,
        protected_proof_methods,
    ) = build_access_rules(access_rules);

    let resman_access_rules = AccessRules::create(
        protected_resman_methods,
        roles,
        btreemap!(
            NON_FUNGIBLE_VAULT_BLUEPRINT.to_string() => protected_vault_methods,
            NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string() => protected_bucket_methods,
            NON_FUNGIBLE_PROOF_BLUEPRINT.to_string() => protected_proof_methods,
        ),
        api,
    )?
    .0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    let bucket_id = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
}
