#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

use clap::Clap;

mod app;
use app::App;
use confy::ConfyError;
mod client;
mod logging;
mod operation;
mod state;
mod transactions;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load config")]
    Load(#[from] ConfyError),

    #[error("failed to run operations")]
    Run(#[from] operation::Error),
}

#[derive(Debug, Clap, Clone, Copy)]
pub struct Args {
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    logging::set_up(args.verbose);

    let app = App::from_config()?;
    if let Err(e) = app.run().await {
        println!("{}", e);
    }
    app.save_auth().await?;
    Ok(())
}
