use scrypto::prelude::*;

pub mod call;
pub mod context;

package_init!(call::blueprint::MoveTest::describe(), context::blueprint::CoreTest::describe());