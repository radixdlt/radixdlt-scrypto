use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> (AccessRulesConfig, AccessRulesConfig, AccessRulesConfig) {
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

    let mut resman_access_rules = AccessRulesConfig::new();
    {
        resman_access_rules.set_group_access_rule_and_mutability(
            "update_metadata",
            update_metadata_access_rule,
            update_metadata_mutability,
        );
        resman_access_rules.set_group_and_mutability(
            MethodKey::new(ObjectModuleId::Metadata, METADATA_SET_IDENT),
            "update_metadata",
            DenyAll,
        );
    }

    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Metadata, METADATA_GET_IDENT),
        AllowAll,
        DenyAll,
    );

    {
        resman_access_rules.set_group_access_rule_and_mutability(
            "mint",
            mint_access_rule,
            mint_mutability,
        );
        resman_access_rules.set_group_and_mutability(
            MethodKey::new(
                ObjectModuleId::Main,
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            ),
            "mint",
            DenyAll,
        );
        resman_access_rules.set_group_and_mutability(
            MethodKey::new(
                ObjectModuleId::Main,
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
            ),
            "mint",
            DenyAll,
        );
        resman_access_rules.set_group_and_mutability(
            MethodKey::new(
                ObjectModuleId::Main,
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT,
            ),
            "mint",
            DenyAll,
        );
        resman_access_rules.set_group_and_mutability(
            MethodKey::new(ObjectModuleId::Main, FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT),
            "mint",
            DenyAll,
        );
    }

    {
        resman_access_rules.set_group_access_rule_and_mutability(
            "burn",
            burn_access_rule,
            burn_mutability,
        );
        resman_access_rules.set_group_and_mutability(
            MethodKey::new(ObjectModuleId::Main, RESOURCE_MANAGER_BURN_IDENT),
            "burn",
            DenyAll,
        );
    }

    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
        ),
        update_non_fungible_data_access_rule,
        update_non_fungible_data_mutability,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Main, RESOURCE_MANAGER_DROP_PROOF_IDENT),
        AllowAll,
        DenyAll,
    );

    let (deposit_access_rule, deposit_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Deposit)
        .unwrap_or((AllowAll, rule!(deny_all)));
    let (withdraw_access_rule, withdraw_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Withdraw)
        .unwrap_or((AllowAll, rule!(deny_all)));
    let (recall_access_rule, recall_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Recall)
        .unwrap_or((DenyAll, rule!(deny_all)));

    let mut vault_access_rules = AccessRulesConfig::new();
    vault_access_rules.set_group_access_rule_and_mutability(
        "withdraw",
        withdraw_access_rule,
        withdraw_mutability,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(ObjectModuleId::Main, VAULT_TAKE_IDENT),
        "withdraw",
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
        ),
        "withdraw",
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(ObjectModuleId::Main, FUNGIBLE_VAULT_LOCK_FEE_IDENT),
        "withdraw",
        DenyAll,
    );

    vault_access_rules.set_group_access_rule_and_mutability(
        "recall",
        recall_access_rule,
        recall_mutability,
    );
    vault_access_rules.set_direct_access_group(
        MethodKey::new(ObjectModuleId::Main, VAULT_RECALL_IDENT),
        "recall",
    );
    vault_access_rules.set_direct_access_group(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT,
        ),
        "recall",
    );

    vault_access_rules.set_group_access_rule_and_mutability(
        "deposit",
        deposit_access_rule,
        deposit_mutability,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(ObjectModuleId::Main, VAULT_PUT_IDENT),
        "deposit",
        DenyAll,
    );

    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Main, VAULT_GET_AMOUNT_IDENT),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Main, VAULT_CREATE_PROOF_OF_ALL_IDENT),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Main, VAULT_CREATE_PROOF_OF_AMOUNT_IDENT),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
        ),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
        ),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
        ),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
        ),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );

    // Not that if a local reference to a bucket is passed to another actor, the recipient will be able
    // to take resource from the bucket. This is not what Scrypto lib supports/encourages, but can be done
    // theoretically.
    let mut bucket_access_rules = AccessRulesConfig::new().default(AllowAll, DenyAll);
    bucket_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Main, FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );
    bucket_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(ObjectModuleId::Main, FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );
    bucket_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT,
        ),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );
    bucket_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            ObjectModuleId::Main,
            NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT,
        ),
        AccessRuleEntry::AccessRule(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        DenyAll,
    );

    (resman_access_rules, vault_access_rules, bucket_access_rules)
}

pub fn globalize_resource_manager<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, String>,
    api: &mut Y,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (resman_access_rules, vault_access_rules, bucket_access_rules) =
        build_access_rules(access_rules);
    let proof_access_rules = AccessRulesConfig::new().default(AllowAll, DenyAll);
    let (vault_blueprint_name, bucket_blueprint_name, proof_blueprint_name) =
        if resource_address.as_node_id().is_global_fungible_resource() {
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

    let resman_access_rules = AccessRules::sys_new(
        resman_access_rules,
        btreemap!(
            vault_blueprint_name.to_string() => vault_access_rules,
            bucket_blueprint_name.to_string() => bucket_access_rules,
            proof_blueprint_name.to_string() => proof_access_rules
        ),
        api,
    )?
    .0;

    let metadata = Metadata::sys_create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

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
    metadata: BTreeMap<String, String>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (resman_access_rules, vault_access_rules, bucket_access_rules) =
        build_access_rules(access_rules);
    let proof_access_rules = AccessRulesConfig::new().default(AllowAll, DenyAll);

    let resman_access_rules = AccessRules::sys_new(
        resman_access_rules,
        btreemap!(
            FUNGIBLE_VAULT_BLUEPRINT.to_string() => vault_access_rules,
            FUNGIBLE_BUCKET_BLUEPRINT.to_string() => bucket_access_rules,
            FUNGIBLE_PROOF_BLUEPRINT.to_string() => proof_access_rules
        ),
        api,
    )?
    .0;

    let metadata = Metadata::sys_create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

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
    metadata: BTreeMap<String, String>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (resman_access_rules, vault_access_rules, bucket_access_rules) =
        build_access_rules(access_rules);
    let proof_access_rules = AccessRulesConfig::new().default(AllowAll, DenyAll);

    let resman_access_rules = AccessRules::sys_new(
        resman_access_rules,
        btreemap!(
            NON_FUNGIBLE_VAULT_BLUEPRINT.to_string() => vault_access_rules,
            NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string()=> bucket_access_rules,
            NON_FUNGIBLE_PROOF_BLUEPRINT.to_string() => proof_access_rules
        ),
        api,
    )?
    .0;

    let metadata = Metadata::sys_create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

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
