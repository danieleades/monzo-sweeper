use super::Pot;

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
    ///
    /// transactions with '0' value are ignored.
    pub fn push(&mut self, transaction: (&'a Pot, i64)) {
        let (pot, amount) = transaction;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        match Ord::cmp(&amount, &0) {
            std::cmp::Ordering::Less => self.withdrawals.push((pot, amount.abs() as u32)),
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Greater => self.deposits.push((pot, amount as u32)),
        }
    }

    /// Checks whether there are zero transactions in the ledger
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.withdrawals.is_empty() && self.deposits.is_empty()
    }

    /// Returns the total number of transactions
    pub fn len(&self) -> usize {
        self.withdrawals.len() + self.deposits.len()
    }
}

impl<'a> IntoIterator for &'a Transactions<'a> {
    type Item = (&'a Pot, i64);

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let withdrawals = self
            .withdrawals
            .iter()
            .map(|(pot, withdrawal)| (*pot, -i64::from(*withdrawal)));

        let deposits = self
            .deposits
            .iter()
            .map(|(pot, deposit)| (*pot, i64::from(*deposit)));

        withdrawals.chain(deposits)
    }
}

#[cfg(test)]
mod tests {
    use monzo::Pot;

    use super::Transactions;

    fn dummy_pot() -> Pot {
        let pot = r#"
        {
            "id": "pot_1234",
            "name": "Savings",
            "style": "teal",
            "balance": 10,
            "currency": "GBP",
            "goal_amount": 1000000,
            "type": "flexible_savings",
            "product_id": "XXX",
            "current_account_id": "acc_1234",
            "cover_image_url": "",
            "isa_wrapper": "ISA",
            "round_up": false,
            "round_up_multiplier": null,
            "is_tax_pot": false,
            "created": "2019-04-28T06:36:54.318Z",
            "updated": "2019-05-11T00:31:04.256Z",
            "deleted": false,
            "locked": false,
            "charity_id": "",
            "available_for_bills": false
        }
        "#;

        serde_yaml::from_str(pot).unwrap()
    }

    #[test]
    fn push() {
        let mut transactions = Transactions::default();
        let pot = dummy_pot();

        transactions.push((&pot, 100));
        assert!(transactions.deposits.len() == 1);

        transactions.push((&pot, -100));
        assert!(transactions.withdrawals.len() == 1);

        transactions.push((&pot, 0));
        assert!(transactions.len() == 2);
    }
}
