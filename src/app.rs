use crate::operation::{self, Operation};
use clap::Clap;
use monzo::client::QuickClient;
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
    operations: Vec<Operation>,
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

    pub async fn run(&self) -> Result<(), operation::Error> {
        for op in &self.operations {
            match op {
                Operation::Sweep(sweep) => sweep.run(&self.client).await?,
            }
        }

        Ok(())
    }
}
