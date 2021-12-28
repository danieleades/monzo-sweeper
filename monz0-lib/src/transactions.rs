use std::{collections::HashMap, iter::FromIterator};

use monzo::Pot;

/// Represents a ledger of transactions (deposits and withdrawals) associated
/// with their respective accounts
#[derive(Debug, Default)]
pub struct Ledger<'a> {
    transactions: HashMap<&'a str, Transactions<'a>>,
}

impl<'a> Ledger<'a> {
    /// Add a new transaction to the [`Ledger`]
    pub fn push(&mut self, account_id: &'a str, pot: &'a Pot, amount: i64) {
        self.transactions
            .entry(account_id)
            .or_default()
            .push((pot, amount));
    }

    /// Checks whether there are zero transactions in the ledger
    #[must_use]
    pub fn is_empty(&self) -> bool {
        if self.transactions.is_empty() {
            return true;
        }

        for transactions in self.transactions.values() {
            if !transactions.is_empty() {
                return false;
            }
        }

        true
    }
}

/// Represents a ledger of transactions (deposits and withdrawals) associated
/// with an account
#[derive(Debug, Default)]
pub struct Transactions<'a> {
    /// A list of withdrawals from pots into the current account
    pub withdrawals: Vec<(&'a Pot, u32)>,

    /// A list of deposits from the current account into pots
    pub deposits: Vec<(&'a Pot, u32)>,
}

impl<'a> Transactions<'a> {
    /// Add a transaction to the ledger
    pub fn push(&mut self, transaction: (&'a Pot, i64)) {
        let (pot, amount) = transaction;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if amount < 0 {
            self.withdrawals.push((pot, amount.abs() as u32));
        } else {
            self.deposits.push((pot, amount as u32));
        }
    }

    /// Checks whether there are zero transactions in the ledger
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.withdrawals.is_empty() && self.deposits.is_empty()
    }
}

impl<'a> IntoIterator for &'a Transactions<'a> {
    type Item = (&'a Pot, i64);

    type IntoIter = impl Iterator<Item = Self::Item> + 'a;

    fn into_iter(self) -> Self::IntoIter {
        let withdrawals = self
            .withdrawals
            .iter()
            .map(|(pot, amount)| (*pot, -i64::from(*amount)));

        let deposits = self
            .deposits
            .iter()
            .map(|(pot, amount)| (*pot, i64::from(*amount)));

        withdrawals.chain(deposits)
    }
}

impl<'a> IntoIterator for &'a Ledger<'a> {
    type Item = (&'a str, &'a Transactions<'a>);

    type IntoIter = impl Iterator<Item = Self::Item> + 'a;

    fn into_iter(self) -> Self::IntoIter {
        self.transactions
            .iter()
            .map(|(account_id, transactions)| (*account_id, transactions))
    }
}

impl<'a> FromIterator<(&'a Pot, i64)> for Ledger<'a> {
    fn from_iter<T: IntoIterator<Item = (&'a Pot, i64)>>(iter: T) -> Self {
        let mut ledger = Self::default();
        for (pot, amount) in iter {
            let account_id = &pot.current_account_id;
            ledger.push(account_id, pot, amount);
        }
        ledger
    }
}
