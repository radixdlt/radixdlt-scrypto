#![no_main]

use std::path::PathBuf;

use scrypto_wasm_fuzzer::*;
use radix_engine_interface::{blueprints::resource::OwnerRole, metadata_init};
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;
use scrypto_test::prelude::*;

extern "C" {
    fn __sanitizer_cov_8bit_counters_init(start: *mut u8, end: *mut u8);
}

static mut COUNTERS : Option<Vec<u8>> = None;
static mut LEDGER : Option<LedgerSimulator<NoExtension, InMemorySubstateDatabase>> = None;
static mut PACKAGE_ADDRESS : Option<PackageAddress> = None;

#[no_mangle]
pub extern "C" fn LLVMFuzzerInitialize(_argc: *const u32, _argv: *const *const *const u8) -> u32 {
    let (code, definition) = build_for_fuzzing(PathBuf::from("fuzz_blueprint"));

    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, metadata_init!(), OwnerRole::None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "FuzzBlueprint", "get_counters_size", ())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let counters_len : usize = receipt.expect_commit_success().output(1);

    unsafe {
        COUNTERS = Some(vec![0; counters_len]);
        LEDGER = Some(ledger);
        PACKAGE_ADDRESS = Some(package_address);

        let start_ptr = COUNTERS.as_mut().unwrap().as_mut_ptr();
        let end_ptr = start_ptr.add(counters_len);
        __sanitizer_cov_8bit_counters_init(start_ptr, end_ptr);
    }
    0
}

#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, size: usize) -> u32 {
    let slice = unsafe {
        std::slice::from_raw_parts(data, size)
    };

    let data = slice.to_vec();
    let counters = unsafe {
        let manifest = ManifestBuilder::new()
            .call_function(PACKAGE_ADDRESS.unwrap(), "FuzzBlueprint", "fuzz", (data, ))
            .build();
        let receipt = LEDGER.as_mut().unwrap().preview_manifest(
            manifest,
            Default::default(),
    Default::default(),
            PreviewFlags {
                use_free_credit: true,
                assume_all_signature_proofs: true,
                skip_epoch_check: true,
                disable_auth: false,
        });

        let counters : Vec<u8> = receipt.expect_commit_success().output(0);
        counters
    };

    unsafe { 
        COUNTERS.as_mut().unwrap().iter_mut().zip(counters.iter()).for_each(|(a, b)| *a += b);
    }
    0
}
