use scrypto::prelude::*;

pub mod badge;
pub mod bucket;

package_init!(
    badge::blueprint::BadgeTest::describe(),
    bucket::blueprint::BucketTest::describe()
);
