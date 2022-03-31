use scrypto::prelude::*;

blueprint! {
    struct Receiver {
        vault: Vault,
    }

    impl Receiver {
        pub fn assert_amount(proof: Proof, amount: Decimal, resource_def_id: ResourceDefId) {
            assert_eq!(proof.amount(), amount);
            assert_eq!(proof.resource_def_id(), resource_def_id);
        }

        pub fn assert_ids(
            proof: Proof,
            ids: BTreeSet<NonFungibleId>,
            resource_def_id: ResourceDefId,
        ) {
            assert_eq!(proof.get_non_fungible_ids(), ids);
            assert_eq!(proof.resource_def_id(), resource_def_id);
        }
    }
}
