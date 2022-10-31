use radix_engine::engine::{OutputEvent, TraceHeapSnapshot};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::LockedAmountOrIds;
use radix_engine::types::*;
use scrypto_unit::*;
use std::ops::Add;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

#[test]
fn test_trace_resource_transfers() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");
    let transfer_amount = 10u8;

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, 10.into())
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "transfer_resource_between_two_components",
            args!(transfer_amount),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    let output = receipt.expect_commit_success();
    let (resource_address, source_component, target_component): (
        ResourceAddress,
        ComponentAddress,
        ComponentAddress,
    ) = scrypto_decode(&output.get(1).unwrap()[..]).unwrap();

    let account_component_id: ComponentId = test_runner.deref_component(account).unwrap().into();
    let source_component_id: ComponentId = test_runner
        .deref_component(source_component)
        .unwrap()
        .into();
    let target_component_id: ComponentId = test_runner
        .deref_component(target_component)
        .unwrap()
        .into();

    /* There should be three resource changes: withdrawal from the source vault,
    deposit to the target vault and withdrawal for the fee */
    assert_eq!(3, receipt.expect_commit().resource_changes.len());

    let fee_summary = &receipt.execution.fee_summary;

    let fee_resource_address = fee_summary.payments.first().unwrap().1.resource_address();

    let total_fee_paid = fee_summary.burned.add(fee_summary.tipped);

    // Source vault withdrawal
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == source_component_id
            && r.amount == -Decimal::from(transfer_amount)));

    // Target vault deposit
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == target_component_id
            && r.amount == Decimal::from(transfer_amount)));

    // Fee withdrawal
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == fee_resource_address
            && r.component_id == account_component_id
            && r.amount == -Decimal::from(total_fee_paid)));
}

#[test]
fn test_trace_fee_payments() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");

    // Prepare the component that will pay the fee
    let manifest_prepare = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "create_and_fund_a_component",
            args!(Expression::entire_worktop()),
        )
        .clear_auth_zone()
        .build();

    let funded_component = test_runner
        .execute_manifest(manifest_prepare, vec![])
        .new_component_addresses()
        .into_iter()
        .nth(0)
        .unwrap()
        .clone();

    let funded_component_id: ComponentId = test_runner
        .deref_component(funded_component)
        .unwrap()
        .into();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(
            funded_component.clone(),
            "test_lock_contingent_fee",
            args!(),
        )
        .clear_auth_zone()
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let _ = receipt.expect_commit_success();
    let resource_changes = &receipt.expect_commit().resource_changes;
    let fee_summary = &receipt.execution.fee_summary;
    let total_fee_paid = fee_summary.burned.add(fee_summary.tipped);

    assert_eq!(1, resource_changes.len());
    assert!(resource_changes
        .iter()
        .any(|r| r.component_id == funded_component_id && r.amount == -total_fee_paid));
}

#[test]
fn test_instruction_traces() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/execution_trace");

    let manfiest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(SYS_FAUCET_COMPONENT, "free", args!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .create_proof_from_bucket(bucket_id, |builder, proof_id| {
                    builder.drop_proof(proof_id)
                })
                .return_to_worktop(bucket_id)
        })
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "create_and_fund_a_component",
            args!(Expression::entire_worktop()),
        )
        .build();

    let receipt = test_runner.execute_manifest(manfiest, vec![]);

    receipt.expect_commit_success();

    // Check traces for the 7 manifest instructions
    let traces: Vec<(Instruction, TraceHeapSnapshot, TraceHeapSnapshot)> = receipt.execution.output_events
        .into_iter()
        .filter_map(|ev| match ev {
            OutputEvent::InstructionTraceV0(inst, pre, post) => Some((inst, pre, post)),
            _ => None
        })
        .collect();

    {
        // LOCK_FEE
        let (inst, pre, post) = traces.get(0).unwrap();
        assert_method(inst, "lock_fee");
        assert_all_empty(pre);
        assert_all_empty(post);
    }

    {
        // CALL_METHOD: free
        let prev_post = &traces.get(0).unwrap().2;
        let (inst, pre, post) = traces.get(1).unwrap();
        assert_method(inst, "free");
        assert_eq!(pre, prev_post);
        assert!(post.auth_zone_proofs.is_empty());
        assert!(post.owned_proofs.is_empty());
        assert!(post.owned_buckets.is_empty());
        assert_eq!(1, post.worktop_resources.len());
        assert_eq!(
            dec!("1000"),
            post.worktop_resources.get(&RADIX_TOKEN).unwrap().amount()
        );
    }

    {
        // TAKE_FROM_WORKTOP
        let prev_post = &traces.get(1).unwrap().2;
        let (inst, pre, post) = traces.get(2).unwrap();
        assert!(matches!(inst, Instruction::TakeFromWorktop { .. }));
        assert_eq!(pre, prev_post);

        assert!(post.auth_zone_proofs.is_empty());
        assert!(post.owned_proofs.is_empty());
        // TODO: fixme, currently worktop resources still contains an entry with 0 amount
        assert_eq!(1, post.worktop_resources.len());
        assert_eq!(
            dec!("0"),
            post.worktop_resources.get(&RADIX_TOKEN).unwrap().amount()
        );
        assert_eq!(1, post.owned_buckets.len());
        let owned_resource = post.owned_buckets.iter().nth(0).unwrap().1;
        assert_eq!(dec!("1000"), owned_resource.amount());
        assert_eq!(RADIX_TOKEN, owned_resource.resource_address());
    }

    {
        // CREATE_PROOF_FROM_BUCKET
        let prev_post = &traces.get(2).unwrap().2;
        let (inst, pre, post) = traces.get(3).unwrap();
        assert!(matches!(inst, Instruction::CreateProofFromBucket { .. }));
        assert_eq!(pre, prev_post);

        assert!(post.auth_zone_proofs.is_empty());
        // TODO: fixme, currently worktop resources still contains an entry with 0 amount
        assert_eq!(1, post.worktop_resources.len());
        assert_eq!(
            dec!("0"),
            post.worktop_resources.get(&RADIX_TOKEN).unwrap().amount()
        );
        assert_eq!(1, post.owned_buckets.len());
        let owned_resource = post.owned_buckets.iter().nth(0).unwrap().1;
        // Owned amount is 0
        assert_eq!(dec!("0"), owned_resource.amount());
        assert_eq!(RADIX_TOKEN, owned_resource.resource_address());
        // And there is a proof
        assert_eq!(1, post.owned_proofs.len());
        let owned_proof = post.owned_proofs.iter().nth(0).unwrap().1;
        assert_eq!(RADIX_TOKEN, owned_proof.resource_address);
        assert_eq!(
            LockedAmountOrIds::Amount(dec!("1000")),
            owned_proof.total_locked
        );
    }

    {
        // DROP_PROOF
        let (_, prev_pre, prev_post) = &traces.get(3).unwrap();
        let (inst, pre, post) = traces.get(4).unwrap();
        assert!(matches!(inst, Instruction::DropProof { .. }));
        assert_eq!(pre, prev_post);
        assert_eq!(post, prev_pre); // DropProof should simply revert CreateProof
    }

    {
        // RETURN_TO_WORKTOP
        let (_, pre_take_from_worktop, _) = &traces.get(2).unwrap();
        let (_, _, prev_post) = &traces.get(4).unwrap();
        let (inst, pre, post) = traces.get(5).unwrap();
        assert!(matches!(inst, Instruction::ReturnToWorktop { .. }));
        assert_eq!(pre, prev_post);
        assert_eq!(post, pre_take_from_worktop); // ReturnToWorktop should simply revert TakeFromWorktop
    }

    {
        // CALL_FUNCTION: create_and_fund_a_component
        let prev_post = &traces.get(5).unwrap().2;
        let (inst, pre, post) = traces.get(6).unwrap();
        assert_function(inst, "create_and_fund_a_component");
        assert_eq!(pre, prev_post);
        assert_all_empty(post);
    }
}

fn assert_all_empty(snapshot: &TraceHeapSnapshot) {
    assert!(snapshot.auth_zone_proofs.is_empty());
    assert!(snapshot.worktop_resources.is_empty());
    assert!(snapshot.owned_buckets.is_empty());
    assert!(snapshot.owned_proofs.is_empty());
}

fn assert_function(inst: &Instruction, fn_name: &str) {
    assert!(
        matches!(inst, Instruction::CallFunction { function_ident, .. } if function_ident.function_name == fn_name)
    );
}

fn assert_method(inst: &Instruction, method_name: &str) {
    assert!(
        matches!(inst, Instruction::CallMethod { method_ident, .. } if method_ident.method_name == method_name)
    );
}
