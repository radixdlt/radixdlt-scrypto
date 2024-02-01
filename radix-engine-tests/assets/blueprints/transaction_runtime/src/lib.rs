use scrypto::prelude::*;

#[blueprint]
mod tx_runtime {
    struct TransactionRuntimeTest {}

    impl TransactionRuntimeTest {
        pub fn query() -> (PackageAddress, Hash, Epoch) {
            (
                Runtime::package_address(),
                Runtime::transaction_hash(),
                Runtime::current_epoch(),
            )
        }
        pub fn generate_ruid() -> [u8; 32] {
            Runtime::generate_ruid()
        }

        pub fn test_instance_of_and_blueprint_id() {
            let x = TransactionRuntimeTest {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            assert_eq!(
                x.instance_of(&BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: "TransactionRuntimeTest".to_owned()
                }),
                true
            );
            assert_eq!(
                x.instance_of(&BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: "TransactionRuntimeTest2".to_owned()
                }),
                false
            );
            assert_eq!(
                x.blueprint_id(),
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: "TransactionRuntimeTest".to_owned()
                }
            );
        }
    }
}
