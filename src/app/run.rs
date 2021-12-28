use clap::Parser;
use monz0_lib::{Client, Ledger, Pot};
use tracing::instrument;

use crate::config;

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
            let state = client.state().await?;
            println!("Running {}", op.name());
            let ledger = op.transactions(&state)?;

            if self.dry_run {
                println!("{}", transactions_summary(&ledger));
                println!("skipping execution ('dry-run' = true)");
            } else if ledger.is_empty() {
                println!("nothing to do ...");
            } else {
                println!("{}", transactions_summary(&ledger));
                client.process_ledger(&ledger).await?;
            }
        }

        config::save_auth(&client.auth().await)?;

        Ok(())
    }
}

fn transactions_summary(ledger: &Ledger) -> String {
    let mut summary = String::new();

    for (account_id, transactions) in ledger {
        summary += &format!("{}:\n", account_id);

        for (pot, amount) in transactions {
            summary += &format!("{}: {}\n", &pot.name, &format_currency(pot, amount));
        }
    }

    summary
}

fn format_currency(pot: &Pot, amount: i64) -> String {
    let currency = rusty_money::iso::find(&pot.currency).expect("unexpected currency ISO code");
    let money = rusty_money::Money::from_minor(amount, currency);
    format!("{}", money)
}
