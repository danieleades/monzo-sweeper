//! Global accounts state

use std::collections::HashMap;

use monzo::{Balance, Pot};

/// A map from account IDs to their respective [`state::Account`](Account)s
pub type State = HashMap<String, Account>;

/// The balance and pots associated with an account
#[derive(Debug)]
pub struct Account {
    /// the current balance of the account
    pub balance: Balance,

    /// the pots associated with an account
    pub pots: Vec<Pot>,
}
