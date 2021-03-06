use std::collections::HashMap;

mod transactions;
use monzo::Pot;
pub use transactions::Transactions;

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
            .push(pot, amount);
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

impl<'a> IntoIterator for Ledger<'a> {
    type Item = (&'a str, Transactions<'a>);

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.transactions.into_iter()
    }
}

impl<'a> IntoIterator for &'a Ledger<'a> {
    type Item = (&'a str, &'a Transactions<'a>);

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.transactions
            .iter()
            .map(|(account_id, transactions)| (*account_id, transactions))
    }
}
