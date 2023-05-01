use scrypto::api::node_modules::metadata::METADATA_SET_IDENT;
use scrypto::api::{ClientObjectApi, ObjectModuleId};
use scrypto::prelude::*;

// Faucet - TestNet only
#[blueprint]
mod faucet {
    struct Faucet {
        vault: Vault,
        transactions: KeyValueStore<Hash, u64>,
    }

    impl Faucet {
        pub fn new(preallocated_address_bytes: [u8; 30], bucket: Bucket) -> ComponentAddress {
            let typed_component = Self {
                vault: Vault::with_bucket(bucket),
                transactions: KeyValueStore::new(),
            }
            .instantiate();

            let access_rules = AccessRules::new({
                let mut config = AccessRulesConfig::new();
                config.set_method_access_rule(
                    MethodKey::new(ObjectModuleId::Metadata, METADATA_SET_IDENT),
                    AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                );
                config.default(AccessRule::AllowAll, AccessRule::DenyAll)
            });
            let metadata = Metadata::new();

            let modules = btreemap!(
                ObjectModuleId::SELF => typed_component.component.0.as_node_id().clone(),
                ObjectModuleId::AccessRules => access_rules.0.0,
                ObjectModuleId::Metadata => metadata.0.0,
                ObjectModuleId::Royalty => Royalty::new(RoyaltyConfig::default()).0.0,
            );

            // See scrypto/src/component/component.rs if this breaks
            scrypto_env::ScryptoEnv
                .globalize_with_address(
                    modules,
                    GlobalAddress::new_or_panic(preallocated_address_bytes),
                )
                .unwrap();

            ComponentAddress::new_or_panic(preallocated_address_bytes)
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
