use monzo::{client::QuickClient, AccountType, Pot};
use serde::Deserialize;
use std::{cmp::Ordering, io};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Monzo(#[from] monzo::Error),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub current_account_id: Option<String>,
    pub current_account_goal: Option<i64>,
    pub pots: Vec<String>,
}

impl Config {
    pub async fn run(&self, client: &QuickClient) -> Result<(), Error> {
        let account_id = self.find_current_account_id(client).await?;
        let balance = client.balance(&account_id).await?.balance();
        let active_pots: Vec<_> = client
            .pots(&account_id)
            .await?
            .into_iter()
            .filter(|pot| !pot.deleted)
            .collect();
        let pots = get_info(active_pots, &self.pots);

        let transactions = calculate_transactions(
            balance,
            self.current_account_goal.unwrap_or_default(),
            pots.as_slice(),
        );

        self.do_sweep(client, &account_id, &transactions).await?;
        send_report(client, &account_id, &transactions).await?;
        Ok(())
    }

    async fn do_sweep(
        &self,
        client: &QuickClient,
        account_id: &str,
        transactions: &[Transaction<'_>],
    ) -> Result<(), monzo::Error> {
        for (pot, amount) in transactions {
            if amount < &0 {
                client
                    .withdraw_from_pot(&pot.id, account_id, amount.abs())
                    .await?;
            } else {
                client.deposit_into_pot(&pot.id, account_id, *amount).await?;
            }
        }

        Ok(())
    }

    async fn find_current_account_id(&self, client: &QuickClient) -> Result<String, Error> {
        if let Some(id) = &self.current_account_id {
            Ok(id.to_string())
        } else {
            let id = client
                .accounts()
                .await?
                .into_iter()
                .find(|account| matches!(account.account_type, AccountType::UkRetail))
                .map(|account| account.id)
                .ok_or_else(|| {
                    Error::NotFound("unable to determine current account".to_string())
                })?;

            Ok(id)
        }
    }
}

async fn send_report(
    client: &QuickClient,
    account_id: &str,
    transactions: &[(&Pot, i64)],
) -> Result<(), monzo::Error> {
    let mut body = String::new();

    for (pot, amount) in transactions {
        body += &format!("transferred {} pence into {}", amount, &pot.name);
    }

    client
        .basic_feed_item(
            account_id,
            "Sweep Completed",
            "http://www.nyan.cat/cats/original.gif",
        )
        .body(&body)
        .send()
        .await
}

pub fn calculate_transactions(
    current_account_balance: i64,
    current_account_goal: i64,
    pots: &[Pot],
) -> Vec<Transaction> {
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

fn withdrawals(pots: &[Pot]) -> (Vec<Transaction>, Vec<Transaction>) {
    pots.iter()
        .filter(|pot| pot.diff() != 0)
        .map(|pot| (pot, pot.diff()))
        .partition(|(_pot, diff)| diff < &0)
}

fn get_info(mut pots: Vec<Pot>, pot_names: &[String]) -> Vec<Pot> {
    fn normalise(name: &str) -> String {
        let processed: String = name
            .chars()
            .filter(char::is_ascii)
            .map(|c| c.to_ascii_lowercase())
            .collect();
        processed.trim().to_string()
    }

    fn find_and_pop(pots: &mut Vec<Pot>, name: &str) -> Option<Pot> {
        let index = pots
            .iter()
            .position(|pot| normalise(&pot.name) == normalise(name))?;
        Some(pots.remove(index))
    }

    let mut info = Vec::default();

    for name in pot_names {
        let pot =
            find_and_pop(&mut pots, name).unwrap_or_else(|| panic!("failed to find pot: {}", name));

        info.push(pot);
    }

    info
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
    fn deserialise_toml() {
        let raw = r#"
        current_account_goal = 100

        pots = [
            "bills",
            "lottery",
            "allowance",
            "student loan",
            "savings",
        ]
        "#;

        let _: Config = toml::from_str(raw).unwrap();
    }

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

        let _: Config = serde_yaml::from_str(raw).unwrap();
    }
}
