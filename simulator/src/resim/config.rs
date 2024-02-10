use std::fs;
use std::path::PathBuf;

use radix_engine::types::*;
use radix_engine::utils::*;
use substate_stores_interface::db_key_mapper::*;
use substate_stores_interface::interface::*;

use crate::resim::*;
use std::env;

/// The environment that the simulator runs in.
pub struct SimulatorEnvironment {
    // Db
    pub db: RocksdbSubstateStore,
    // VMs
    pub scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    pub native_vm: DefaultNativeVm,
}

impl SimulatorEnvironment {
    pub fn new() -> Result<Self, Error> {
        // Create the database
        let db = RocksdbSubstateStore::standard(get_data_dir()?);

        // Create the VMs
        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
        let native_vm = DefaultNativeVm::new();

        let mut env = Self {
            db,
            scrypto_vm,
            native_vm,
        };
        env.bootstrap();

        Ok(env)
    }

    pub fn reset(self) -> Result<Self, Error> {
        drop(self);

        let dir = get_data_dir()?;
        std::fs::remove_dir_all(dir).map_err(Error::IOError)?;

        Self::new()
    }

    fn bootstrap(&mut self) {
        let vm = Vm::new(&self.scrypto_vm, self.native_vm.clone());

        // Bootstrap
        Bootstrapper::new(NetworkDefinition::simulator(), &mut self.db, vm, false)
            .bootstrap_test_default();

        // Run the protocol updates - unlike the test runner, the user has no way in whether they
        // get these protocol updates or not.
        {
            let state_updates = generate_seconds_precision_state_updates(&self.db);
            let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            self.db.commit(&db_updates);
        }
        {
            let state_updates = generate_vm_boot_scrypto_minor_version_state_updates();
            let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            self.db.commit(&db_updates);
        }
        {
            let state_updates = generate_pools_v1_1_state_updates(&self.db);
            let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            self.db.commit(&db_updates);
        }
    }
}

/// Simulator configurations.
#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct Configs {
    pub default_account: Option<ComponentAddress>,
    pub default_private_key: Option<String>,
    pub default_owner_badge: Option<NonFungibleGlobalId>,
    pub nonce: u32,
}

fn get_data_dir() -> Result<PathBuf, Error> {
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

pub fn get_default_private_key() -> Result<Secp256k1PrivateKey, Error> {
    get_configs()?
        .default_private_key
        .map(|v| Secp256k1PrivateKey::from_hex(&v).unwrap())
        .ok_or(Error::NoDefaultPrivateKey)
}

pub fn get_default_owner_badge() -> Result<NonFungibleGlobalId, Error> {
    get_configs()?
        .default_owner_badge
        .ok_or(Error::NoDefaultOwnerBadge)
}

pub fn get_nonce() -> Result<u32, Error> {
    Ok(get_configs()?.nonce)
}
