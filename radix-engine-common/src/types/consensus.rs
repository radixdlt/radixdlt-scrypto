use sbor::Sbor;

/// An index of a specific validator within the current validator set.
/// To be exact: a `ValidatorIndex` equal to `k` references the `k-th` element returned by the
/// iterator of the `IndexMap<ComponentAddress, Validator>` in this epoch's active validator set
/// (which is expected to be sorted by stake, descending).
/// This uniquely identifies the validator, while being shorter than `ComponentAddress` (we do care
/// about the constant factor of the space taken by `LeaderProposalHistory` under prolonged liveness
/// break scenarios).
pub type ValidatorIndex = u8;

/// A type-safe consensus epoch number.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
#[sbor(transparent)]
pub struct Epoch(u64);

impl Epoch {
    /// Creates a zero epoch (i.e. pre-genesis).
    pub fn zero() -> Self {
        Self::of(0)
    }

    /// Creates an epoch of the given number.
    pub fn of(number: u64) -> Self {
        Self(number)
    }

    /// Returns a raw epoch number.
    pub fn number(&self) -> u64 {
        self.0
    }

    /// Creates an epoch immediately following this one.
    /// Panics if this epoch's number is [`u64::MAX`] (such situation would indicate a bug or a
    /// deliberate harm meant by byzantine actors, since regular epoch progression should not reach
    /// such numbers within next thousands of years).
    pub fn next(&self) -> Self {
        self.after(1)
    }

    /// Creates an epoch following this one after the given number of epochs.
    /// Panics if the resulting number is greater than [`u64::MAX`] (such situation would indicate a
    /// bug or a deliberate harm meant by byzantine actors, since regular epoch delays configured by
    /// a network should not span thousands of years).
    pub fn after(&self, epoch_count: u64) -> Self {
        self.relative(epoch_count as i128)
    }

    /// Creates an epoch immediately preceding this one.
    /// Panics if this epoch's number is 0 (such situation would indicate a bug or a deliberate
    /// harm, since a legitimate genesis should not reference previous epochs).
    pub fn previous(&self) -> Self {
        self.relative(-1)
    }

    /// Creates an epoch of a number relative to this one.
    /// Panics if the resulting number does not fit within `u64` - please see the documentation of
    /// the callers for reasoning on why this should be safe in practice.
    /// Note: the internal callers of this private method only use [`epoch_count`]s representable
    /// by a signed 65-digits number (e.g. by casting `u64` as `i128`).
    fn relative(&self, epoch_count: i128) -> Self {
        let epoch_number = self.0 as i128; // every u64 is safe to represent as i128
        let relative_number = epoch_number
            .checked_add(epoch_count)
            .expect("both operands are representable by i65, so their sum must fit in i128");
        Self(u64::try_from(relative_number).unwrap_or_else(|error| {
            panic!(
                "cannot reference epoch {} + {} ({:?})",
                self.0, epoch_count, error
            )
        }))
    }
}

/// A type-safe consensus round number *within a single epoch*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
#[sbor(transparent)]
pub struct Round(u64);

impl Round {
    /// Creates a zero round (i.e. a state right after progressing to a next epoch).
    pub fn zero() -> Self {
        Self::of(0)
    }

    /// Creates a round of the given number.
    pub fn of(number: u64) -> Self {
        Self(number)
    }

    /// Returns a raw round number.
    pub fn number(&self) -> u64 {
        self.0
    }

    /// Returns a number of rounds between `from` and `to`, or `None` if there was no progress
    /// (i.e. their difference was not positive).
    pub fn calculate_progress(from: Round, to: Round) -> Option<u64> {
        let difference = (to.0 as i128) - (from.0 as i128);
        if difference <= 0 {
            None
        } else {
            Some(difference as u64) // if a difference of two u64 is positive, then it fits in u64
        }
    }
}
