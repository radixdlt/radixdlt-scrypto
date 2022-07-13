#[macro_export]
macro_rules! type_from_entity_type {
    (EntityType::Resource) => {
        ResourceAddress
    };
    (EntityType::Package) => {
        PackageAddress
    };
    (EntityType::Component) => {
        ComponentAddress
    };
    (EntityType::AccountComponent) => {
        ComponentAddress
    };
    (EntityType::SystemComponent) => {
        ComponentAddress
    };
}

#[macro_export]
macro_rules! entity_type_id_from_entity_type {
    (EntityType::Resource) => {
        RESOURCE_ADDRESS_ENTITY_ID
    };
    (EntityType::Package) => {
        PACKAGE_ADDRESS_ENTITY_ID
    };
    (EntityType::Component) => {
        COMPONENT_ADDRESS_ENTITY_ID
    };
    (EntityType::AccountComponent) => {
        ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID
    };
    (EntityType::SystemComponent) => {
        SYSTEM_COMPONENT_ADDRESS_ENTITY_ID
    };
}

#[macro_export]
macro_rules! address {
    (EntityType::$entity_type: tt, [$last_byte: expr; 26]) => {
        address!(EntityType::$entity_type, $last_byte)
    };
    (EntityType::$entity_type: tt, $last_byte: expr) => {
        type_from_entity_type!(EntityType::$entity_type)([
            entity_type_id_from_entity_type!(EntityType::$entity_type),
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
            $last_byte,
        ])
    };
}
