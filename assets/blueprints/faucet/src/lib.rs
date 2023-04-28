use scrypto::prelude::*;
use radix_engine_interface::api::{
    object_api::{ClientObjectApi, ObjectModuleId},
    node_modules::metadata::METADATA_SET_IDENT,
};

// Faucet - TestNet only
#[blueprint]
mod faucet {
    struct Faucet {
        vault: Vault,
        transactions: KeyValueStore<Hash, u64>,
    }

    impl Faucet {
        pub fn new(preallocated_component_address: ComponentAddress, bucket: Bucket) -> ComponentAddress {
            let typed_component = Self {
                vault: Vault::with_bucket(bucket),
                transactions: KeyValueStore::new(),
            }
            .instantiate();

            let mut access_rules_config = AccessRulesConfig::new();
            access_rules_config.set_method_access_rule(
                MethodKey::new(ObjectModuleId::Metadata, METADATA_SET_IDENT),
                AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            );
            let access_rules_config =
                access_rules_config.default(AccessRule::AllowAll, AccessRule::DenyAll);

            let access_rules = AccessRules::new(access_rules_config);
            let metadata = Metadata::new();
            let royalty = Royalty::new(RoyaltyConfig::default());

            let modules = btreemap!(
                ObjectModuleId::SELF => typed_component.component.0.as_node_id().clone(),
                ObjectModuleId::AccessRules => access_rules.0.0,
                ObjectModuleId::Metadata => metadata.0.0,
                ObjectModuleId::Royalty => royalty.0.0,
            );

            // See scrypto/src/component/component.rs if this breaks 
            scrypto_env::ScryptoEnv
                .globalize_with_address(modules, preallocated_component_address.into())
                .unwrap();

            preallocated_component_address
        }

        /// Gives away tokens.
        pub fn free(&mut self) -> Bucket {
            let transaction_hash = Runtime::transaction_hash();
            let epoch = Runtime::current_epoch();
            assert!(self.transactions.get(&transaction_hash).is_none());
            self.transactions.insert(transaction_hash, epoch);
            self.vault.take(10000)
        }

        /// Locks fees.
        pub fn lock_fee(&mut self, amount: Decimal) {
            // There is MAX_COST_UNIT_LIMIT and COST_UNIT_PRICE which limit how much fee can be spent
            // per transaction, thus no further limitation is applied.
            self.vault.lock_fee(amount);
        }
    }
}
