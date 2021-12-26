use futures_util::future::{try_join, try_join_all};
use monzo::{inner_client::Quick, Balance, Pot};
use serde::{Deserialize, Serialize};
use tracing::{instrument, Level};

use crate::transactions::Transactions;

mod auto_refresh;

pub struct State {
    pub balance: Balance,
    pub pots: Vec<Pot>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BasicAuth {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Auth {
    Refreshable(auto_refresh::Auth),
    Basic { access_token: String },
}

impl Default for Auth {
    fn default() -> Self {
        Self::Basic {
            access_token: "ACCESS_TOKEN".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Client {
    Quick(monzo::Client<Quick>),
    AutoRefresh(auto_refresh::Client),
}

impl From<Auth> for Client {
    fn from(auth: Auth) -> Self {
        match auth {
            Auth::Basic { access_token } => Client::Quick(monzo::Client::new(access_token)),
            Auth::Refreshable(auth) => Client::AutoRefresh(auth.into()),
        }
    }
}

impl Client {
    #[instrument(skip(self))]
    pub async fn auth(&self) -> Auth {
        match self {
            Client::Quick(client) => Auth::Basic {
                access_token: client.access_token().to_string(),
            },
            Client::AutoRefresh(client) => Auth::Refreshable(client.auth().await),
        }
    }

    #[instrument(skip(self))]
    async fn balance(&self, account_id: &str) -> monzo::Result<Balance> {
        match self {
            Self::Quick(client) => client.balance(account_id).await,
            Self::AutoRefresh(client) => client.balance(account_id).await,
        }
    }

    #[instrument(skip(self))]
    async fn pots(&self, account_id: &str) -> monzo::Result<Vec<Pot>> {
        match self {
            Self::Quick(client) => client.pots(account_id).await,
            Self::AutoRefresh(client) => client.pots(account_id).await,
        }
    }

    #[instrument(skip(self))]
    async fn withdraw_from_pot(
        &self,
        pot_id: &str,
        destination_account_id: &str,
        amount: u32,
    ) -> monzo::Result<Pot> {
        match self {
            Self::Quick(client) => {
                client
                    .withdraw_from_pot(pot_id, destination_account_id, amount)
                    .await
            }
            Self::AutoRefresh(client) => {
                client
                    .withdraw_from_pot(pot_id, destination_account_id, amount)
                    .await
            }
        }
    }

    #[instrument(skip(self))]
    async fn deposit_into_pot(
        &self,
        pot_id: &str,
        source_account_id: &str,
        amount: u32,
    ) -> monzo::Result<Pot> {
        match self {
            Self::Quick(client) => {
                client
                    .deposit_into_pot(pot_id, source_account_id, amount)
                    .await
            }
            Self::AutoRefresh(client) => {
                client
                    .deposit_into_pot(pot_id, source_account_id, amount)
                    .await
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn state(&self, account_id: &str) -> Result<State, monzo::Error> {
        let balance_fut = self.balance(account_id);
        let pots_fut = self.pots(account_id);
        let (balance, pots) = try_join(balance_fut, pots_fut).await?;

        tracing::event!(Level::INFO, "recieved account data");

        Ok(State { balance, pots })
    }

    #[instrument(skip(self))]
    pub async fn process_transactions(
        &self,
        transactions: Transactions<'_>,
    ) -> Result<(), monzo::Error> {
        let withdrawals = transactions.withdrawals;
        let deposits = transactions.deposits;

        try_join_all(
            withdrawals.into_iter().map(|(pot, amount)| {
                self.withdraw_from_pot(&pot.id, &pot.current_account_id, amount)
            }),
        )
        .await?;

        tracing::event!(Level::DEBUG, "processed withdrawals");

        try_join_all(
            deposits.into_iter().map(|(pot, amount)| {
                self.deposit_into_pot(&pot.id, &pot.current_account_id, amount)
            }),
        )
        .await?;

        tracing::event!(Level::DEBUG, "processed deposits");

        Ok(())
    }
}
