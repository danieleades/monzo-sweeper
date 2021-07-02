use crate::{
    operation::{util::find_current_account, Error, Operation},
    state::State,
    transactions::Ledger,
};
use monzo::Pot;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    #[serde(default)]
    pub current_account_id: Option<String>,
    #[serde(default)]
    pub current_account_goal: i64,
    pub pots: Vec<String>,
}

impl Operation for Sweep {
    fn name(&self) -> &'static str {
        "Sweep"
    }

    fn transactions<'a>(&'a self, state: &'a State) -> Result<Ledger<'a>, Error> {
        let account = find_current_account(&state.accounts, self.current_account_id.as_deref())?;
        let balance = state.balance.get(&account.id).unwrap().balance();
        let pots = sort_and_filter_pots(state.pots.get(&account.id).unwrap(), &self.pots)?;

        let transactions =
            calculate_transactions(balance, self.current_account_goal * 100, pots.as_slice());

        let mut ledger = Ledger::default();
        transactions
            .into_iter()
            .for_each(|t| ledger.push(account, t));

        Ok(ledger)
    }
}

fn calculate_transactions<'a>(
    current_account_balance: i64,
    current_account_goal: i64,
    pots: &[&'a Pot],
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

fn withdrawals<'a>(pots: &[&'a Pot]) -> (Vec<Transaction<'a>>, Vec<Transaction<'a>>) {
    pots.iter()
        .filter(|pot| pot.diff() != 0)
        .map(|pot| (*pot, pot.diff()))
        .partition(|(_pot, diff)| diff < &0)
}

fn sort_and_filter_pots<'a>(
    pots: &'a [Pot],
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

    let mut active_pots: Vec<_> = pots.iter().filter(|pot| !pot.deleted).collect();

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
    fn diff(&self) -> i64;
}

impl PotExt for Pot {
    fn diff(&self) -> i64 {
        self.goal_amount.unwrap() - self.balance
    }
}

#[cfg(test)]
mod tests {
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

        let _: Sweep = serde_yaml::from_str(raw).unwrap();
    }
}
