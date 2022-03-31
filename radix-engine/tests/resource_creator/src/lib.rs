use scrypto::prelude::*;

pub mod resource_creator;

package_init!(resource_creator::blueprint::ResourceCreator::describe());
