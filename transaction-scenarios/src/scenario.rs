use radix_engine::errors::RuntimeError;
use transaction::errors::TransactionValidationError;
use transaction::manifest::decompiler::ManifestObjectNames;
use transaction::validation::{NotarizedTransactionValidator, TransactionValidator};

use crate::internal_prelude::*;

use crate::accounts::ed25519_account_1;

pub struct NextTransaction {
    pub logical_name: String,
    pub stage_counter: usize,
    pub manifest: TransactionManifestV1,
    pub naming: ManifestObjectNames,
    pub raw_transaction: RawNotarizedTransaction,
}

impl NextTransaction {
    pub fn of(
        logical_name: String,
        stage_counter: usize,
        naming: ManifestObjectNames,
        transaction: NotarizedTransactionV1,
    ) -> Self {
        let manifest = TransactionManifestV1::from_intent(&transaction.signed_intent.intent);
        Self {
            logical_name,
            stage_counter,
            manifest,
            naming,
            raw_transaction: transaction.to_raw().expect("Transaction could be encoded"),
        }
    }

    pub fn validate(
        &self,
        validator: &NotarizedTransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, ScenarioError> {
        validator
            .validate_from_raw(&self.raw_transaction)
            .map_err(|err| {
                ScenarioError::TransactionValidationFailed(self.logical_name.clone(), err)
            })
    }

    #[cfg(feature = "std")]
    pub fn dump_manifest(
        &self,
        dump_directory: &Option<std::path::PathBuf>,
        network: &NetworkDefinition,
    ) {
        use transaction::manifest::dumper::dump_manifest_to_file_system;

        let Some(directory_path) = dump_directory else {
            return;
        };
        let file_name = format!("{:03}--{}", self.stage_counter, self.logical_name);
        dump_manifest_to_file_system(
            &self.manifest,
            self.naming.clone(),
            directory_path,
            Some(&file_name),
            &network,
        )
        .unwrap()
    }
}

pub(crate) trait Completeable: Sized {
    fn done<E>(self) -> Result<Self, E>;
}

impl Completeable for ManifestBuilder {
    fn done<E>(self) -> Result<Self, E> {
        Ok(self)
    }
}

/// A core set of functionality and utilities common to every scenario
pub struct ScenarioCore {
    network: NetworkDefinition,
    epoch: Epoch,
    nonce: u32,
    default_notary: PrivateKey,
    last_transaction_name: Option<String>,
    stage_counter: usize,
}

impl ScenarioCore {
    pub fn new(network: NetworkDefinition, epoch: Epoch, starting_nonce: u32) -> Self {
        Self {
            network,
            epoch,
            nonce: starting_nonce,
            default_notary: ed25519_account_1().key,
            last_transaction_name: None,
            stage_counter: 0,
        }
    }

    pub fn next_stage(&mut self) -> usize {
        self.stage_counter += 1;
        self.stage_counter
    }

    pub fn next_transaction_with_faucet_lock_fee(
        &mut self,
        logical_name: &str,
        create_manifest: impl FnOnce(ManifestBuilder) -> ManifestBuilder,
        signers: Vec<&PrivateKey>,
    ) -> Result<NextTransaction, ScenarioError> {
        let builder = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .then(create_manifest);
        let object_names = builder.object_names();
        self.next_transaction(logical_name, builder.build(), object_names, signers)
    }

    pub fn next_transaction_with_faucet_lock_fee_fallible(
        &mut self,
        logical_name: &str,
        create_manifest: impl FnOnce(ManifestBuilder) -> Result<ManifestBuilder, ScenarioError>,
        signers: Vec<&PrivateKey>,
    ) -> Result<NextTransaction, ScenarioError> {
        let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
        builder = create_manifest(builder)?;
        let object_names = builder.object_names();
        self.next_transaction(logical_name, builder.build(), object_names, signers)
    }

    pub fn next_transaction_free_xrd_from_faucet(
        &mut self,
        to_account: ComponentAddress,
    ) -> Result<NextTransaction, ScenarioError> {
        self.next_transaction_with_faucet_lock_fee(
            "faucet-top-up",
            |builder| {
                builder
                    .get_free_xrd_from_faucet()
                    .take_all_from_worktop(XRD, "free_xrd")
                    .try_deposit_or_abort(to_account, "free_xrd")
            },
            vec![],
        )
    }

    pub fn next_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn next_transaction(
        &mut self,
        logical_name: &str,
        manifest: TransactionManifestV1,
        naming: ManifestObjectNames,
        signers: Vec<&PrivateKey>,
    ) -> Result<NextTransaction, ScenarioError> {
        let nonce = self.nonce;
        self.nonce += 1;
        let mut builder = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: self.network.id,
                start_epoch_inclusive: self.epoch,
                end_epoch_exclusive: self.epoch.next(),
                nonce,
                notary_public_key: self.default_notary.public_key(),
                notary_is_signatory: false,
                tip_percentage: 0,
            })
            .manifest(manifest);
        for signer in signers {
            builder = builder.sign(signer);
        }
        builder = builder.notarize(&self.default_notary);
        self.last_transaction_name = Some(logical_name.to_owned());
        Ok(NextTransaction::of(
            logical_name.to_owned(),
            self.stage_counter,
            naming,
            builder.build(),
        ))
    }

    pub fn finish_scenario(&self, output: ScenarioOutput) -> EndState {
        EndState {
            next_unused_nonce: self.nonce,
            output,
        }
    }

    pub fn network(&self) -> &NetworkDefinition {
        &self.network
    }

    pub fn encoder(&self) -> AddressBech32Encoder {
        AddressBech32Encoder::new(&self.network)
    }

    pub fn check_start(&self, previous: &Option<&TransactionReceipt>) -> Result<(), ScenarioError> {
        match previous {
            Some(_) => Err(ScenarioError::PreviousResultProvidedAtStart),
            None => Ok(()),
        }
    }

    pub fn check_previous<'a>(
        &self,
        previous: &Option<&'a TransactionReceipt>,
    ) -> Result<&'a TransactionReceipt, ScenarioError> {
        match previous {
            Some(previous) => Ok(previous),
            None => Err(ScenarioError::MissingPreviousResult),
        }
    }

    pub fn check_commit_success<'a>(
        &self,
        receipt: &'a TransactionReceipt,
    ) -> Result<&'a CommitResult, ScenarioError> {
        match &receipt.transaction_result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => Ok(c),
                TransactionOutcome::Failure(err) => Err(ScenarioError::TransactionFailed(
                    self.last_transaction_description(),
                    err.clone(),
                )),
            },
            TransactionResult::Reject(result) => Err(ScenarioError::TransactionRejected(
                self.last_transaction_description(),
                result.clone(),
            )),
            TransactionResult::Abort(result) => Err(ScenarioError::TransactionAborted(
                self.last_transaction_description(),
                result.clone(),
            )),
        }
    }

    pub fn check_commit_failure<'a>(
        &self,
        receipt: &'a TransactionReceipt,
    ) -> Result<&'a RuntimeError, ScenarioError> {
        match &receipt.transaction_result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => Err(ScenarioError::TransactionSucceeded),
                TransactionOutcome::Failure(err) => Ok(err),
            },
            TransactionResult::Reject(result) => Err(ScenarioError::TransactionRejected(
                self.last_transaction_description(),
                result.clone(),
            )),
            TransactionResult::Abort(result) => Err(ScenarioError::TransactionAborted(
                self.last_transaction_description(),
                result.clone(),
            )),
        }
    }

    pub fn last_transaction_description(&self) -> String {
        self.last_transaction_name.clone().unwrap_or("".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FullScenarioError {
    pub scenario: String,
    pub error: ScenarioError,
}

#[derive(Debug, Clone)]
pub enum ScenarioError {
    PreviousResultProvidedAtStart,
    MissingPreviousResult,
    TransactionSucceeded,
    TransactionFailed(String, RuntimeError),
    TransactionRejected(String, RejectResult),
    TransactionAborted(String, AbortResult),
    TransactionValidationFailed(String, TransactionValidationError),
    StateReadBeforeSet,
    Custom(String),
}

impl ScenarioError {
    pub fn into_full(self, scenario: &Box<dyn ScenarioInstance>) -> FullScenarioError {
        FullScenarioError {
            scenario: scenario.metadata().logical_name.to_owned(),
            error: self,
        }
    }
}

pub enum NextAction {
    Transaction(NextTransaction),
    Completed(EndState),
}

#[derive(Debug)]
pub struct EndState {
    pub next_unused_nonce: u32,
    pub output: ScenarioOutput,
}

#[derive(Debug)]
pub enum DescribedAddress {
    Global(GlobalAddress),
    Internal(InternalAddress),
    NonFungible(NonFungibleGlobalId),
}

impl From<&VirtualAccount> for DescribedAddress {
    fn from(value: &VirtualAccount) -> Self {
        Self::Global(value.address.clone().into())
    }
}

impl From<GlobalAddress> for DescribedAddress {
    fn from(value: GlobalAddress) -> Self {
        Self::Global(value)
    }
}

impl From<PackageAddress> for DescribedAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Global(value.into())
    }
}

impl From<ComponentAddress> for DescribedAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Global(value.into())
    }
}

impl From<ResourceAddress> for DescribedAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Global(value.into())
    }
}

impl From<InternalAddress> for DescribedAddress {
    fn from(value: InternalAddress) -> Self {
        Self::Internal(value)
    }
}

impl From<NonFungibleGlobalId> for DescribedAddress {
    fn from(value: NonFungibleGlobalId) -> Self {
        Self::NonFungible(value)
    }
}

#[derive(Debug)]
pub struct DescribedAddresses(pub IndexMap<String, DescribedAddress>);

impl DescribedAddresses {
    pub fn new() -> Self {
        Self(indexmap!())
    }

    pub fn add(mut self, descriptor: impl ToString, address: impl Into<DescribedAddress>) -> Self {
        self.0.insert(descriptor.to_string(), address.into());
        self
    }
}

#[derive(Clone)]
pub struct ScenarioMetadata {
    /// The logical name of the scenario:
    /// - This is used in Node genesis to specify which scenarios should be run
    /// - This should be spaceless as it will be used for a file path
    pub logical_name: &'static str,
}

pub trait ScenarioCreator: Sized {
    type Config: Default;
    type State: Default;

    fn create(core: ScenarioCore) -> Box<dyn ScenarioInstance> {
        Self::create_with_config_and_state(core, Default::default(), Default::default())
    }

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance>;
}

pub trait ScenarioInstance {
    fn metadata(&self) -> &ScenarioMetadata;

    /// Consumes the previous receipt, and gets the next transaction in the scenario.
    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError>;
}

pub struct ScenarioBuilder<Config, State> {
    core: ScenarioCore,
    metadata: ScenarioMetadata,
    config: Config,
    state: State,
    transactions: Vec<ScenarioTransaction<Config, State>>,
}

impl<Config: 'static, State: 'static> ScenarioBuilder<Config, State> {
    pub fn new(
        core: ScenarioCore,
        metadata: ScenarioMetadata,
        config: Config,
        start_state: State,
    ) -> Self {
        Self {
            core,
            metadata,
            config,
            state: start_state,
            transactions: vec![],
        }
    }

    /// Also checks that the transaction commits successfully
    pub fn successful_transaction(
        mut self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
    ) -> Self {
        self.transactions.push(ScenarioTransaction {
            creator: Box::new(creator),
            handler: Box::new(|core, _, _, receipt| {
                core.check_commit_success(&receipt)?;
                Ok(())
            }),
        });
        self
    }

    pub fn successful_transaction_with_result_handler(
        mut self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
        handler: impl Fn(&mut ScenarioCore, &Config, &mut State, &CommitResult) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.transactions.push(ScenarioTransaction {
            creator: Box::new(creator),
            handler: Box::new(
                move |core, config, state, receipt| -> Result<(), ScenarioError> {
                    let commit_result = core.check_commit_success(receipt)?;
                    handler(core, config, state, commit_result)
                },
            ),
        });
        self
    }

    pub fn failed_transaction_with_error_handler(
        mut self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
        handler: impl Fn(&mut ScenarioCore, &Config, &mut State, &RuntimeError) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.transactions.push(ScenarioTransaction {
            creator: Box::new(creator),
            handler: Box::new(
                move |core, config, state, receipt| -> Result<(), ScenarioError> {
                    let error = core.check_commit_failure(receipt)?;
                    handler(core, config, state, error)
                },
            ),
        });
        self
    }

    pub fn add_transaction_advanced(
        mut self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
        handler: impl Fn(
                &mut ScenarioCore,
                &Config,
                &mut State,
                &TransactionReceipt,
            ) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.transactions.push(ScenarioTransaction {
            creator: Box::new(creator),
            handler: Box::new(handler),
        });
        self
    }

    pub fn finalize(
        self,
        finalizer: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<ScenarioOutput, ScenarioError>
            + 'static,
    ) -> Box<dyn ScenarioInstance> {
        Box::new(Scenario::<Config, State> {
            core: self.core,
            metadata: self.metadata,
            config: self.config,
            state: self.state,
            transactions: self.transactions,
            finalizer: Box::new(finalizer),
        })
    }
}

pub struct Scenario<Config, State> {
    core: ScenarioCore,
    metadata: ScenarioMetadata,
    config: Config,
    state: State,
    transactions: Vec<ScenarioTransaction<Config, State>>,
    finalizer: Box<ScenarioFinalizer<Config, State>>,
}

pub struct ScenarioTransaction<Config, State> {
    creator: Box<TransactionCreator<Config, State>>,
    handler: Box<TransactionResultHandler<Config, State>>,
}

type TransactionCreator<Config, State> = dyn Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
    + 'static;
type TransactionResultHandler<Config, State> = dyn Fn(&mut ScenarioCore, &Config, &mut State, &TransactionReceipt) -> Result<(), ScenarioError>
    + 'static;
type ScenarioFinalizer<Config, State> = dyn Fn(&mut ScenarioCore, &Config, &mut State) -> Result<ScenarioOutput, ScenarioError>
    + 'static;

#[derive(Debug)]
pub struct ScenarioOutput {
    /// The `interesting_addresses` should be a list of addresses that the scenario touched/created,
    /// with a descriptor in lower_snake_case.
    pub interesting_addresses: DescribedAddresses,
}

impl<Config, State> ScenarioInstance for Scenario<Config, State> {
    fn metadata(&self) -> &ScenarioMetadata {
        &self.metadata
    }

    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        let core = &mut self.core;
        let next_transaction_index = core.next_stage() - 1;
        if next_transaction_index == 0 {
            core.check_start(&previous)?;
        } else {
            let receipt = core.check_previous(&previous)?;
            self.transactions[next_transaction_index - 1]
                .handler
                .as_ref()(core, &self.config, &mut self.state, receipt)?;
        }
        let next_action = if next_transaction_index < self.transactions.len() {
            let next_transaction = self.transactions[next_transaction_index].creator.as_ref()(
                core,
                &self.config,
                &mut self.state,
            )?;
            NextAction::Transaction(next_transaction)
        } else {
            let output = self.finalizer.as_ref()(core, &self.config, &mut self.state)?;
            NextAction::Completed(core.finish_scenario(output))
        };
        Ok(next_action)
    }
}

/// A helper class for transaction scenario state entries
pub(crate) struct State<T>(Option<T>);

impl<T> State<T> {
    #[allow(unused)]
    pub fn as_ref(&self) -> Result<&T, ScenarioError> {
        self.0.as_ref().ok_or(ScenarioError::StateReadBeforeSet)
    }

    pub fn set(&mut self, value: T) {
        self.0 = Some(value);
    }
}

impl<T: Clone> State<T> {
    pub fn get(&self) -> Result<T, ScenarioError> {
        self.0
            .as_ref()
            .map(Clone::clone)
            .ok_or(ScenarioError::StateReadBeforeSet)
    }

    // TODO - remove this when we create a better manifest builder which doesn't use callbacks,
    // and so have easier error propogation
    pub fn unwrap(&self) -> T {
        self.0.as_ref().map(Clone::clone).unwrap()
    }
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
