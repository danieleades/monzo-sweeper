use futures_util::future::{ready, try_join3, try_join_all};
use monzo::{client::QuickClient, Account, Balance, Pot};
use std::collections::HashMap;

pub struct State {
    pub accounts: Vec<Account>,
    pub balance: HashMap<String, Balance>,
    pub pots: HashMap<String, Vec<Pot>>,
}

pub async fn get(client: &QuickClient) -> Result<State, monzo::Error> {
    let accounts = client.accounts().await?;

    let account_data = try_join_all(accounts.iter().map(|account| {
        let balance = client.balance(&account.id);
        let pots = client.pots(&account.id);
        try_join3(ready(Ok(account.id.clone())), balance, pots)
    }))
    .await?;

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
