use crate::internal_prelude::*;

use crate::accounts::*;
#[derive(Clone, Debug)]
pub struct NextTransaction {
    pub logical_name: String,
    pub stage_counter: usize,
    pub transaction_manifest: UserTransactionManifest,
    pub subintent_manifests: Vec<UserSubintentManifest>,
    pub raw_transaction: RawNotarizedTransaction,
}

impl NextTransaction {
    pub fn validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedUserTransaction, ScenarioError> {
        self.raw_transaction.validate(validator).map_err(|err| {
            ScenarioError::TransactionValidationFailed(self.logical_name.clone(), err)
        })
    }
}

pub(crate) trait CompletableSubintentBuilder {
    type SignedPartialTransaction: Sized;
    fn complete(self, core: &mut ScenarioCore) -> Self::SignedPartialTransaction;
}

impl CompletableSubintentBuilder for PartialTransactionV2Builder {
    type SignedPartialTransaction = DetailedSignedPartialTransactionV2;
    fn complete(self, core: &mut ScenarioCore) -> Self::SignedPartialTransaction {
        core.complete_partial_transaction_v2(self)
    }
}

pub(crate) trait CompletableTransactionBuilder {
    fn complete(self, core: &mut ScenarioCore) -> Result<NextTransaction, ScenarioError>;
}

impl CompletableTransactionBuilder for TransactionV1Builder {
    fn complete(self, core: &mut ScenarioCore) -> Result<NextTransaction, ScenarioError> {
        core.complete_v1(self)
    }
}

impl CompletableTransactionBuilder for TransactionV2Builder {
    fn complete(self, core: &mut ScenarioCore) -> Result<NextTransaction, ScenarioError> {
        core.complete_v2(self)
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
    next_transaction_name: Option<String>,
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
            next_transaction_name: None,
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
        create_manifest: impl FnOnce(TransactionManifestV1Builder) -> TransactionManifestV1Builder,
        signers: Vec<&PrivateKey>,
    ) -> Result<NextTransaction, ScenarioError> {
        let builder = ManifestBuilder::new_v1()
            .lock_fee_from_faucet()
            .then(create_manifest);
        self.next_transaction_from_manifest_v1(logical_name, builder.build(), signers)
    }

    pub fn next_transaction_with_faucet_lock_fee_fallible(
        &mut self,
        logical_name: &str,
        create_manifest: impl FnOnce(
            TransactionManifestV1Builder,
        ) -> Result<TransactionManifestV1Builder, ScenarioError>,
        signers: Vec<&PrivateKey>,
    ) -> Result<NextTransaction, ScenarioError> {
        let mut builder = ManifestBuilder::new_v1().lock_fee_from_faucet();
        builder = create_manifest(builder)?;
        self.next_transaction_from_manifest_v1(logical_name, builder.build(), signers)
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
                    .try_deposit_or_abort(to_account, None, "free_xrd")
            },
            vec![],
        )
    }

    pub fn next_transaction_from_manifest_v1(
        &mut self,
        logical_name: &str,
        manifest: TransactionManifestV1,
        signers: Vec<&PrivateKey>,
    ) -> Result<NextTransaction, ScenarioError> {
        let mut builder = self.v1_transaction(logical_name).manifest(manifest);
        for signer in signers {
            builder = builder.sign(signer);
        }
        builder.complete(self)
    }

    pub fn v1_transaction(&mut self, transaction_name: impl Into<String>) -> TransactionV1Builder {
        let nonce = self.nonce;
        self.nonce += 1;
        self.next_transaction_name = Some(transaction_name.into());
        TransactionBuilder::new().header(TransactionHeaderV1 {
            network_id: self.network.id,
            start_epoch_inclusive: self.epoch,
            end_epoch_exclusive: self.epoch.next().unwrap(),
            nonce,
            notary_public_key: self.default_notary.public_key(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
    }

    pub fn complete_v1(
        &mut self,
        mut builder: TransactionV1Builder,
    ) -> Result<NextTransaction, ScenarioError> {
        let logical_name = self
            .next_transaction_name
            .take()
            .expect("Expected next transaction name to be set when the transaction was created");
        self.last_transaction_name = Some(logical_name.to_owned());

        builder = builder.notarize(&self.default_notary);
        let transaction = builder.build();
        let raw_transaction = transaction.to_raw().expect("Transaction could be encoded");
        let transaction_manifest = builder.into_manifest().into();
        Ok(NextTransaction {
            logical_name: logical_name.to_owned(),
            stage_counter: self.stage_counter,
            transaction_manifest,
            subintent_manifests: vec![],
            raw_transaction,
        })
    }

    /// A builder with headers configured.
    ///
    /// It's expected that the caller will:
    /// * Optionally add children
    /// * Add a manifest
    /// * Add any signatures
    ///
    /// The transaction will then be notarized at finalization time.
    ///
    /// ```ignore
    /// let child = core.v2_subintent()
    ///     .manifest(|manifest_builder| manifest_builder
    ///         .yield_to_parent(())
    ///     )
    ///     .finalize(core);
    /// core.v2_transaction()
    ///     .add_signed_child("child_1", signed_child)
    ///     .manifest(|manifest_builder| {
    ///         manifest_builder
    ///             .lock_fee_from_faucet()
    ///             .yield_to_child("child_1", ())
    ///     })
    ///     .sign(key)
    ///     .finalize(core)
    /// });
    /// ```
    pub fn v2_transaction(&mut self, transaction_name: impl Into<String>) -> TransactionV2Builder {
        self.v2_transaction_with_timestamp_range(transaction_name, None, None)
    }

    pub fn v2_transaction_with_timestamp_range(
        &mut self,
        transaction_name: impl Into<String>,
        min_proposer_timestamp_inclusive: Option<Instant>,
        max_proposer_timestamp_exclusive: Option<Instant>,
    ) -> TransactionV2Builder {
        let nonce = self.nonce;
        self.nonce += 1;
        self.next_transaction_name = Some(transaction_name.into());
        TransactionV2Builder::new()
            .intent_header(IntentHeaderV2 {
                network_id: self.network.id,
                start_epoch_inclusive: self.epoch,
                end_epoch_exclusive: self.epoch.next().unwrap(),
                min_proposer_timestamp_inclusive,
                max_proposer_timestamp_exclusive,
                intent_discriminator: nonce as u64,
            })
            .transaction_header(TransactionHeaderV2 {
                notary_public_key: self.default_notary.public_key(),
                notary_is_signatory: false,
                tip_basis_points: 0,
            })
    }

    pub fn complete_v2(
        &mut self,
        mut builder: TransactionV2Builder,
    ) -> Result<NextTransaction, ScenarioError> {
        let logical_name = self
            .next_transaction_name
            .take()
            .expect("Expected next transaction name to be set when the transaction was created");
        builder = builder.notarize(&self.default_notary);
        self.last_transaction_name = Some(logical_name.to_owned());
        let DetailedNotarizedTransactionV2 {
            transaction,
            raw: raw_transaction,
            object_names,
            ..
        } = builder.build();

        let (transaction_manifest, subintent_manifests) =
            transaction.extract_manifests_with_names(object_names);
        Ok(NextTransaction {
            logical_name,
            stage_counter: self.stage_counter,
            transaction_manifest,
            subintent_manifests,
            raw_transaction,
        })
    }

    /// For recommended usage, see the docs on [`v2_transaction`][`Self::v2_transaction`].
    pub fn v2_subintent(&mut self) -> PartialTransactionV2Builder {
        let nonce = self.nonce;
        self.nonce += 1;
        PartialTransactionV2Builder::new().intent_header(IntentHeaderV2 {
            network_id: self.network.id,
            start_epoch_inclusive: self.epoch,
            end_epoch_exclusive: self.epoch.next().unwrap(),
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
            intent_discriminator: nonce as u64,
        })
    }

    pub fn complete_partial_transaction_v2(
        &mut self,
        builder: PartialTransactionV2Builder,
    ) -> DetailedSignedPartialTransactionV2 {
        builder.build()
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

    pub fn decoder(&self) -> AddressBech32Decoder {
        AddressBech32Decoder::new(&self.network)
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
        match &receipt.result {
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
        match &receipt.result {
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

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for DescribedAddress {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        match self {
            DescribedAddress::Global(address) => address
                .contextual_format(f, context)
                .map_err(|_| fmt::Error),
            DescribedAddress::Internal(address) => address
                .contextual_format(f, context)
                .map_err(|_| fmt::Error),
            DescribedAddress::NonFungible(global_id) => global_id.contextual_format(f, context),
        }
    }
}

impl From<&PreallocatedAccount> for DescribedAddress {
    fn from(value: &PreallocatedAccount) -> Self {
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

#[derive(Clone, Debug)]
pub struct ScenarioMetadata {
    /// The logical name of the scenario:
    /// - This is used in Node genesis to specify which scenarios should be run
    /// - This should be spaceless as it will be used for a file path
    pub logical_name: &'static str,
    /// The minimal protocol version required to successfully run this scenario.
    pub protocol_min_requirement: ProtocolVersion,
    /// The maximal protocol version required to successfully run this scenario.
    pub protocol_max_requirement: ProtocolVersion,
    /// If set, this will run immediately after this protocol update on a testnet.
    /// Note that setting this will change the definition of the given protocol update,
    /// so shouldn't be changed once the protocol update is locked in.
    pub testnet_run_at: Option<ProtocolVersion>,
    /// This setting should be `true` for new scenarios, because new scenarios should
    /// not use pre-allocated account addresses which may already exist on-ledger,
    /// which could break scenario execution.
    ///
    /// A test validates adherence to this:
    /// * If `!safe_to_run_on_used_ledger` then `testnet_run_at` is not past genesis.
    /// * If `safe_to_run_on_used_ledger` then pre-allocated account/identity addresses
    ///   do not appear in well-known addresses.
    pub safe_to_run_on_used_ledger: bool,
}

pub trait ScenarioCreator: Sized + 'static + Send + Sync {
    type Config: Default;
    type State: Default;
    type Instance: ScenarioInstance + 'static;

    const METADATA: ScenarioMetadata;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance;
}

pub trait ScenarioCreatorObjectSafe: Send + Sync + 'static {
    fn metadata(&self) -> ScenarioMetadata;

    fn create(&self, core: ScenarioCore) -> Box<dyn ScenarioInstance>;
}

impl<T: ScenarioCreator> ScenarioCreatorObjectSafe for T {
    fn metadata(&self) -> ScenarioMetadata {
        Self::METADATA
    }

    fn create(&self, core: ScenarioCore) -> Box<dyn ScenarioInstance> {
        Box::new(Self::create_with_config_and_state(
            core,
            Default::default(),
            Default::default(),
        ))
    }
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
    next_commit_handler: Option<Box<TransactionCommitResultHandler<Config, State>>>,
    next_error_handler: Option<Box<TransactionErrorResultHandler<Config, State>>>,
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
            next_commit_handler: None,
            next_error_handler: None,
        }
    }

    pub fn on_next_transaction_commit(
        mut self,
        handler: impl Fn(&mut ScenarioCore, &Config, &mut State, &CommitResult) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.next_commit_handler = Some(Box::new(handler));
        self
    }

    pub fn successful_transaction(
        mut self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
    ) -> Self {
        let handler: Box<TransactionResultHandler<Config, State>> =
            match self.next_commit_handler.take() {
                Some(commit_handler) => Box::new(move |core, config, state, receipt| {
                    let commit_result = core.check_commit_success(&receipt)?;
                    commit_handler(core, config, state, commit_result)
                }),
                None => Box::new(|core, _, _, receipt| {
                    core.check_commit_success(&receipt)?;
                    Ok(())
                }),
            };
        self.transactions.push(ScenarioTransaction {
            creator: Box::new(creator),
            handler,
        });
        self
    }

    #[deprecated = "Prefer using on_next_transaction_commit(..) and successful_transaction(..) to reduce nesting"]
    pub fn successful_transaction_with_result_handler(
        self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
        handler: impl Fn(&mut ScenarioCore, &Config, &mut State, &CommitResult) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.on_next_transaction_commit(handler)
            .successful_transaction(creator)
    }

    pub fn on_next_transaction_error(
        mut self,
        handler: impl Fn(&mut ScenarioCore, &Config, &mut State, &RuntimeError) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.next_error_handler = Some(Box::new(handler));
        self
    }

    pub fn failed_transaction(
        mut self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
    ) -> Self {
        let handler: Box<TransactionResultHandler<Config, State>> =
            match self.next_error_handler.take() {
                Some(error_handler) => Box::new(move |core, config, state, receipt| {
                    let error = core.check_commit_failure(&receipt)?;
                    error_handler(core, config, state, error)
                }),
                None => Box::new(|core, _, _, receipt| {
                    core.check_commit_failure(&receipt)?;
                    Ok(())
                }),
            };
        self.transactions.push(ScenarioTransaction {
            creator: Box::new(creator),
            handler,
        });
        self
    }

    #[deprecated = "Prefer using on_next_transaction_error(..) and failed_transaction(..) to reduce nesting"]
    pub fn failed_transaction_with_error_handler(
        self,
        creator: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<NextTransaction, ScenarioError>
            + 'static,
        handler: impl Fn(&mut ScenarioCore, &Config, &mut State, &RuntimeError) -> Result<(), ScenarioError>
            + 'static,
    ) -> Self {
        self.on_next_transaction_error(handler)
            .failed_transaction(creator)
    }

    pub fn finalize(
        self,
        finalizer: impl Fn(&mut ScenarioCore, &Config, &mut State) -> Result<ScenarioOutput, ScenarioError>
            + 'static,
    ) -> Scenario<Config, State> {
        Scenario::<Config, State> {
            core: self.core,
            metadata: self.metadata,
            config: self.config,
            state: self.state,
            transactions: self.transactions,
            finalizer: Box::new(finalizer),
        }
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
type TransactionCommitResultHandler<Config, State> = dyn Fn(&mut ScenarioCore, &Config, &mut State, &CommitResult) -> Result<(), ScenarioError>
    + 'static;
type TransactionErrorResultHandler<Config, State> = dyn Fn(&mut ScenarioCore, &Config, &mut State, &RuntimeError) -> Result<(), ScenarioError>
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
    // and so have easier error propagation
    pub fn unwrap(&self) -> T {
        self.0.as_ref().map(Clone::clone).unwrap()
    }
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
