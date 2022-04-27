use scrypto::prelude::*;

blueprint! {
    struct Receiver {
        vault: Vault,
    }

    impl Receiver {
        pub fn assert_amount(proof: Proof, amount: Decimal, resource_address: ResourceAddress) {
            assert_eq!(proof.amount(), amount);
            assert_eq!(proof.resource_address(), resource_address);
        }

        pub fn assert_ids(
            proof: Proof,
            ids: BTreeSet<NonFungibleId>,
            resource_address: ResourceAddress,
        ) {
            assert_eq!(proof.non_fungible_ids(), ids);
            assert_eq!(proof.resource_address(), resource_address);
        }
    }
}
