use radix_engine_common::math::*;
use radix_engine_interface::{dec, pdec};

#[test]
fn test_dec_macro() {
    let x1 = dec!("1.1");
    assert_eq!(x1, Decimal::try_from("1.1").unwrap());

    let x2 = dec!("3138550867693340381917894711603833208051.177722232017256447");
    assert_eq!(x2, Decimal::MAX);

    let x3 = dec!("-3138550867693340381917894711603833208051.177722232017256448");
    assert_eq!(x3, Decimal::MIN);

    const X1: Decimal = dec!("111111.10");
    assert_eq!(X1, Decimal::try_from("111111.10").unwrap());

    const X2: Decimal = dec!(-111);
    assert_eq!(X2, Decimal::try_from(-111).unwrap());

    const X3: Decimal = dec!(129u128);
    assert_eq!(X3, Decimal::try_from(129u128).unwrap());

    const X4: Decimal = dec!(-1_000_000_i64);
    assert_eq!(X4, Decimal::try_from(-1_000_000_i64).unwrap());
}

#[test]
fn test_pdec_macro() {
    let x1 = pdec!("1.1");
    assert_eq!(x1, PreciseDecimal::try_from("1.1").unwrap());

    let x2 =
        pdec!("57896044618658097711785492504343953926634.992332820282019728792003956564819967");
    assert_eq!(x2, PreciseDecimal::MAX);

    let x3 =
        pdec!("-57896044618658097711785492504343953926634.992332820282019728792003956564819968");
    assert_eq!(x3, PreciseDecimal::MIN);

    const X1: PreciseDecimal = pdec!("111111.10");
    assert_eq!(X1, PreciseDecimal::try_from("111111.10").unwrap());

    const X2: PreciseDecimal = pdec!(-111);
    assert_eq!(X2, PreciseDecimal::try_from(-111).unwrap());

    const X3: PreciseDecimal = pdec!(129u128);
    assert_eq!(X3, PreciseDecimal::try_from(129u128).unwrap());

    const X4: PreciseDecimal = pdec!(-1_000_000_i64);
    assert_eq!(X4, PreciseDecimal::try_from(-1_000_000_i64).unwrap());
}
