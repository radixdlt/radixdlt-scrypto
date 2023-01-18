/// Constructs an address.
#[macro_export]
macro_rules! construct_address {
    (EntityType::Resource, $($bytes:expr),*) => {
        radix_engine_interface::model::ResourceAddress::Normal([$($bytes),*])
    };
    (EntityType::Package, $($bytes:expr),*) => {
        radix_engine_interface::model::PackageAddress::Normal([$($bytes),*])
    };
    (EntityType::NormalComponent, $($bytes:expr),*) => {
        radix_engine_interface::model::ComponentAddress::Normal([$($bytes),*])
    };
    (EntityType::AccountComponent, $($bytes:expr),*) => {
        radix_engine_interface::model::ComponentAddress::Account([$($bytes),*])
    };
    (EntityType::EpochManager, $($bytes:expr),*) => {
        radix_engine_interface::model::ComponentAddress::EpochManager([$($bytes),*])
    };
    (EntityType::Clock, $($bytes:expr),*) => {
        radix_engine_interface::model::ComponentAddress::Clock([$($bytes),*])
    };
}

#[macro_export]
macro_rules! address {
    (EntityType::$entity_type:tt, $last_byte:literal) => {
        radix_engine_interface::construct_address!(
            EntityType::$entity_type,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            $last_byte
        )
    };
    (EntityType::$entity_type:tt, [$repeat_byte:literal; 26]) => {
        radix_engine_interface::construct_address!(
            EntityType::$entity_type,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte,
            $repeat_byte
        )
    };
    (EntityType::$entity_type:tt, $($bytes:literal),*) => {
        radix_engine_interface::construct_address!(EntityType::$entity_type, $($bytes),*)
    };
}
