use scrypto::prelude::*;

const PACKAGE_ADDRESS_PLACE_HOLDER: [u8; NodeId::LENGTH] = [
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
    0x66, 0x77, 0x88, 0x99, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
];

#[blueprint]
mod cross_package {

    struct Sample {}

    impl Sample {
        pub fn call_external_package() {
            let _: () = Runtime::call_function(
                PackageAddress::try_from(PACKAGE_ADDRESS_PLACE_HOLDER).unwrap(),
                "Sample",
                "callee",
                scrypto_args!(),
            );
        }

        pub fn callee() {}
    }
}
