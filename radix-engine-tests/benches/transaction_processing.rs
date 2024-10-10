use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radix_common::prelude::*;
use radix_common::types::Epoch;
use radix_common::ManifestSbor;
use radix_engine_interface::blueprints::resource::RoleAssignmentInit;
use radix_engine_interface::blueprints::resource::{NonFungibleResourceRoles, OwnerRole};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::{metadata, metadata_init};
use radix_transactions::manifest::*;
use radix_transactions::model::*;
use radix_transactions::prelude::*;
use radix_transactions::validation::*;
use scrypto::prelude::*;

fn decompile_notarized_intent_benchmarks(c: &mut Criterion) {
    let network = NetworkDefinition::simulator();
    let raw_transaction = compile_large_notarized_transaction(&network);
    let validator = TransactionValidator::new_with_latest_config(&network);
    c.bench_function("large_transaction_processing::prepare", |b| {
        b.iter(|| {
            black_box(
                raw_transaction
                    .prepare(validator.preparation_settings())
                    .unwrap(),
            )
        })
    });
    c.bench_function("large_transaction_processing::prepare_and_decompile", |b| {
        b.iter(|| {
            #[allow(deprecated)]
            black_box({
                let transaction = raw_transaction
                    .prepare(validator.preparation_settings())
                    .unwrap();
                let PreparedUserTransaction::V1(transaction) = transaction else {
                    unreachable!()
                };
                let manifest = TransactionManifestV1 {
                    instructions: transaction.signed_intent.intent.instructions.inner.0,
                    blobs: transaction.signed_intent.intent.blobs.blobs_by_hash,
                    object_names: Default::default(),
                };
                decompile(&manifest, &NetworkDefinition::simulator()).unwrap()
            })
        })
    });
    c.bench_function(
        "large_transaction_processing::prepare_and_decompile_and_recompile",
        |b| {
            b.iter(|| {
                #[allow(deprecated)]
                black_box({
                    let transaction = raw_transaction
                        .prepare(validator.preparation_settings())
                        .unwrap();
                    let PreparedUserTransaction::V1(transaction) = transaction else {
                        unreachable!()
                    };
                    let manifest = TransactionManifestV1 {
                        instructions: transaction.signed_intent.intent.instructions.inner.into(),
                        blobs: transaction.signed_intent.intent.blobs.blobs_by_hash,
                        object_names: Default::default(),
                    };
                    let decompiled = decompile(&manifest, &NetworkDefinition::simulator()).unwrap();
                    compile_manifest_v1(
                        &decompiled,
                        &NetworkDefinition::simulator(),
                        BlobProvider::new_with_prehashed_blobs(manifest.blobs),
                    )
                })
            })
        },
    );
}

fn compile_large_notarized_transaction(
    network_definition: &NetworkDefinition,
) -> RawNotarizedTransaction {
    let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();
    let component_address = ComponentAddress::preallocated_account_from_public_key(&public_key);

    let manifest = {
        ManifestBuilder::new()
            .lock_fee(component_address, 500)
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles::default(),
                metadata! {},
                Some(
                    (0u64..10_000u64)
                        .into_iter()
                        .map(|id| (NonFungibleLocalId::integer(id), EmptyStruct {}))
                        .collect::<BTreeMap<NonFungibleLocalId, EmptyStruct>>(),
                ),
            )
            .try_deposit_entire_worktop_or_abort(component_address, None)
            .build()
    };
    let header = TransactionHeaderV1 {
        network_id: network_definition.id,
        start_epoch_inclusive: Epoch::of(10),
        end_epoch_exclusive: Epoch::of(13),
        nonce: 0x02,
        notary_public_key: public_key.into(),
        notary_is_signatory: true,
        tip_percentage: 0,
    };
    TransactionBuilder::new()
        .header(header)
        .manifest(manifest)
        .notarize(&private_key)
        .build()
        .to_raw()
        .unwrap()
}

#[derive(NonFungibleData, ScryptoSbor, ManifestSbor)]
struct EmptyStruct {}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = decompile_notarized_intent_benchmarks
);
criterion_main!(benches);
