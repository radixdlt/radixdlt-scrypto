use scrypto::prelude::*;

pub mod cyclic_map;
pub mod lazy_map;
pub mod super_lazy_map;

package_init!(
    cyclic_map::blueprint::CyclicMap::describe(),
    lazy_map::blueprint::LazyMapTest::describe(),
    super_lazy_map::blueprint::SuperLazyMap::describe()
);
