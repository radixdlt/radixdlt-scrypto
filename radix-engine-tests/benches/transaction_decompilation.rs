use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radix_engine::types::{
    ManifestExpression, NetworkDefinition, NonFungibleIdType, NonFungibleLocalId,
};
use radix_engine_common::types::Epoch;
use radix_engine_common::{manifest_args, ManifestSbor};
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::resource::OwnerRole;
use radix_engine_interface::blueprints::resource::RolesInit;
use radix_engine_interface::{metadata, metadata_init, ScryptoSbor};
use scrypto::prelude::{AccessRule, ComponentAddress};
use scrypto::NonFungibleData;
use transaction::builder::{ManifestBuilder, TransactionBuilder};
use transaction::manifest::{compile, decompile, BlobProvider};
use transaction::model::{
    PreparedNotarizedTransactionV1, TransactionHeaderV1, TransactionPayload,
    TransactionPayloadPreparable,
};
use transaction::signing::secp256k1::Secp256k1PrivateKey;

fn decompile_notarized_intent_benchmarks(c: &mut Criterion) {
    let compiled_transaction = compiled_notarized_transaction();
    let mut group = c.benchmark_group("Decompile Intent Natively");

    group.bench_function("Prepare NotarizedTransaction", |b| {
        b.iter(|| {
            black_box(
                PreparedNotarizedTransactionV1::prepare_from_payload(&compiled_transaction)
                    .unwrap(),
            )
        })
    });
    group.bench_function("Prepare NotarizedTransaction and Decompile", |b| {
        b.iter(|| {
            black_box({
                let transaction: PreparedNotarizedTransactionV1 =
                    PreparedNotarizedTransactionV1::prepare_from_payload(&compiled_transaction)
                        .unwrap();
                decompile(
                    &transaction.signed_intent.intent.instructions.inner.0,
                    &NetworkDefinition::simulator(),
                )
                .unwrap()
            })
        })
    });
    group.bench_function(
        "Prepare NotarizedTransaction, Decompile, then Recompile",
        |b| {
            b.iter(|| {
                black_box({
                    let transaction =
                        PreparedNotarizedTransactionV1::prepare_from_payload(&compiled_transaction)
                            .unwrap();
                    let manifest = decompile(
                        &transaction.signed_intent.intent.instructions.inner.0,
                        &NetworkDefinition::simulator(),
                    )
                    .unwrap();
                    compile(
                        &manifest,
                        &NetworkDefinition::simulator(),
                        BlobProvider::new(),
                    )
                })
            })
        },
    );

    group.finish();
}

fn compiled_notarized_transaction() -> Vec<u8> {
    let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();
    let component_address = ComponentAddress::virtual_account_from_public_key(&public_key);

    let manifest = {
        let mut builder = ManifestBuilder::new();
        builder.lock_fee(component_address, 500u32.into());
        builder.create_non_fungible_resource(
            OwnerRole::None,
            NonFungibleIdType::Integer,
            false,
            metadata! {},
            BTreeMap::<_, (_, AccessRule)>::new(),
            Some(
                (0u64..10_000u64)
                    .into_iter()
                    .map(|id| (NonFungibleLocalId::integer(id), EmptyStruct {}))
                    .collect::<BTreeMap<NonFungibleLocalId, EmptyStruct>>(),
            ),
        );
        builder.call_method(
            component_address,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        );
        builder.build()
    };
    let header = TransactionHeaderV1 {
        network_id: 0xf2,
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
        .to_payload_bytes()
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
