use radix_engine::engine::*;
use radix_engine::model::*;
use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_trace_resource_transfers() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");
    let transfer_amount = 10u8;

    // Act
    let manifest = ManifestBuilder::new()
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
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let (resource_address, source_component, target_component): (
        ResourceAddress,
        ComponentAddress,
        ComponentAddress,
    ) = receipt.output(1);

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

    let total_fee_paid = fee_summary.total_execution_cost_xrd + fee_summary.total_royalty_cost_xrd
        - fee_summary.bad_debt_xrd;

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
        .any(|r| r.resource_address == RADIX_TOKEN
            && r.component_id == account_component_id
            && r.amount == -Decimal::from(total_fee_paid)));
}

#[test]
fn test_trace_fee_payments() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");

    // Prepare the component that will pay the fee
    let manifest_prepare = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "create_and_fund_a_component",
            args!(ManifestExpression::EntireWorktop),
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
    let manifest = ManifestBuilder::new()
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
    let total_fee_paid = fee_summary.total_execution_cost_xrd + fee_summary.total_royalty_cost_xrd
        - fee_summary.bad_debt_xrd;

    assert_eq!(1, resource_changes.len());
    assert!(resource_changes
        .iter()
        .any(|r| r.component_id == funded_component_id && r.amount == -total_fee_paid));
}

#[test]
fn test_instruction_traces() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");

    let manfiest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .create_proof_from_bucket(&bucket_id, |builder, proof_id| {
                    builder.drop_proof(proof_id)
                })
                .return_to_worktop(bucket_id)
        })
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "create_and_fund_a_component",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest(manfiest, vec![]);

    receipt.expect_commit_success();

    let mut traces: Vec<SysCallTrace> = receipt
        .execution
        .events
        .into_iter()
        .filter_map(|e| match e {
            TrackedEvent::SysCallTrace(trace) => Some(trace),
        })
        .collect();

    // Expecting a single root trace
    assert_eq!(1, traces.len());

    let root_trace = traces.remove(0);
    let child_traces = root_trace.children;

    // Check traces for the 7 manifest instructions
    {
        // LOCK_FEE
        let traces = traces_for_instruction(&child_traces, 0);
        assert!(traces.is_empty()); // No traces for lock_fee
    }

    {
        // CALL_METHOD: free
        let traces = traces_for_instruction(&child_traces, 1);
        // Expecting two traces: an output bucket from the "free" call
        // followed by a single input (auto-add to worktop) - in this order.
        assert_eq!(2, traces.len());
        let free_trace = traces.get(0).unwrap();
        if let SysCallTraceOrigin::ScryptoMethod(ScryptoFnIdentifier {
            ident: method_name, ..
        }) = &free_trace.origin
        {
            assert_eq!("free", method_name);
        } else {
            panic!(
                "Expected a scrypto method call but was {:?}",
                free_trace.origin
            );
        };
        assert!(free_trace.input.is_empty());
        assert!(free_trace.output.proofs.is_empty());
        assert_eq!(1, free_trace.output.buckets.len());
        let output_resource = free_trace.output.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, output_resource.resource_address());
        assert_eq!(dec!("1000"), output_resource.amount());

        let worktop_put_trace = traces.get(1).unwrap();
        assert_eq!(
            SysCallTraceOrigin::NativeFn(NativeFn::Worktop(WorktopFn::Put)),
            worktop_put_trace.origin
        );
        assert!(worktop_put_trace.output.is_empty());
        assert!(worktop_put_trace.input.proofs.is_empty());
        assert_eq!(1, worktop_put_trace.input.buckets.len());
        let input_resource = worktop_put_trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_resource.resource_address());
        assert_eq!(dec!("1000"), input_resource.amount());

        // We're tracking up to depth "1" (default), so no more child traces
        assert!(free_trace.children.is_empty());
        assert!(worktop_put_trace.children.is_empty());
    }

    {
        // TAKE_FROM_WORKTOP
        let traces = traces_for_instruction(&child_traces, 2);
        // Take from worktop is just a single sys call with a single bucket output
        assert_eq!(1, traces.len());

        let trace = traces.get(0).unwrap();
        assert_eq!(
            SysCallTraceOrigin::NativeFn(NativeFn::Worktop(WorktopFn::TakeAll)),
            trace.origin
        );

        assert!(trace.input.is_empty());
        assert!(trace.output.proofs.is_empty());
        assert_eq!(1, trace.output.buckets.len());

        let output_resource = trace.output.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, output_resource.resource_address());
        assert_eq!(dec!("1000"), output_resource.amount());
    }

    {
        // CREATE_PROOF_FROM_BUCKET
        let traces = traces_for_instruction(&child_traces, 3);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            SysCallTraceOrigin::NativeFn(NativeFn::Bucket(BucketFn::CreateProof)),
            trace.origin
        );

        assert!(trace.input.is_empty());
        assert!(trace.output.buckets.is_empty());
        assert_eq!(1, trace.output.proofs.len());

        let output_proof = trace.output.proofs.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, output_proof.resource_address);
        assert_eq!(
            LockedAmountOrIds::Amount(dec!("1000")),
            output_proof.total_locked
        );
    }

    {
        // DROP_PROOF
        let traces = traces_for_instruction(&child_traces, 4);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(SysCallTraceOrigin::DropNode, trace.origin);

        assert!(trace.output.is_empty());
        assert!(trace.input.buckets.is_empty());
        assert_eq!(1, trace.input.proofs.len());

        let input_proof = trace.input.proofs.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_proof.resource_address);
        assert_eq!(
            LockedAmountOrIds::Amount(dec!("1000")),
            input_proof.total_locked
        );
    }

    {
        // RETURN_TO_WORKTOP
        let traces = traces_for_instruction(&child_traces, 5);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            SysCallTraceOrigin::NativeFn(NativeFn::Worktop(WorktopFn::Put)),
            trace.origin
        );
        assert!(trace.output.is_empty());
        assert!(trace.input.proofs.is_empty());
        assert_eq!(1, trace.input.buckets.len());

        let input_resource = trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_resource.resource_address());
        assert_eq!(dec!("1000"), input_resource.amount());
    }

    {
        // CALL_FUNCTION: create_and_fund_a_component
        let traces = traces_for_instruction(&child_traces, 6);
        // Expected two traces: take from worktop and call scrypto function
        assert_eq!(2, traces.len());

        let take_trace = traces.get(0).unwrap();
        assert_eq!(
            SysCallTraceOrigin::NativeFn(NativeFn::Worktop(WorktopFn::Drain)),
            take_trace.origin
        );

        let call_trace = traces.get(1).unwrap();
        if let SysCallTraceOrigin::ScryptoFunction(ScryptoFnIdentifier {
            ident: function_name,
            ..
        }) = &call_trace.origin
        {
            assert_eq!("create_and_fund_a_component", function_name);
        } else {
            panic!("Expected a scrypto function call");
        };
        assert!(call_trace.output.is_empty());
        assert!(call_trace.input.proofs.is_empty());
        assert_eq!(1, call_trace.input.buckets.len());
        let input_resource = call_trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_resource.resource_address());
        assert_eq!(dec!("1000"), input_resource.amount());
    }
}

fn traces_for_instruction(
    traces: &Vec<SysCallTrace>,
    instruction_index: u32,
) -> Vec<&SysCallTrace> {
    traces
        .iter()
        .filter(|t| t.instruction_index == Some(instruction_index))
        .collect()
}
