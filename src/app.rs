use crate::{
    operation,
    operation::{Op, Operation},
    state,
};
use clap::Clap;
use monzo::{client::QuickClient, Pot};
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
            let transactions = op.transactions(&state)?;
            if transactions.is_empty() {
                println!("nothing to do ...");
            } else {
                println!("{}", transactions_summary(&transactions));
                process_transactions(&self.client, transactions).await?;
            }
        }

        Ok(())
    }
}

async fn process_transactions(
    client: &QuickClient,
    transactions: Vec<(&Pot, i64)>,
) -> Result<(), monzo::Error> {
    for (pot, amount) in transactions {
        let account_id = &pot.current_account_id;
        if amount < 0 {
            client
                .withdraw_from_pot(&pot.id, account_id, amount.abs())
                .await?;
        } else {
            client.deposit_into_pot(&pot.id, account_id, amount).await?;
        }
    }

    Ok(())
}

fn transactions_summary(transactions: &Vec<(&Pot, i64)>) -> String {
    let mut summary = String::new();

    for (pot, amount) in transactions {
        summary += &format!("{}: {}", &pot.name, &format_currency(pot, *amount));
    }

    summary
}

fn format_currency(pot: &Pot, amount: i64) -> String {
    let currency = rusty_money::iso::find(&pot.currency).unwrap();
    let money = rusty_money::Money::from_minor(amount, currency);
    format!("{}", money)
}
