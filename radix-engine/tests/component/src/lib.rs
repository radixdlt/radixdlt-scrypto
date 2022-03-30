use scrypto::prelude::*;

pub mod auth_component;
pub mod auth_list_component;
pub mod chess;
pub mod component;
pub mod cross_component;
pub mod package;

package_init!(
    auth_component::blueprint::AuthComponent::describe(),
    auth_list_component::blueprint::AuthListComponent::describe(),
    chess::blueprint::Chess::describe(),
    component::blueprint::ComponentTest::describe(),
    cross_component::blueprint::CrossComponent::describe(),
    package::blueprint::PackageTest::describe()
);