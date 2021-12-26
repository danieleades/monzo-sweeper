#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

mod app;
mod config;
mod logging;

use app::App;
use monz0_lib::operation;

#[tokio::main]
async fn main() {
    let app = App::from_cli();

    if let Err(e) = app.run().await {
        println!("{}", e);
    }
}
