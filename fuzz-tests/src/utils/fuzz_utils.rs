//! This module contains a set of utility functions used by some of the fuzz tests. Additionally,
//! there is a set of macros that are useful.

use radix_engine::errors::*;
use radix_engine::transaction::*;
use radix_engine_interface::blueprints::transaction_processor::*;

use transaction::prelude::*;

/// Checks if the manifest is preparable or not through [`manifest_can_be_prepared`]. If it is not
/// then a `break` is inserted. Otherwise, the manifest is returned. This is useful for cases when
/// we wish to break out of a loop if an unpreparable manifest is encountered.
#[macro_export]
macro_rules! break_if_manifest_is_unpreparable {
    ($manifest: expr) => {
        ::fuzz_tests::arbitrary_action_if_manifest_is_unpreparable!($manifest, break)
    };
}

/// Checks if the manifest is preparable or not through [`manifest_can_be_prepared`]. If it is not
/// then a `return` is inserted. Otherwise, the manifest is returned. This is useful in cases when
/// we wish to early return if an unpreparable manifest is encountered.
#[macro_export]
macro_rules! return_if_manifest_is_unpreparable {
    ($manifest: expr) => {
        ::fuzz_tests::arbitrary_action_if_manifest_is_unpreparable!($manifest, return)
    };
}

/// Checks if the manifest is preparable or not through [`manifest_can_be_prepared`]. If it is not
/// then a `continue` is inserted. Otherwise, the manifest is returned. This is useful for cases
/// when we wish to continue to the next iteration if an unpreparable manifest is encountered.
#[macro_export]
macro_rules! continue_if_manifest_is_unpreparable {
    ($manifest: expr) => {
        ::fuzz_tests::arbitrary_action_if_manifest_is_unpreparable!($manifest, continue)
    };
}

/// Checks if the manifest is preparable or not through [`manifest_can_be_prepared`]. If it is not
/// then whatever passed `$action` is performed.
#[macro_export]
macro_rules! arbitrary_action_if_manifest_is_unpreparable {
    (
        $manifest: expr,
        $action: block $(,)?
    ) => {{
        let manifest = $manifest;
        if !::fuzz_tests::utils::manifest_can_be_prepared(manifest.clone()) {
            $action
        } else {
            manifest
        }
    }};
    (
        $manifest: expr,
        $stmt: stmt $(,)?
    ) => {
        ::fuzz_tests::arbitrary_action_if_manifest_is_unpreparable!($manifest, { $stmt })
    };
}

/// Checks if the manifest can be prepared. A manifest that can't be prepared is one which exceeds
/// the maximum depth limit allowed in the Manifest SBOR codec.
pub fn manifest_can_be_prepared(manifest: TransactionManifestV1) -> bool {
    TestTransaction::new_from_nonce(manifest, 0xFF)
        .prepare()
        .is_ok()
}

/* Receipt handling */

/// Panics if the passed receipt contains a [`NativeRuntimeError::Trap`] in the receipt result. This
/// function takes an additional optional argument of the `index` to allow this function to be give
/// better panic messages when checking more than
pub fn panic_if_native_vm_trap<'r, R: Into<OneOrMore<'r, TransactionReceipt>>>(receipt: R) {
    let receipt = receipt.into();
    match receipt {
        OneOrMore::One(receipt) => {
            if receipt_contains_native_vm_trap(receipt) {
                panic!("Receipt contained a native-vm trap. Receipt: {receipt:?}")
            }
        }
        OneOrMore::More(receipts) => receipts.iter().enumerate().for_each(|(index, receipt)| {
            if receipt_contains_native_vm_trap(receipt) {
                panic!("Receipt at index {index} contained a native-vm trap. Receipt: {receipt:?}")
            }
        }),
    }
}

/// Returns [`true`] if the receipt has a [`NativeRuntimeError::Trap`].
pub fn receipt_contains_native_vm_trap(receipt: &TransactionReceipt) -> bool {
    matches!(
        get_receipt_error(receipt),
        Some(RuntimeError::VmError(VmError::Native(
            radix_engine::errors::NativeRuntimeError::Trap { .. }
        )))
    )
}

/// Returns the [`RuntimeError`] in the transaction if one is reported. A [`None`] is returned if no
/// [`RuntimeError`] is encountered in the receipt.
pub fn get_receipt_error(receipt: &TransactionReceipt) -> Option<&RuntimeError> {
    match receipt.result {
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Failure(ref error),
            ..
        })
        | TransactionResult::Reject(RejectResult {
            reason: RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(ref error),
        }) => Some(error),
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Success(..),
            ..
        })
        | TransactionResult::Abort(..)
        | TransactionResult::Reject(RejectResult { .. }) => None,
    }
}

/// Executes the passed function is the receipt was committed successfully returning a [`Some`]
/// variant with the returns of the callback function, otherwise [`None`] is returned.
pub fn map_if_commit_success<F, O>(receipt: &TransactionReceipt, callback: F) -> Option<O>
where
    F: Fn(&TransactionReceipt, &CommitResult, &[InstructionOutput]) -> O,
{
    match receipt.result {
        TransactionResult::Commit(
            ref commit_result @ CommitResult {
                outcome: TransactionOutcome::Success(ref outcome),
                ..
            },
        ) => Some(callback(receipt, commit_result, outcome)),
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Failure(..),
            ..
        })
        | TransactionResult::Abort(..)
        | TransactionResult::Reject(..) => None,
    }
}

/// Converts any typed value that implements [`ManifestEncode`] to a [`ManifestValue`] without
/// respecting the maximum depth limit.
pub fn to_manifest_value_ignoring_depth<T>(value: &T) -> ManifestValue
where
    T: ManifestEncode,
{
    const DEPTH_LIMIT: usize = usize::MAX;
    manifest_encode_with_depth_limit(value, DEPTH_LIMIT)
        .ok()
        .and_then(|encoded| manifest_decode_with_depth_limit(&encoded, DEPTH_LIMIT).ok())
        .expect("Impossible case!")
}

pub enum OneOrMore<'t, T> {
    One(&'t T),
    More(&'t [T]),
}

impl<'t, T> From<&'t T> for OneOrMore<'t, T> {
    fn from(value: &'t T) -> Self {
        Self::One(value)
    }
}

impl<'t, T> From<&'t [T]> for OneOrMore<'t, T> {
    fn from(value: &'t [T]) -> Self {
        Self::More(value)
    }
}
