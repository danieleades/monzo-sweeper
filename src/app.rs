use crate::{
    client::{Auth, Client},
    operation,
    operation::{Op, Operation},
    state,
    transactions::{Ledger, Transactions},
};
use futures_util::future::try_join_all;
use monzo::{Account, Pot};

static BIN_NAME: &str = std::env!("CARGO_PKG_NAME");

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    Monzo(#[from] monzo::Error),
    Operation(#[from] operation::Error),
}

pub struct App {
    client: Client,
    operations: Vec<Op>,
}

impl App {
    pub fn from_config() -> Result<Self, confy::ConfyError> {
        let auth: Auth = confy::load(BIN_NAME, "auth")?;
        let client = auth.into();
        let operations: Vec<Op> = confy::load(BIN_NAME, "config")?;

        Ok(Self { client, operations })
    }

    pub async fn save_auth(&self) -> Result<(), confy::ConfyError> {
        confy::store(BIN_NAME, "auth", self.client.auth().await)
    }

    pub async fn run(&self) -> Result<(), operation::Error> {
        for op in &self.operations {
            let state = state::get(&self.client).await?;
            println!("Running {}", op.name());
            let ledger = op.transactions(&state)?;
            if ledger.transactions.is_empty() {
                println!("nothing to do ...");
            } else {
                println!("{}", transactions_summary(&ledger));
                process_transactions(&self.client, ledger).await?;
            }
        }

        Ok(())
    }
}

async fn process_transactions(client: &Client, ledger: Ledger<'_>) -> Result<(), monzo::Error> {
    try_join_all(
        ledger
            .transactions
            .into_iter()
            .map(|(account, transactions)| process_account(client, account, transactions)),
    )
    .await?;

    Ok(())
}

async fn process_account<'a>(
    client: &'a Client,
    account: &Account,
    transactions: Transactions<'a>,
) -> Result<(), monzo::Error> {
    try_join_all(
        transactions
            .withdrawals
            .into_iter()
            .map(|(pot, amount)| client.withdraw_from_pot(&pot.id, &account.id, amount)),
    )
    .await?;

    try_join_all(
        transactions
            .deposits
            .into_iter()
            .map(|(pot, amount)| client.deposit_into_pot(&pot.id, &account.id, amount)),
    )
    .await?;

    Ok(())
}

fn transactions_summary(ledger: &Ledger) -> String {
    let mut summary = String::new();

    for (account, transactions) in &ledger.transactions {
        summary += &format!("{}:\n", account.description);
        for (pot, amount) in transactions {
            summary += &format!("{}: {}\n", &pot.name, &format_currency(pot, amount));
        }
    }

    summary
}

fn format_currency(pot: &Pot, amount: i64) -> String {
    let currency = rusty_money::iso::find(&pot.currency).unwrap();
    let money = rusty_money::Money::from_minor(amount, currency);
    format!("{}", money)
}
