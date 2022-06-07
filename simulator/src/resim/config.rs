use std::fs;
use std::path::PathBuf;

use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::types::*;
use transaction::signing::EcdsaPrivateKey;

use crate::resim::*;
use std::env;

/// Simulator configurations.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Configs {
    pub default_account: ComponentAddress,
    pub default_private_key: Vec<u8>,
    pub nonce: u64,
}

/// Returns the data directory.
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

/// Returns the config file.
pub fn get_config_file() -> Result<PathBuf, Error> {
    let mut path = get_data_dir()?;
    path.push("config");
    Ok(path.with_extension("sbor"))
}

pub fn get_configs() -> Result<Option<Configs>, Error> {
    let path = get_config_file()?;
    if path.exists() {
        Ok(Some(
            scrypto_decode(&fs::read(path).map_err(Error::IOError)?.as_ref())
                .map_err(Error::ConfigDecodingError)?,
        ))
    } else {
        Ok(None)
    }
}

pub fn set_configs(configs: &Configs) -> Result<(), Error> {
    let path = get_config_file()?;
    fs::write(path, scrypto_encode(configs)).map_err(Error::IOError)
}

pub fn get_default_account() -> Result<ComponentAddress, Error> {
    get_configs()?
        .ok_or(Error::NoDefaultAccount)
        .map(|config| config.default_account)
}

pub fn get_default_private_key() -> Result<EcdsaPrivateKey, Error> {
    get_configs()?
        .ok_or(Error::NoDefaultAccount)
        .map(|config| EcdsaPrivateKey::from_bytes(&config.default_private_key).unwrap())
}

pub fn get_nonce() -> Result<u64, Error> {
    Ok(get_configs()?.map(|c| c.nonce).unwrap_or(0))
}
