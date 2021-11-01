use std::fs;
use std::path::PathBuf;

use sbor::*;
use scrypto::buffer::*;
use scrypto::types::*;

use crate::resim::*;

/// Radix Engine configurations.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Configs {
    pub default_account: Option<Address>,
    pub current_epoch: u64,
    pub nonce: u64,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            default_account: None,
            current_epoch: 0,
            nonce: 0,
        }
    }
}

/// Returns the data directory.
pub fn get_data_dir() -> Result<PathBuf, Error> {
    let mut path = dirs::home_dir().ok_or(Error::MissingHomeDirectory)?;
    path.push("scrypto-simulator");
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

/// Returns resim configurations.
pub fn get_configs() -> Result<Configs, Error> {
    let path = get_config_file()?;
    if path.exists() {
        Ok(
            scrypto_decode(&fs::read(path).map_err(Error::IOError)?.as_ref())
                .map_err(Error::InvalidConfig)?,
        )
    } else {
        Ok(Configs::default())
    }
}

/// Sets configurations.
pub fn set_configs(configs: Configs) -> Result<(), Error> {
    let path = get_config_file()?;
    fs::write(path, scrypto_encode(&configs)).map_err(Error::IOError)
}
