use monzo::{Account, Pot};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Ledger<'a> {
    pub account: &'a Account,
    pub transactions: Transactions<'a>,
}

impl<'a> Ledger<'a> {
    pub fn new(account: &'a Account) -> Self {
        Self {
            account,
            transactions: Transactions::default(),
        }
    }

    pub fn push(&mut self, transaction: (&'a Pot, i64)) {
        self.transactions.push(transaction);
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct Transactions<'a> {
    pub withdrawals: Vec<(&'a Pot, u32)>,
    pub deposits: Vec<(&'a Pot, u32)>,
}

impl<'a> Transactions<'a> {
    fn push(&mut self, transaction: (&'a Pot, i64)) {
        let (pot, amount) = transaction;
        if amount < 0 {
            self.withdrawals
                .push((pot, amount.abs().try_into().unwrap()));
        } else {
            self.deposits.push((pot, amount.try_into().unwrap()));
        }
    }

    fn is_empty(&self) -> bool {
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
