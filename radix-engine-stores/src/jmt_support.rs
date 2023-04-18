use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::{ModuleId, SubstateKey, SysModuleId};
use crate::interface::SubstateKeyMapper;

pub struct JmtKeyMapper;

impl SubstateKeyMapper for JmtKeyMapper {
    fn map_to_db_key(module_id: ModuleId, key: SubstateKey) -> Vec<u8> {
        let module_id = SysModuleId::from_repr(module_id.0).unwrap();
        let key = key.into();

        let bytes = match module_id {
            SysModuleId::Metadata | SysModuleId::Map | SysModuleId::Iterable => {
                hash(key).0[12..32].to_vec()
            }
            SysModuleId::Tuple
            | SysModuleId::AccessRules
            | SysModuleId::TypeInfo
            | SysModuleId::Royalty => key,
            SysModuleId::Sorted => {
                let mut bytes = key[0..2].to_vec(); // 2 bytes
                bytes.extend(hash(key[2..].to_vec()).0[12..32].to_vec()); // 20 bytes
                bytes
            }
        };
        bytes
    }
}