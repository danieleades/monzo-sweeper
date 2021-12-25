use clap::Parser;
use futures_util::future::try_join_all;
use monzo::Pot;
use tracing::{instrument, Level};

use crate::{
    client::Client, config, operation::Operation, state, transactions::Transactions, Error,
};

#[derive(Debug, Parser, Clone, Copy)]
pub struct Run;

impl Run {
    pub async fn run(self) -> Result<(), Error> {
        let client = config::auth()?.into();
        let operations = config::operations()?;

        for op in &operations {
            let state = state::get(&client, op.account_id()).await?;
            println!("Running {}, account: {}", op.name(), op.account_id());
            let transactions = op.transactions(&state)?;
            if transactions.is_empty() {
                println!("nothing to do ...");
            } else {
                println!("{}", transactions_summary(op.account_id(), &transactions));
                process_transactions(&client, op.account_id(), transactions).await?;
            }
        }

        config::save_auth(&client.auth().await)?;

        Ok(())
    }
}

#[instrument(skip(client, account_id, transactions))]
async fn process_transactions<'a>(
    client: &'a Client,
    account_id: &str,
    transactions: Transactions<'a>,
) -> Result<(), monzo::Error> {
    let withdrawals = transactions.withdrawals;
    let deposits = transactions.deposits;

    try_join_all(
        withdrawals
            .into_iter()
            .map(|(pot, amount)| client.withdraw_from_pot(&pot.id, account_id, amount)),
    )
    .await?;

    tracing::event!(Level::DEBUG, "processed withdrawals");

    try_join_all(
        deposits
            .into_iter()
            .map(|(pot, amount)| client.deposit_into_pot(&pot.id, account_id, amount)),
    )
    .await?;

    tracing::event!(Level::DEBUG, "processed deposits");

    Ok(())
}

fn transactions_summary(account_id: &str, transactions: &Transactions) -> String {
    let mut summary = String::new();

    summary += &format!("{}:\n", account_id);
    for (pot, amount) in transactions {
        summary += &format!("{}: {}\n", &pot.name, &format_currency(pot, amount));
    }

    summary
}

fn format_currency(pot: &Pot, amount: i64) -> String {
    let currency = rusty_money::iso::find(&pot.currency).unwrap();
    let money = rusty_money::Money::from_minor(amount, currency);
    format!("{}", money)
}
