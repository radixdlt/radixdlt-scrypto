use crate::prelude::*;

/// A factory for Proofs that can create them (for testing) through multiple creation strategies
pub struct ProofFactory;

impl ProofFactory {
    pub fn create_fungible_proof<S>(
        resource_address: ResourceAddress,
        amount: Decimal,
        creation_strategy: CreationStrategy,
        env: &mut TestEnvironment<S>,
    ) -> Result<FungibleProof, RuntimeError>
    where
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        BucketFactory::create_fungible_bucket(resource_address, amount, creation_strategy, env)
            .and_then(|bucket| bucket.create_proof_of_all(env))
    }

    pub fn create_non_fungible_proof<I, D, S>(
        resource_address: ResourceAddress,
        non_fungibles: I,
        creation_strategy: CreationStrategy,
        env: &mut TestEnvironment<S>,
    ) -> Result<NonFungibleProof, RuntimeError>
    where
        I: IntoIterator<Item = (NonFungibleLocalId, D)>,
        D: ScryptoEncode,
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        BucketFactory::create_non_fungible_bucket(
            resource_address,
            non_fungibles,
            creation_strategy,
            env,
        )
        .and_then(|bucket| bucket.create_proof_of_all(env))
    }

    pub fn create_proof<S>(
        resource_specifier: FactoryResourceSpecifier,
        creation_strategy: CreationStrategy,
        env: &mut TestEnvironment<S>,
    ) -> Result<Proof, RuntimeError>
    where
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        BucketFactory::create_bucket(resource_specifier, creation_strategy, env)
            .and_then(|bucket| bucket.create_proof_of_all(env))
    }
}
