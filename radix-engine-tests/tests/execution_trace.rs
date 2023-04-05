use radix_engine::system::kernel_modules::execution_trace::{
    ApplicationFnIdentifier, ExecutionTrace, Origin, ResourceSpecifier, WorktopChange,
};
use radix_engine::types::*;
use radix_engine_interface::blueprints::account::ACCOUNT_DEPOSIT_BATCH_IDENT;
use radix_engine_interface::blueprints::resource::*;
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
            manifest_args!(transfer_amount),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let (_resource_address, source_component, target_component): (
        ResourceAddress,
        ComponentAddress,
        ComponentAddress,
    ) = receipt.expect_commit(true).output(1);

    /* There should be three resource changes: withdrawal from the source vault,
    deposit to the target vault and withdrawal for the fee */
    println!("{:?}", receipt.execution_trace.resource_changes);
    assert_eq!(2, receipt.execution_trace.resource_changes.len()); // Two instructions
    assert_eq!(
        1,
        receipt
            .execution_trace
            .resource_changes
            .get(&0)
            .unwrap()
            .len()
    ); // One resource change in the first instruction (lock fee)
    assert_eq!(
        2,
        receipt
            .execution_trace
            .resource_changes
            .get(&1)
            .unwrap()
            .len()
    ); // One resource change in the first instruction (lock fee)

    let fee_summary = receipt.expect_commit(true).fee_summary.clone();
    let total_fee_paid = fee_summary.total_execution_cost_xrd + fee_summary.total_royalty_cost_xrd
        - fee_summary.total_bad_debt_xrd;

    // Source vault withdrawal
    assert!(receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(
            |r| r.node_id == source_component.into() && r.amount == -Decimal::from(transfer_amount)
        ));

    // Target vault deposit
    assert!(receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(
            |r| r.node_id == target_component.into() && r.amount == Decimal::from(transfer_amount)
        ));

    // Fee withdrawal
    assert!(receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|r| r.node_id == account.into() && r.amount == -Decimal::from(total_fee_paid)));
}

#[test]
fn test_trace_fee_payments() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");

    // Prepare the component that will pay the fee
    let manifest_prepare = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "create_and_fund_a_component",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .clear_auth_zone()
        .build();

    let funded_component = test_runner
        .execute_manifest(manifest_prepare, vec![])
        .expect_commit(true)
        .new_component_addresses()
        .into_iter()
        .nth(0)
        .unwrap()
        .clone();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            funded_component.clone(),
            "test_lock_contingent_fee",
            manifest_args!(),
        )
        .clear_auth_zone()
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let resource_changes = &receipt.execution_trace.resource_changes;
    let fee_summary = receipt.expect_commit(true).fee_summary.clone();
    let total_fee_paid = fee_summary.total_execution_cost_xrd + fee_summary.total_royalty_cost_xrd
        - fee_summary.total_bad_debt_xrd;

    assert_eq!(1, resource_changes.len());
    assert!(resource_changes
        .into_iter()
        .flat_map(|(_, rc)| rc)
        .any(|r| r.node_id == funded_component.into() && r.amount == -total_fee_paid));
}

#[test]
fn test_instruction_traces() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/execution_trace");

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
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
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    receipt.expect_commit_success();

    let mut traces: Vec<ExecutionTrace> = receipt.execution_trace.execution_traces;

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
        if let Origin::ScryptoMethod(ApplicationFnIdentifier {
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
        assert_eq!(dec!("10000"), output_resource.amount());

        let worktop_put_trace = traces.get(1).unwrap();
        assert_eq!(
            Origin::ScryptoMethod(ApplicationFnIdentifier {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: WORKTOP_BLUEPRINT.to_string(),
                ident: WORKTOP_PUT_IDENT.to_string(),
            }),
            worktop_put_trace.origin
        );
        assert!(worktop_put_trace.output.is_empty());
        assert!(worktop_put_trace.input.proofs.is_empty());
        assert_eq!(1, worktop_put_trace.input.buckets.len());
        let input_resource = worktop_put_trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_resource.resource_address());
        assert_eq!(dec!("10000"), input_resource.amount());

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
            Origin::ScryptoMethod(ApplicationFnIdentifier {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: WORKTOP_BLUEPRINT.to_string(),
                ident: WORKTOP_TAKE_ALL_IDENT.to_string(),
            }),
            trace.origin
        );

        assert!(trace.input.is_empty());
        assert!(trace.output.proofs.is_empty());
        assert_eq!(1, trace.output.buckets.len());

        let output_resource = trace.output.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, output_resource.resource_address());
        assert_eq!(dec!("10000"), output_resource.amount());
    }

    {
        // CREATE_PROOF_FROM_BUCKET
        let traces = traces_for_instruction(&child_traces, 3);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            Origin::ScryptoMethod(ApplicationFnIdentifier {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: BUCKET_BLUEPRINT.to_string(),
                ident: BUCKET_CREATE_PROOF_IDENT.to_string(),
            }),
            trace.origin
        );

        assert!(trace.input.is_empty());
        assert!(trace.output.buckets.is_empty());
        assert_eq!(1, trace.output.proofs.len());

        let output_proof = trace.output.proofs.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, output_proof.resource_address());
        assert_eq!(dec!("10000"), output_proof.amount());
    }

    {
        // DROP_PROOF
        let traces = traces_for_instruction(&child_traces, 4);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            Origin::ScryptoFunction(ApplicationFnIdentifier {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: PROOF_BLUEPRINT.to_string(),
                ident: PROOF_DROP_IDENT.to_string()
            }),
            trace.origin
        );

        assert!(trace.output.is_empty());
        assert!(trace.input.buckets.is_empty());
        assert_eq!(1, trace.input.proofs.len());

        let input_proof = trace.input.proofs.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_proof.resource_address());
        assert_eq!(dec!("10000"), input_proof.amount());
    }

    {
        // RETURN_TO_WORKTOP
        let traces = traces_for_instruction(&child_traces, 5);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            Origin::ScryptoMethod(ApplicationFnIdentifier {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: WORKTOP_BLUEPRINT.to_string(),
                ident: WORKTOP_PUT_IDENT.to_string(),
            }),
            trace.origin
        );
        assert!(trace.output.is_empty());
        assert!(trace.input.proofs.is_empty());
        assert_eq!(1, trace.input.buckets.len());

        let input_resource = trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(RADIX_TOKEN, input_resource.resource_address());
        assert_eq!(dec!("10000"), input_resource.amount());
    }

    {
        // CALL_FUNCTION: create_and_fund_a_component
        let traces = traces_for_instruction(&child_traces, 6);
        // Expected two traces: take from worktop and call scrypto function
        assert_eq!(2, traces.len());

        let take_trace = traces.get(0).unwrap();
        assert_eq!(
            Origin::ScryptoMethod(ApplicationFnIdentifier {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: WORKTOP_BLUEPRINT.to_string(),
                ident: WORKTOP_DRAIN_IDENT.to_string(),
            }),
            take_trace.origin
        );

        let call_trace = traces.get(1).unwrap();
        if let Origin::ScryptoFunction(ApplicationFnIdentifier {
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
        assert_eq!(dec!("10000"), input_resource.amount());
    }
}

#[test]
fn test_worktop_changes() {
    // Arrange
    fn return_to_worktop<'a>(
        builder: &'a mut ManifestBuilder,
        bucket: ManifestBucket,
    ) -> &'a mut ManifestBuilder {
        builder.return_to_worktop(bucket)
    }

    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);

    let fungible_resource = test_runner.create_fungible_resource(100.into(), 18, account);
    let non_fungible_resource = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .withdraw_from_account(account, fungible_resource, 100.into())
        .withdraw_non_fungibles_from_account(
            account,
            non_fungible_resource,
            &[
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]
            .into(),
        )
        .take_from_worktop(fungible_resource, return_to_worktop)
        .take_from_worktop_by_amount(20.into(), fungible_resource, return_to_worktop)
        .take_from_worktop(non_fungible_resource, return_to_worktop)
        .take_from_worktop_by_amount(2.into(), non_fungible_resource, return_to_worktop)
        .take_from_worktop_by_ids(
            &[
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(3),
            ]
            .into(),
            non_fungible_resource,
            return_to_worktop,
        )
        .call_method(
            account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    {
        receipt.expect_commit_success();

        let worktop_changes = receipt.execution_trace.worktop_changes();

        // Lock fee
        assert_eq!(worktop_changes.get(&0), None);

        // Withdraw fungible resource from account
        assert_eq!(
            worktop_changes.get(&1),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Amount(
                fungible_resource,
                100.into()
            ))])
        );

        // Withdraw non-fungible resource from account
        assert_eq!(
            worktop_changes.get(&2),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3),
                ]
                .into()
            ))])
        );

        // Take fungible resource from worktop (takes all)
        assert_eq!(
            worktop_changes.get(&3),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Amount(
                fungible_resource,
                100.into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&4),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Amount(
                fungible_resource,
                100.into()
            ))])
        );

        // Take fungible resource from worktop by amount
        assert_eq!(
            worktop_changes.get(&5),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Amount(
                fungible_resource,
                20.into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&6),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Amount(
                fungible_resource,
                20.into()
            ))])
        );

        // Take non-fungible from worktop (takes all)
        assert_eq!(
            worktop_changes.get(&7),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3),
                ]
                .into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&8),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3),
                ]
                .into()
            ))])
        );

        // Take non-fungible from worktop by amount
        assert_eq!(
            worktop_changes.get(&9),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                ]
                .into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&10),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                ]
                .into()
            ))])
        );

        // Take non-fungible from worktop by ids
        assert_eq!(
            worktop_changes.get(&11),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(3),
                ]
                .into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&12),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                [
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(3),
                ]
                .into()
            ))])
        );

        // Take all from worktop and deposit
        assert_eq!(
            worktop_changes.get(&13),
            Some(&vec![
                WorktopChange::Take(ResourceSpecifier::Amount(fungible_resource, 100.into())),
                WorktopChange::Take(ResourceSpecifier::Ids(
                    non_fungible_resource,
                    [
                        NonFungibleLocalId::integer(1),
                        NonFungibleLocalId::integer(2),
                        NonFungibleLocalId::integer(3),
                    ]
                    .into()
                )),
            ])
        );
    }
}

fn traces_for_instruction(
    traces: &Vec<ExecutionTrace>,
    instruction_index: usize,
) -> Vec<&ExecutionTrace> {
    traces
        .iter()
        .filter(|t| t.instruction_index == instruction_index)
        .collect()
}
