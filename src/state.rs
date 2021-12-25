use futures_util::future::try_join;
use monzo::{Balance, Pot};
use tracing::{event, instrument, Level};

use crate::client::Client;

pub struct State {
    pub balance: Balance,
    pub pots: Vec<Pot>,
}

#[instrument(name = "get current state", skip(client))]
pub async fn get(client: &Client, account_id: &str) -> Result<State, monzo::Error> {
    let balance_fut = client.balance(account_id);
    let pots_fut = client.pots(account_id);
    let (balance, pots) = try_join(balance_fut, pots_fut).await?;

    event!(Level::INFO, "recieved account data");

    Ok(State { balance, pots })
}
