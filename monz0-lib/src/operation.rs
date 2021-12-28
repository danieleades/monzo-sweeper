//! Operations on Monzo pots

use crate::{client::State, transactions::Ledger};

mod sweep;
pub use sweep::Sweep;
// mod ratio;
// pub use ratio::Ratio;

/// Represents an operation that may be applied to the pots of an account
pub trait Operation {
    /// The error type returned by the operation
    ///
    /// Different operations may use custom error types
    type Err: std::error::Error;

    /// The name of the operation. Used for logging and pretty-printing
    const NAME: &'static str;

    /// Given an account state, generate a list of transactions to apply to that
    /// account.
    ///
    /// # Errors
    ///
    /// This method may be fallible. It's up to specific implementations to
    /// define the appropriate error type for the operation.
    fn transactions<'a>(&'a self, state: &'a State) -> Result<Ledger, Self::Err>;
}
