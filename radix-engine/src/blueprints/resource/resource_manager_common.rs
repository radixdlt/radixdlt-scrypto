use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> (AccessRulesConfig, AccessRulesConfig) {
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
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
        update_metadata_access_rule,
        update_metadata_mutability,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::Metadata, METADATA_GET_IDENT),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_group_access_rule_and_mutability(
        "mint",
        mint_access_rule,
        mint_mutability,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
        ),
        "mint",
        DenyAll,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
        ),
        "mint",
        DenyAll,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT,
        ),
        "mint",
        DenyAll,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
        ),
        "mint",
        DenyAll,
    );

    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, RESOURCE_MANAGER_BURN_IDENT),
        burn_access_rule,
        burn_mutability,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
        ),
        update_non_fungible_data_access_rule,
        update_non_fungible_data_mutability,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_CREATE_VAULT_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
        ),
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
        MethodKey::new(TypedModuleId::ObjectState, VAULT_TAKE_IDENT),
        "withdraw",
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
        ),
        "withdraw",
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, FUNGIBLE_VAULT_LOCK_FEE_IDENT),
        "withdraw",
        DenyAll,
    );
    vault_access_rules.set_group_access_rule_and_mutability(
        "recall",
        recall_access_rule,
        recall_mutability,
    );
    vault_access_rules.set_direct_access_group(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_RECALL_IDENT),
        "recall",
    );
    vault_access_rules.set_direct_access_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT,
        ),
        "recall",
    );

    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_PUT_IDENT),
        deposit_access_rule,
        deposit_mutability,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_GET_AMOUNT_IDENT),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_CREATE_PROOF_OF_ALL_IDENT),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
        ),
        AllowAll,
        DenyAll,
    );

    (resman_access_rules, vault_access_rules)
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
    let (resman_access_rules, vault_access_rules) = build_access_rules(access_rules);
    let vault_blueprint_name = if resource_address.as_node_id().is_global_fungible_resource() {
        FUNGIBLE_VAULT_BLUEPRINT
    } else {
        NON_FUNGIBLE_VAULT_BLUEPRINT
    }
    .to_string();

    let resman_access_rules = AccessRules::sys_new(
        resman_access_rules,
        btreemap!(vault_blueprint_name => vault_access_rules),
        api,
    )?
    .0;

    let metadata = Metadata::sys_create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

    api.globalize_with_address(
        object_id,
        btreemap!(
            TypedModuleId::AccessRules => resman_access_rules.0,
            TypedModuleId::Metadata => metadata.0,
            TypedModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
    )?;

    Ok(())
}
