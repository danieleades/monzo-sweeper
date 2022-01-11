use super::Pot;

/// Represents a ledger of transactions (deposits and withdrawals) associated
/// with an account
#[derive(Debug, Default)]
pub struct Transactions {
    /// A list of withdrawals from pots into the current account
    pub withdrawals: Vec<(Pot, u32)>,

    /// A list of deposits from the current account into pots
    pub deposits: Vec<(Pot, u32)>,
}

impl Transactions {
    /// Add a transaction to the ledger
    pub fn push(&mut self, transaction: (Pot, i64)) {
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

impl<'a> IntoIterator for &'a Transactions {
    type Item = (&'a Pot, i64);

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.withdrawals.iter().map(|(pot, withdrawal)| (pot, -i64::from(*withdrawal)))
    }
}