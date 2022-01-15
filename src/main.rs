#![feature(derive_default_enum)]
#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

mod app;
mod config;
mod logging;
mod operation;

use app::App;

#[tokio::main]
async fn main() {
    let app = App::from_cli();

    if let Err(e) = app.run().await {
        println!("{}", e);
    }
}
