use monzo::{Account, Pot};
use std::{collections::HashMap, convert::TryInto};

#[derive(Debug, Default)]
pub struct Ledger<'a> {
    pub transactions: HashMap<&'a Account, Transactions<'a>>,
}

impl<'a> Ledger<'a> {
    pub fn push(&mut self, account: &'a Account, transaction: (&'a Pot, i64)) {
        self.transactions
            .entry(account)
            .or_default()
            .push(transaction);
    }
}

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
