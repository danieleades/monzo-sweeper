use crate::{
    operation,
    operation::{Op, Operation},
    state,
    transactions::{Ledger, Transactions},
};
use clap::Clap;
use futures_util::future::try_join_all;
use monzo::{client::QuickClient, Account, Pot};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    Monzo(#[from] monzo::Error),
    Io(#[from] io::Error),
    Yaml(#[from] serde_yaml::Error),
    Operation(#[from] operation::Error),
}

#[derive(Debug, Clap)]
struct Args {
    /// The API key
    #[clap(long, short = 't')]
    access_token: String,

    /// the path to the config file which defines the saving strategy
    #[clap(long, short)]
    config: PathBuf,
}

pub struct App {
    client: QuickClient,
    operations: Vec<Op>,
}

impl App {
    pub fn new(access_token: &str, config_path: &Path) -> Result<Self, Error> {
        let client = QuickClient::new(access_token);
        let operations = serde_yaml::from_reader(File::open(config_path)?)?;

        Ok(Self { client, operations })
    }

    pub fn from_args() -> Result<Self, Error> {
        let args = Args::parse();
        Self::new(&args.access_token, &args.config)
    }

    pub async fn run(&self) -> Result<(), Error> {
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

async fn process_transactions(
    client: &QuickClient,
    ledger: Ledger<'_>,
) -> Result<(), monzo::Error> {
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
    client: &'a QuickClient,
    account: &Account,
    transactions: Transactions<'a>,
) -> Result<(), monzo::Error> {
    try_join_all(
        transactions
            .withdrawals
            .into_iter()
            .map(|(pot, amount)| client.withdraw_from_pot(&pot.id, &account.id, amount.into())),
    )
    .await?;

    try_join_all(
        transactions
            .deposits
            .into_iter()
            .map(|(pot, amount)| client.deposit_into_pot(&pot.id, &account.id, amount.into())),
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
