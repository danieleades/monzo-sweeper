use futures_util::future::{try_join, try_join_all};
use monzo::{Account, Balance, Pot};
use std::collections::HashMap;
use tracing::{event, instrument, Level};

use crate::client::Client;

pub struct State {
    pub accounts: Vec<Account>,
    pub balance: HashMap<String, Balance>,
    pub pots: HashMap<String, Vec<Pot>>,
}

#[instrument(name = "get current state", skip(client))]
pub async fn get(client: &Client) -> Result<State, monzo::Error> {
    let accounts = client.accounts().await?;
    event!(Level::INFO, "recieved accounts");

    let account_data = try_join_all(
        accounts
            .iter()
            .map(|account| fetch_account_data(client, &account.id)),
    )
    .await?;

    event!(Level::INFO, "recieved account data");

    let mut balance_map = HashMap::default();
    let mut pots_map = HashMap::default();
    for (account_id, balance, pots) in account_data {
        balance_map.insert(account_id.clone(), balance);
        pots_map.insert(account_id, pots);
    }

    Ok(State {
        accounts,
        balance: balance_map,
        pots: pots_map,
    })
}

#[instrument(skip(client))]
async fn fetch_account_data(
    client: &Client,
    account_id: &str,
) -> monzo::Result<(String, Balance, Vec<Pot>)> {
    let (balance, pots) = try_join(
        fetch_balance(client, account_id),
        fetch_pots(client, account_id),
    )
    .await?;

    event!(Level::INFO, "retrieved account data");

    Ok((account_id.to_string(), balance, pots))
}

async fn fetch_balance(client: &Client, account_id: &str) -> monzo::Result<Balance> {
    let balance = client.balance(account_id).await?;
    event!(Level::DEBUG, "retrieved account balance");
    event!(Level::TRACE, balance=?balance);
    Ok(balance)
}

async fn fetch_pots(client: &Client, account_id: &str) -> monzo::Result<Vec<Pot>> {
    client.pots(account_id).await
}
