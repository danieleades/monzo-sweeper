#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

mod app;
use app::App;
mod operation;
mod state;
mod transactions;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load config")]
    Load(#[from] app::Error),

    #[error("failed to run operations")]
    Run(#[from] operation::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = App::from_args()?;
    app.run().await?;
    Ok(())
}
