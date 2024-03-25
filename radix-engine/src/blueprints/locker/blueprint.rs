use super::*;
use crate::internal_prelude::*;
use radix_engine_interface::blueprints::locker::*;

pub const SENDER_ROLE: &str = "sender";
pub const SENDER_UPDATER_ROLE: &str = "sender_updater";
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
                store_batch: Some(ReceiverInfo::normal_ref_mut()),
                send_or_store: Some(ReceiverInfo::normal_ref_mut()),
                send_or_store_batch: Some(ReceiverInfo::normal_ref_mut()),
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
                BatchStoreEvent,
                RecoveryEvent,
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
                        SENDER_ROLE => updaters: [SENDER_UPDATER_ROLE];
                        SENDER_UPDATER_ROLE => updaters: [SENDER_UPDATER_ROLE];
                        RECOVERER_ROLE => updaters: [RECOVERER_UPDATER_ROLE];
                        RECOVERER_UPDATER_ROLE => updaters: [RECOVERER_UPDATER_ROLE];
                    },
                    methods {
                        ACCOUNT_LOCKER_STORE_IDENT => [SENDER_ROLE];
                        ACCOUNT_LOCKER_STORE_BATCH_IDENT => [SENDER_ROLE];
                        ACCOUNT_LOCKER_SEND_OR_STORE_IDENT => [SENDER_ROLE];
                        ACCOUNT_LOCKER_SEND_OR_STORE_BATCH_IDENT => [SENDER_ROLE];

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

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        dispatch! {
            export_name,
            input,
            api,
            AccountLocker,
            [
                instantiate,
                instantiate_simple,
                store,
                store_batch,
                send_or_store,
                send_or_store_batch,
                recover,
                recover_non_fungibles,
                claim,
                claim_non_fungibles,
                get_amount,
                get_non_fungible_local_ids,
            ]
        }
    }

    fn instantiate<Y>(
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
        todo!()
    }

    fn instantiate_simple<Y>(
        AccountLockerInstantiateSimpleInput {
            allow_forceful_withdraws,
        }: AccountLockerInstantiateSimpleInput,
        api: &mut Y,
    ) -> Result<AccountLockerInstantiateSimpleOutput, RuntimeError> {
        todo!()
    }

    fn store<Y>(
        AccountLockerStoreInput { claimant, bucket }: AccountLockerStoreInput,
        api: &mut Y,
    ) -> Result<AccountLockerStoreOutput, RuntimeError> {
        todo!()
    }

    fn store_batch<Y>(
        AccountLockerStoreBatchInput { claimants, bucket }: AccountLockerStoreBatchInput,
        api: &mut Y,
    ) -> Result<AccountLockerStoreBatchOutput, RuntimeError> {
        todo!()
    }

    fn send_or_store<Y>(
        AccountLockerSendOrStoreInput { claimant, bucket }: AccountLockerSendOrStoreInput,
        api: &mut Y,
    ) -> Result<AccountLockerSendOrStoreOutput, RuntimeError> {
        todo!()
    }

    fn send_or_store_batch<Y>(
        AccountLockerSendOrStoreBatchInput { claimants, bucket  }: AccountLockerSendOrStoreBatchInput,
        api: &mut Y,
    ) -> Result<AccountLockerSendOrStoreBatchOutput, RuntimeError> {
        todo!()
    }

    fn recover<Y>(
        AccountLockerRecoverInput {
            claimant,
            resource_address,
            amount,
        }: AccountLockerRecoverInput,
        api: &mut Y,
    ) -> Result<AccountLockerRecoverOutput, RuntimeError> {
        todo!()
    }

    fn recover_non_fungibles<Y>(
        AccountLockerRecoverNonFungiblesInput {
            claimant,
            resource_address,
            amount,
        }: AccountLockerRecoverNonFungiblesInput,
        api: &mut Y,
    ) -> Result<AccountLockerRecoverNonFungiblesOutput, RuntimeError> {
        todo!()
    }

    fn claim<Y>(
        AccountLockerClaimInput {
            claimant,
            resource_address,
            amount,
        }: AccountLockerClaimInput,
        api: &mut Y,
    ) -> Result<AccountLockerClaimOutput, RuntimeError> {
        todo!()
    }

    fn claim_non_fungibles<Y>(
        AccountLockerClaimNonFungiblesInput {
            claimant,
            resource_address,
            amount,
        }: AccountLockerClaimNonFungiblesInput,
        api: &mut Y,
    ) -> Result<AccountLockerClaimNonFungiblesOutput, RuntimeError> {
        todo!()
    }

    fn get_amount<Y>(
        AccountLockerGetAmountInput {
            claimant,
            resource_address,
        }: AccountLockerGetAmountInput,
        api: &mut Y,
    ) -> Result<AccountLockerGetAmountOutput, RuntimeError> {
        todo!()
    }

    fn get_non_fungible_local_ids<Y>(
        AccountLockerGetNonFungibleLocalIdsInput {
            claimant,
            resource_address,
            limit,
        }: AccountLockerGetNonFungibleLocalIdsInput,
        api: &mut Y,
    ) -> Result<AccountLockerGetNonFungibleLocalIdsOutput, RuntimeError> {
        todo!()
    }
}
