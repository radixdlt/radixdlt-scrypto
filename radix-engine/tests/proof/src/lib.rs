use scrypto::prelude::*;

pub mod bucket_proof;
pub mod vault_proof;

package_init!(bucket_proof::blueprint::BucketProof::describe(), vault_proof::blueprint::VaultProof::describe());