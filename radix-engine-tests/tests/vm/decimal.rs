use radix_common::math::*;
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;
use std::env;

use trybuild;

#[test]
fn test_dec_macro_try_compile() {
    // Change CARGO_MANIFEST_DIR to tests/dec_macros, where the dec_macros test crate is located.
    // By default 'trybuild' crate uses current manifest dir, but it does not work with
    // radix-engine-tests dir (presumably too complicated set of dependencies and features).
    let manifest_dir = env::current_dir().unwrap().join("tests/dec_macros");
    env::set_var("CARGO_MANIFEST_DIR", &manifest_dir);

    // Also change the current dir to the 'dec_macros' dir.
    // Otherwise 'trybuild' will not be able to find files to compile.
    assert!(env::set_current_dir(manifest_dir).is_ok());

    let t = trybuild::TestCases::new();

    // Paths must be relative to the manifest_dir
    t.pass("src/dec_success.rs");
    t.compile_fail("src/dec_err_*.rs");
    t.pass("src/pdec_success.rs");
    t.compile_fail("src/pdec_err_*.rs");
}

#[test]
fn test_dec_macro_valid() {
    let x1 = dec!("1.1");
    assert_eq!(x1, Decimal::try_from("1.1").unwrap());

    let x2 = dec!("3138550867693340381917894711603833208051.177722232017256447");
    assert_eq!(x2, Decimal::MAX);

    let x2 = dec!(3138550867693340381917894711603833208051.177722232017256447);
    assert_eq!(x2, Decimal::MAX);

    let x3 = dec!("-3138550867693340381917894711603833208051.177722232017256448");
    assert_eq!(x3, Decimal::MIN);

    let x3 = dec!(-3138550867693340381917894711603833208051.177722232017256448);
    assert_eq!(x3, Decimal::MIN);

    const X1: Decimal = dec!("111111.10");
    assert_eq!(X1, Decimal::try_from("111111.10").unwrap());

    const X2: Decimal = dec!(-111);
    assert_eq!(X2, Decimal::try_from(-111).unwrap());

    const X3: Decimal = dec!(129);
    assert_eq!(X3, Decimal::try_from(129).unwrap());

    const X4: Decimal = dec!(-1_000_000);
    assert_eq!(X4, Decimal::try_from(-1_000_000).unwrap());

    static X5: Decimal = dec!(1);
    assert_eq!(X5, Decimal::ONE);

    static X6: Decimal = dec!(10);
    assert_eq!(X6, Decimal::TEN);

    static X7: Decimal = dec!(100);
    assert_eq!(X7, Decimal::ONE_HUNDRED);

    static X8: Decimal = dec!("0.1");
    assert_eq!(X8, Decimal::ONE_TENTH);

    static X9: Decimal = dec!("0.01");
    assert_eq!(X9, Decimal::ONE_HUNDREDTH);

    const X10: Decimal = dec!(1.1);
    assert_eq!(X10, Decimal::try_from("1.1").unwrap());

    const X11: Decimal = dec!(1.12313214124);
    assert_eq!(X11, Decimal::try_from("1.12313214124").unwrap());

    const X12: Decimal = dec!("3138550867693340381917894711603833208051.177722232017256447");
    assert_eq!(X12, Decimal::MAX);

    const X13: Decimal = dec!("-3138550867693340381917894711603833208051.177722232017256448");
    assert_eq!(X13, Decimal::MIN);

    const X14: Decimal = dec!("0.000000000000000048");
    assert_eq!(X14, Decimal::from_attos(I192::from(48)));
}

#[test]
fn test_pdec_macro_valid() {
    let x1 = pdec!("1.1");
    assert_eq!(x1, PreciseDecimal::try_from("1.1").unwrap());

    let x2 =
        pdec!("57896044618658097711785492504343953926634.992332820282019728792003956564819967");
    assert_eq!(x2, PreciseDecimal::MAX);

    let x2 = pdec!(57896044618658097711785492504343953926634.992332820282019728792003956564819967);
    assert_eq!(x2, PreciseDecimal::MAX);

    let x3 =
        pdec!("-57896044618658097711785492504343953926634.992332820282019728792003956564819968");
    assert_eq!(x3, PreciseDecimal::MIN);

    let x3 = pdec!(-57896044618658097711785492504343953926634.992332820282019728792003956564819968);
    assert_eq!(x3, PreciseDecimal::MIN);

    const X1: PreciseDecimal = pdec!("111111.10");
    assert_eq!(X1, PreciseDecimal::try_from("111111.10").unwrap());

    const X2: PreciseDecimal = pdec!(-111);
    assert_eq!(X2, PreciseDecimal::try_from(-111).unwrap());

    const X3: PreciseDecimal = pdec!(129);
    assert_eq!(X3, PreciseDecimal::try_from(129u128).unwrap());

    const X4: PreciseDecimal = pdec!(-1_000_000);
    assert_eq!(X4, PreciseDecimal::try_from(-1_000_000_i64).unwrap());

    static X5: PreciseDecimal = pdec!(1);
    assert_eq!(X5, PreciseDecimal::ONE);

    static X6: PreciseDecimal = pdec!(10);
    assert_eq!(X6, PreciseDecimal::TEN);

    static X7: PreciseDecimal = pdec!(100);
    assert_eq!(X7, PreciseDecimal::ONE_HUNDRED);

    static X8: PreciseDecimal = pdec!("0.1");
    assert_eq!(X8, PreciseDecimal::ONE_TENTH);

    static X9: PreciseDecimal = pdec!("0.01");
    assert_eq!(X9, PreciseDecimal::ONE_HUNDREDTH);

    const X10: PreciseDecimal = pdec!(21.1);
    assert_eq!(X10, PreciseDecimal::try_from("21.1").unwrap());

    const X11: PreciseDecimal = pdec!(0.12313214124);
    assert_eq!(X11, PreciseDecimal::try_from("0.12313214124").unwrap());

    const X12: PreciseDecimal =
        pdec!("57896044618658097711785492504343953926634.992332820282019728792003956564819967");
    assert_eq!(X12, PreciseDecimal::MAX);

    const X13: PreciseDecimal =
        pdec!("-57896044618658097711785492504343953926634.992332820282019728792003956564819968");
    assert_eq!(X13, PreciseDecimal::MIN);

    {
        let x = pdec!("0.000000000000000001");
        assert_eq!(x, PreciseDecimal::ONE_ATTO);
    }
}
#[test]
fn test_dec_macro_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("decimal"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "DecimalTest",
            "test_dec_macro",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    let output = result.outcome.expect_success();
    output[1].expect_return_value(&Decimal::from(6666));
}

#[test]
fn test_pdec_macro_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("decimal"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "DecimalTest",
            "test_pdec_macro",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    let output = result.outcome.expect_success();
    output[1].expect_return_value(&PreciseDecimal::from(6666));
}
