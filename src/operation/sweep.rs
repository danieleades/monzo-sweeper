use monzo::{Account, AccountType, Pot};
use serde::Deserialize;
use std::cmp::Ordering;

use crate::state::State;

use super::Operation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Monzo(#[from] monzo::Error),

    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    #[serde(default)]
    pub current_account_id: Option<String>,
    pub current_account_goal: Option<i64>,
    pub pots: Vec<String>,
}

impl Operation for Sweep {
    type Err = Error;

    fn name(&self) -> &'static str {
        "Sweep"
    }

    fn transactions<'a>(&'a self, state: &'a State) -> Result<Vec<(&'a Pot, i64)>, Self::Err> {
        let account_id = self.find_current_account_id(&state.accounts)?;
        let balance = state.balance.get(account_id).unwrap().balance();
        let pots = sort_and_filter_pots(state.pots.get(account_id).unwrap(), &self.pots)?;

        let transactions = calculate_transactions(
            balance,
            self.current_account_goal
                .map(|f| f * 100)
                .unwrap_or_default(),
            pots.as_slice(),
        );

        Ok(transactions)
    }
}

impl Sweep {
    fn find_current_account_id<'a>(&'a self, accounts: &'a [Account]) -> Result<&'a String, Error> {
        if let Some(id) = &self.current_account_id {
            Ok(id)
        } else {
            let id = accounts
                .iter()
                .find(|account| matches!(account.account_type, AccountType::UkRetail))
                .map(|account| &account.id)
                .ok_or_else(|| {
                    Error::NotFound("unable to determine current account".to_string())
                })?;

            Ok(id)
        }
    }
}

// async fn send_report(
//     client: &QuickClient,
//     account_id: &str,
//     transactions: &[(&Pot, i64)],
// ) -> Result<(), monzo::Error> {
//     let mut body = String::new();

//     for (pot, amount) in transactions {
//         body += &format!("{}: {}\n", &pot.name, format_currency(pot,
// *amount));     }

//     client
//         .basic_feed_item(
//             account_id,
//             "Sweep Completed",
//             "http://www.nyan.cat/cats/original.gif",
//         )
//         .body(&body)
//         .send()
//         .await
// }

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
