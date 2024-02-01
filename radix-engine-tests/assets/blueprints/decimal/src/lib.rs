use scrypto::prelude::*;

#[blueprint]
mod cast_test {
    const GD: Decimal = dec!(2);
    const GP: PreciseDecimal = pdec!(2);

    struct DecimalTest {}

    impl DecimalTest {
        pub fn test_dec_macro() -> Decimal {
            const C: Decimal = dec!("1111.2222");
            static S: Decimal = dec!(2222.1111);

            C.checked_add(S)
                .unwrap()
                .checked_add(dec!(-0.3333))
                .unwrap()
                .checked_mul(GD)
                .unwrap()
        }

        pub fn test_pdec_macro() -> PreciseDecimal {
            const C: PreciseDecimal = pdec!("1111.2222");
            static S: PreciseDecimal = pdec!(2222.1111);

            C.checked_add(S)
                .unwrap()
                .checked_add(pdec!(-0.3333))
                .unwrap()
                .checked_mul(GP)
                .unwrap()
        }
    }
}
