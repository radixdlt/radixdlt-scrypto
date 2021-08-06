use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use scrypto::buffer::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

pub struct FileBasedLedger {
    root: PathBuf,
}

const BLUEPRINTS: &'static str = "blueprints";
const COMPONENTS: &'static str = "components";
const ACCOUNTS: &'static str = "accounts";
const RESOURCES: &'static str = "resources";

impl FileBasedLedger {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn get_path<T: AsRef<str>>(&self, kind: &str, name: T, ext: &str) -> PathBuf {
        let mut path = self.root.clone();
        path.push(kind);
        path.push(name.as_ref());
        path.set_extension(ext);
        path
    }

    fn write<P: AsRef<Path>, T: AsRef<[u8]>>(path: P, value: T) {
        let p = path.as_ref();

        p.parent().map(|par| {
            fs::create_dir_all(par)
                .expect(format!("Failed to create directory: {:?}", par).as_str())
        });

        File::create(p)
            .expect(format!("Failed to create file: {:?}", p).as_str())
            .write_all(value.as_ref())
            .expect(format!("Failed to write file: {:?}", p).as_str());
    }

    fn read<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
        let p = path.as_ref();

        if p.exists() {
            Some(fs::read(p).expect(format!("Failed to read file: {:?}", p).as_str()))
        } else {
            None
        }
    }
}

impl Ledger for FileBasedLedger {
    fn get_blueprint(&self, address: Address) -> Option<Vec<u8>> {
        Self::read(self.get_path(BLUEPRINTS, address.to_string(), ".wasm"))
    }

    fn put_blueprint(&mut self, address: Address, blueprint: Vec<u8>) {
        Self::write(
            self.get_path(BLUEPRINTS, address.to_string(), ".wasm"),
            blueprint,
        )
    }

    fn get_resource(&self, address: Address) -> Option<ResourceInfo> {
        Self::read(self.get_path(RESOURCES, address.to_string(), ".json"))
            .map(|v| scrypto_decode(v.as_ref()).unwrap())
    }

    fn put_resource(&mut self, address: Address, info: ResourceInfo) {
        Self::write(
            self.get_path(RESOURCES, address.to_string(), ".json"),
            scrypto_encode(&info),
        )
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        Self::read(self.get_path(COMPONENTS, address.to_string(), ".json"))
            .map(|v| scrypto_decode(v.as_ref()).unwrap())
    }

    fn put_component(&mut self, address: Address, component: Component) {
        Self::write(
            self.get_path(COMPONENTS, address.to_string(), ".json"),
            scrypto_encode(&component),
        )
    }

    fn get_account(&self, address: Address) -> Option<Account> {
        Self::read(self.get_path(ACCOUNTS, address.to_string(), ".json"))
            .map(|v| scrypto_decode(v.as_ref()).unwrap())
    }

    fn put_account(&mut self, address: Address, account: Account) {
        Self::write(
            self.get_path(ACCOUNTS, address.to_string(), ".json"),
            scrypto_encode(&account),
        )
    }
}
