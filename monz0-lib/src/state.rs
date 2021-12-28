//! Global accounts state

use std::collections::HashMap;

use monzo::{Balance, Pot};

/// The balance and pots associated with respective account ids
#[derive(Debug, Default)]
pub struct State {
    /// A map from account IDs to their respective [`state::Account`](Account)s
    pub accounts: HashMap<String, Account>,
}

/// The balance and pots associated with an account
#[derive(Debug)]
pub struct Account {
    /// the current balance of the account
    pub balance: Balance,

    /// the pots associated with an account
    pub pots: Vec<Pot>,
}
