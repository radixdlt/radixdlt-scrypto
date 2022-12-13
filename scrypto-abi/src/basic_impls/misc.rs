use sbor::rust::borrow::Cow;
use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
use sbor::rust::cell::RefCell;
use sbor::rust::rc::Rc;
use sbor::{CustomTypeId, TypeId};

use crate::v2::*;

use_same_generic_schema!(T, &T, T);

impl<'a, X: CustomTypeId, B: ?Sized + 'a + ToOwned + Schema<X>> Schema<X> for Cow<'a, B> {
    const SCHEMA_TYPE_REF: TypeRef = <B>::SCHEMA_TYPE_REF;

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        <B>::get_local_type_data()
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        <B>::add_all_dependencies(aggregator)
    }
}

use_same_generic_schema!(T, Box<T>, T);
use_same_generic_schema!(T, Rc<T>, T);
use_same_generic_schema!(T, RefCell<T>, T);
