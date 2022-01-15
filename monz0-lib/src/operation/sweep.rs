use std::cmp::Ordering;

use monzo::Pot;
use serde::{Deserialize, Serialize};

use crate::{ledger::Ledger, operation::Operation, State};

/// Errors that can occur when processing a [`Sweep`] operation
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// a [`NotFound`](Self::NotFound) is returned when an account or pot
    /// configured in the [`Sweep`] operation cannot be found in the monzo
    /// [`State`]
    #[error("not found: {0}")]
    NotFound(String),

    /// The [`Sweep`] operation can only be used with pots that have a goal
    /// amount set
    #[error("Pot '{0}' has no 'goal amount' set")]
    NoPotGoal(String),
}

/// A [`Sweep`] operation moves through a list of pots, sweeping any extra money
/// above the goal amount into the next pot down the list.
///
/// It is an error to sweep pots that do not have a goal amount set.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    /// The ID of the account to be swept
    #[serde(default)]
    current_account_id: String,

    /// The goal amount of the current account itself
    #[serde(default)]
    current_account_goal: i64,

    /// A list of names of pots that should be swept, in order
    ///
    /// When determining the pots, the names are normalised by removing emojis,
    /// normalising capitalisation, and then stripping any leading or trailing
    /// whitespace.
    pots: Vec<String>,
}

impl Operation for Sweep {
    type Err = Error;

    const NAME: &'static str = "Sweep";

    fn transactions<'a>(&'a self, state: &'a State) -> Result<Ledger<'a>, Self::Err> {
        let account_state = state.get(&self.current_account_id).ok_or_else(|| {
            Error::NotFound(format!("account {} not found", self.current_account_id))
        })?;
        let balance = account_state.balance.balance;

        let pots = sort_and_filter_pots(&self.current_account_id, &account_state.pots, &self.pots)?;

        let transactions = calculate_transactions(balance, self.current_account_goal * 100, pots);

        let mut ledger = Ledger::default();

        for (pot, amount) in transactions {
            ledger.push(&self.current_account_id, pot, amount);
        }

        Ok(ledger)
    }
}

fn calculate_transactions<'a>(
    current_account_balance: i64,
    current_account_goal: i64,
    pots: impl IntoIterator<Item = &'a Pot>,
) -> Vec<Transaction<'a>> {
    let (withdrawals, remainder) = withdrawals(pots);

    let total_withdrawals: i64 = withdrawals.iter().map(|(_pot, diff)| diff).sum();
    let mut spare_cash = current_account_balance - current_account_goal - total_withdrawals;

    let mut deposits = Vec::default();

    for (pot, diff) in remainder {
        if spare_cash <= 0 {
            break;
        }

        let deposit = match spare_cash.cmp(&diff) {
            Ordering::Less | Ordering::Equal => spare_cash,
            Ordering::Greater => diff,
        };

        spare_cash -= deposit;
        deposits.push((pot, deposit));
    }

    let mut transactions = Vec::default();
    transactions.extend(withdrawals);
    transactions.extend(deposits);
    transactions
}

type Transaction<'a> = (&'a Pot, i64);

/// Returns the set of [`Transaction`]s needed to shift the balance of each
/// [`Pot`] to its respective goal amount.
///
/// The results are partitioned into
/// withdrawals and deposits respectively. Note that withdrawals should always
/// be possible, but deposits are constrained by the available spare balance.
/// Zero-value transactions are ignored.
fn withdrawals<'a>(
    pots: impl IntoIterator<Item = &'a Pot>,
) -> (Vec<Transaction<'a>>, Vec<Transaction<'a>>) {
    pots.into_iter()
        .filter(|pot| pot.diff_unchecked() != 0)
        .map(|pot| {
            let diff = pot.diff_unchecked();
            (pot, diff)
        })
        .partition(|(_pot, diff)| diff < &0)
}

fn sort_and_filter_pots<'a>(
    account_id: &str,
    pots: &'a [monzo::Pot],
    pot_names: &'a [String],
) -> Result<Vec<&'a Pot>, Error> {
    fn normalise(name: &str) -> String {
        let processed: String = name
            .chars()
            .filter(char::is_ascii)
            .map(|c| c.to_ascii_lowercase())
            .collect();
        processed.trim().to_string()
    }

    // Filter out any pots that are 'deleted' or where the account id doesn't match
    // the configured one
    let mut active_pots = pots
        .iter()
        // Filter out any pots that are 'deleted'
        .filter(|pot| !pot.deleted)
        // Filter out any pots where the account id doesn't match
        .filter(|pot| pot.current_account_id == account_id)
        // Check the pot has a goal set
        .map(|pot| {
            match pot.goal_amount {
                Some(_) => Ok(pot),
                None => Err(Error::NoPotGoal(pot.name.clone())),
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let mut info = Vec::default();

    for name in pot_names {
        let index = active_pots
            .iter()
            .position(|pot| normalise(&pot.name) == normalise(name))
            .ok_or_else(|| Error::NotFound(format!("failed to find pot: {}", name)))?;

        info.push(active_pots.remove(index));
    }

    Ok(info)
}

trait PotExt {
    /// the difference between a [`Pot`]s current balance and its goal balance
    ///
    /// # Panics
    ///
    /// This method will panic if the [`Pot`] doesn't have a goal amount set
    fn diff_unchecked(&self) -> i64;
}

impl PotExt for Pot {
    fn diff_unchecked(&self) -> i64 {
        self.goal_amount.unwrap() - self.balance
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test]
    fn deserialise_yaml() {
        let raw = r#"
        current_account_goal: 10000

        pots:
         - bills
         - lottery
         - allowance
         - student loan
         - savings
        "#;

        serde_yaml::from_str::<Sweep>(raw).unwrap();
    }

    #[test_case("ACCOUNT_ID", &[], &[] => Ok(vec![]); "no op")]
    fn sort_and_filter_pots<'a>(
        account_id: &'a str,
        pots: &'a [monzo::Pot],
        pot_names: &'a [String],
    ) -> Result<Vec<&'a Pot>, Error> {
        super::super::sort_and_filter_pots(account_id, pots, pot_names)
    }
}
