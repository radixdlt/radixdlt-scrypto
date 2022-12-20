use radix_engine_interface::constants::SYSTEM_TOKEN;
use radix_engine_interface::crypto::PublicKey;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::model::*;

pub struct AuthModule;

// TODO: Integrate with AuthModule in radix-engine
impl AuthModule {
    pub fn system_role_non_fungible_address() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::U32(1))
    }

    pub fn validator_role_non_fungible_address() -> NonFungibleAddress {
        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::U32(0))
    }

    pub fn pk_non_fungibles(signer_public_keys: &[PublicKey]) -> Vec<NonFungibleAddress> {
        signer_public_keys
            .iter()
            .map(NonFungibleAddress::from_public_key)
            .collect()
    }
}
