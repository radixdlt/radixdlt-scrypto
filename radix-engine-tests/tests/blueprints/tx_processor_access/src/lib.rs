use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum InstructionOutput {
    CallReturn(Vec<u8>),
    None,
}

#[blueprint]
mod execute_manifest {
    extern_blueprint!(
        "package_rdx1pkgxxxxxxxxxtxnpxrxxxxxxxxx002962227406xxxxxxxxxtxnpxr",
        TransactionProcessor {
            fn run(
                manifest_encoded_instructions: Vec<u8>,
                global_address_reservations: Vec<GlobalAddressReservation>,
                references: Vec<Reference>,
                blobs: IndexMap<Hash, Vec<u8>>
            ) -> Vec<InstructionOutput>;
        }
    );

    struct ExecuteManifest {}

    impl ExecuteManifest {
        pub fn execute_manifest(
            manifest_encoded_instructions: Vec<u8>,
            references: Vec<Reference>,
        ) {
            Blueprint::<TransactionProcessor>::run(
                manifest_encoded_instructions,
                vec![],
                references,
                index_map_new(),
            );
        }
    }
}
