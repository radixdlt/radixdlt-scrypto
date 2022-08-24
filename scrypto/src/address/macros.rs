#[macro_export]
macro_rules! construct_address {
    (EntityType::Resource, $($bytes:expr),*) => {
        ::scrypto::resource::ResourceAddress::Normal([$($bytes),*])
    };
    (EntityType::Package, $($bytes:expr),*) => {
        ::scrypto::component::PackageAddress::Normal([$($bytes),*])
    };
    (EntityType::NormalComponent, $($bytes:expr),*) => {
        ::scrypto::component::ComponentAddress::Normal([$($bytes),*])
    };
    (EntityType::AccountComponent, $($bytes:expr),*) => {
        ::scrypto::component::ComponentAddress::Account([$($bytes),*])
    };
    (EntityType::SystemComponent, $($bytes:expr),*) => {
        ::scrypto::component::ComponentAddress::System([$($bytes),*])
    };
}

#[macro_export]
macro_rules! address {
    (EntityType::$entity_type:tt, $last_byte:literal) => {
        ::scrypto::construct_address!(
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
        ::scrypto::construct_address!(
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
        ::scrypto::construct_address!(EntityType::$entity_type, $($bytes),*)
    };
}
