use futures_util::future::try_join;
use monzo::{Account, Balance, Pot};
use tracing::{event, instrument, Level};

use crate::{client::Client, operation::Error};

pub struct State {
    pub account: Account,
    pub balance: Balance,
    pub pots: Vec<Pot>,
}

#[instrument(name = "get current state", skip(client))]
pub async fn get(client: &Client, account_id: &str) -> Result<State, Error> {
    let account = client
        .accounts()
        .await?
        .into_iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| Error::NotFound(account_id.to_string()))?;

    let balance_fut = client.balance(account_id);
    let pots_fut = client.pots(account_id);
    let (balance, pots) = try_join(balance_fut, pots_fut).await?;

    event!(Level::INFO, "recieved account data");

    Ok(State {
        account,
        balance,
        pots,
    })
}
