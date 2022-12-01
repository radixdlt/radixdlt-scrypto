use radix_engine::engine::{AuthError, ModuleError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_mint_owner_badge() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let mut entries = HashMap::new();
    entries.insert(
        NonFungibleId::U32(0),
        (scrypto_encode(&()).unwrap(), scrypto_encode(&()).unwrap()),
    );
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_native_method(
            RENodeId::Global(GlobalAddress::Resource(ENTITY_OWNER_TOKEN)),
            &ResourceManagerMethod::Mint.to_string(),
            scrypto_encode(&ResourceManagerMintInvocation {
                receiver: ENTITY_OWNER_TOKEN,
                mint_params: MintParams::NonFungible { entries },
            })
            .unwrap(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
        )
    });
}
