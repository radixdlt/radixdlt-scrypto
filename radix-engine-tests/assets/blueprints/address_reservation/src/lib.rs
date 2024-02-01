use scrypto::prelude::*;

#[blueprint]
mod address_reservation {
    struct AddressReservation {}

    impl AddressReservation {
        pub fn create() -> Global<AddressReservation> {
            let (reservation, address) =
                Runtime::allocate_component_address(AddressReservation::blueprint_id());

            let global_address = Runtime::get_reservation_address(&reservation);
            let reserved: GlobalAddress = address.into();
            assert_eq!(reserved, global_address);

            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(reservation)
                .globalize()
        }
    }
}
