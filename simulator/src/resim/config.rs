use std::fs;
use std::path::PathBuf;

use radix_engine::types::*;

use crate::resim::*;
use std::env;

/// Simulator configurations.
#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct Configs {
    pub default_account: Option<ComponentAddress>,
    pub default_private_key: Option<String>,
    pub default_owner_badge: Option<NonFungibleGlobalId>,
    pub nonce: u64,
}

pub fn get_data_dir() -> Result<PathBuf, Error> {
    let path = match env::var(ENV_DATA_DIR) {
        Ok(value) => std::path::PathBuf::from(value),
        Err(..) => {
            let mut path = dirs::home_dir().ok_or(Error::HomeDirUnknown)?;
            path.push(DEFAULT_SCRYPTO_DIR_UNDER_HOME);
            path
        }
    };
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(Error::IOError)?;
    }
    Ok(path)
}

pub fn get_configs_path() -> Result<PathBuf, Error> {
    let mut path = get_data_dir()?;
    path.push("config");
    Ok(path.with_extension("sbor"))
}

pub fn get_configs() -> Result<Configs, Error> {
    let path = get_configs_path()?;
    if path.exists() {
        scrypto_decode(&fs::read(path).map_err(Error::IOError)?.as_ref())
            .map_err(Error::SborDecodeError)
    } else {
        Ok(Configs::default())
    }
}

pub fn set_configs(configs: &Configs) -> Result<(), Error> {
    fs::write(get_configs_path()?, scrypto_encode(configs).unwrap()).map_err(Error::IOError)
}

pub fn get_default_account() -> Result<ComponentAddress, Error> {
    get_configs()?
        .default_account
        .ok_or(Error::NoDefaultAccount)
}

pub fn get_default_private_key() -> Result<EcdsaSecp256k1PrivateKey, Error> {
    get_configs()?
        .default_private_key
        .map(|v| EcdsaSecp256k1PrivateKey::from_bytes(&hex::decode(&v).unwrap()).unwrap())
        .ok_or(Error::NoDefaultPrivateKey)
}

pub fn get_default_owner_badge() -> Result<NonFungibleGlobalId, Error> {
    get_configs()?
        .default_owner_badge
        .ok_or(Error::NoDefaultOwnerBadge)
}

pub fn get_nonce() -> Result<u64, Error> {
    Ok(get_configs()?.nonce)
}
