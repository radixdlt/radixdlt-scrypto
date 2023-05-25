use sbor::Sbor;

/// A type-safe consensus epoch number.
/// Assuming one epoch per minute (i.e. much faster progression than designed), this gives us time
/// until A.D. ~5700.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
#[sbor(transparent)]
pub struct Epoch(u32);

/// An index of a specific validator within the current validator set.
/// To be exact: a `ValidatorIndex` equal to `k` references the `k-th` element returned by the
/// iterator of the `IndexMap<ComponentAddress, Validator>` in this epoch's active validator set
/// (which is expected to be sorted by stake, descending).
/// This uniquely identifies the validator, while being shorter than `ComponentAddress` (we do care
/// about the constant factor of the space taken by `LeaderProposalHistory` under prolonged liveness
/// break scenarios).
pub type ValidatorIndex = u8;

impl Epoch {
    /// Creates a zero epoch (i.e. pre-genesis).
    pub fn zero() -> Self {
        Self::of(0)
    }

    /// Creates an epoch of the given number.
    pub fn of(number: u32) -> Self {
        Self(number)
    }

    /// Returns a raw epoch number.
    pub fn number(&self) -> u32 {
        self.0
    }

    /// Creates an epoch immediately following this one.
    pub fn next(&self) -> Self {
        self.after(1)
    }

    /// Creates an epoch following this one after the given number of epochs.
    pub fn after(&self, epoch_count: u32) -> Self {
        self.relative(epoch_count as i64)
    }

    /// Creates an epoch immediately preceding this one.
    pub fn previous(&self) -> Self {
        self.relative(-1)
    }

    /// Creates an epoch of a number relative to this one.
    fn relative(&self, epoch_count: i64) -> Self {
        Self(u32::try_from(self.0 as i64 + epoch_count).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
#[sbor(transparent)]
pub struct Round(u32);

/// A type-safe consensus round number *within a single epoch*.
/// Assuming one round per millisecond (i.e. much faster progression than designed), this gives us
/// time a maximum epoch duration od ~23 days (i.e. much longer than designed).
impl Round {
    /// Creates a zero round (i.e. a state right after progressing to a next epoch).
    pub fn zero() -> Self {
        Self::of(0)
    }

    /// Creates a round of the given number.
    pub fn of(number: u32) -> Self {
        Self(number)
    }

    /// Returns a raw round number.
    pub fn number(&self) -> u32 {
        self.0
    }

    /// Returns a number of rounds between `from` and `to`, or `None` if there was no progress
    /// (i.e. their difference was not positive).
    pub fn calculate_progress(from: Round, to: Round) -> Option<u32> {
        let difference = (to.0 as i64) - (from.0 as i64);
        if difference <= 0 {
            None
        } else {
            Some(difference as u32)
        }
    }
}
