use crate::internal_prelude::*;
use crate::system::type_info::TypeInfoSubstate;
use radix_engine_interface::types::SubstateKey;

pub fn type_info_partition(info: TypeInfoSubstate) -> BTreeMap<SubstateKey, IndexedScryptoValue> {
    BTreeMap::from([(
        TypeInfoField::TypeInfo.into(),
        IndexedScryptoValue::from_typed(&info),
    )])
}
