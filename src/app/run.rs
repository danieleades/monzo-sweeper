use clap::Parser;
use monz0_lib::{Client, Transactions};
use monzo::Pot;
use tracing::instrument;

use crate::{config, operation::Operation};

#[derive(Debug, Parser, Clone, Copy)]
pub struct Run {
    #[clap(long)]
    dry_run: bool,
}

impl Run {
    #[instrument(skip(self))]
    pub async fn run(self) -> anyhow::Result<()> {
        let client: Client = config::auth()?.into();
        let operations = config::operations()?;

        tracing::info!("config: {:#?}", &self);
        tracing::info!("operations: {:#?}", &operations);

        for op in &operations {
            let state = client.state(op.account_id()).await?;
            println!("Running {}, account: {}", op.name(), op.account_id());
            let transactions = op.transactions(&state)?;

            if self.dry_run {
                println!("{}", transactions_summary(op.account_id(), &transactions));
                println!("skipping execution ('dry-run' = true)");
            } else if transactions.is_empty() {
                println!("nothing to do ...");
            } else {
                println!("{}", transactions_summary(op.account_id(), &transactions));
                client.process_transactions(transactions).await?;
            }
        }

        config::save_auth(&client.auth().await)?;

        Ok(())
    }
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
