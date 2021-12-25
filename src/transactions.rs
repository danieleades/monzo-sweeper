use monzo::Pot;
use std::{convert::TryInto, iter::FromIterator};

#[derive(Debug, Default)]
pub struct Transactions<'a> {
    pub withdrawals: Vec<(&'a Pot, u32)>,
    pub deposits: Vec<(&'a Pot, u32)>,
}

impl<'a> Transactions<'a> {
    pub fn push(&mut self, transaction: (&'a Pot, i64)) {
        let (pot, amount) = transaction;
        if amount < 0 {
            self.withdrawals
                .push((pot, amount.abs().try_into().unwrap()));
        } else {
            self.deposits.push((pot, amount.try_into().unwrap()));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.withdrawals.is_empty() && self.deposits.is_empty()
    }
}

impl<'a> IntoIterator for &'a Transactions<'a> {
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
    type Item = (&'a Pot, i64);

    fn into_iter(self) -> Self::IntoIter {
        let withdrawals = self
            .withdrawals
            .iter()
            .map(|(pot, amount)| (*pot, -i64::from(*amount)));

        let deposits = self
            .deposits
            .iter()
            .map(|(pot, amount)| (*pot, i64::from(*amount)));

        Box::new(withdrawals.chain(deposits))
    }
}

impl<'a> FromIterator<(&'a Pot, i64)> for Transactions<'a> {
    fn from_iter<T: IntoIterator<Item = (&'a Pot, i64)>>(iter: T) -> Self {
        let mut transactions = Self::default();
        for t in iter {
            transactions.push(t);
        }
        transactions
    }
}
