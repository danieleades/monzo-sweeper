#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

mod app;
use confy::ConfyError;
mod client;
mod config;
mod logging;
mod operation;
mod state;
mod transactions;

use app::App;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load config")]
    Load(#[from] ConfyError),

    #[error("failed to run operations")]
    Run(#[from] operation::Error),
}

impl From<monzo::Error> for Error {
    fn from(e: monzo::Error) -> Self {
        operation::Error::from(e).into()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = App::from_cli();

    if let Err(e) = app.run().await {
        println!("{}", e);
    }
    Ok(())
}
