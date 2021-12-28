use futures_util::future::{try_join, try_join_all};
use monzo::{inner_client::Quick, Balance, Pot};
use serde::{Deserialize, Serialize};
use tracing::{instrument, Level};

use crate::{
    state::{self, State},
    transactions::Transactions,
    Ledger,
};

mod auto_refresh;

#[derive(Debug, Serialize, Deserialize)]
struct BasicAuth {
    access_token: String,
}

/// The authentication details used by the [`Client`]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Auth {
    /// The credentials required for a refreshable client
    Refreshable(auto_refresh::Auth),

    /// Basic authentication
    Basic {
        /// Temporary API token
        access_token: String,
    },
}

impl Default for Auth {
    fn default() -> Self {
        Self::Basic {
            access_token: "ACCESS_TOKEN".to_string(),
        }
    }
}

#[derive(Debug)]
enum InnerClient {
    Quick(monzo::Client<Quick>),
    AutoRefresh(auto_refresh::Client),
}

/// A client to the Monzo API.
///
/// This client will 'auto-refresh' itself if it detects the its token has
/// expired, provided it is instantiated with the appropriate auth
/// ([`Auth::Refreshable`]).
#[derive(Debug)]
pub struct Client {
    inner_client: InnerClient,
}

impl From<Auth> for Client {
    fn from(auth: Auth) -> Self {
        let inner_client = match auth {
            Auth::Basic { access_token } => InnerClient::Quick(monzo::Client::new(access_token)),
            Auth::Refreshable(auth) => InnerClient::AutoRefresh(auth.into()),
        };
        Self { inner_client }
    }
}

impl Client {
    /// Return the authentication information associated with the client
    #[instrument(skip(self))]
    pub async fn auth(&self) -> Auth {
        match &self.inner_client {
            InnerClient::Quick(client) => Auth::Basic {
                access_token: client.access_token().to_string(),
            },
            InnerClient::AutoRefresh(client) => Auth::Refreshable(client.auth().await),
        }
    }

    /// List the monzo accounts
    #[instrument(skip(self))]
    pub async fn accounts(&self) -> monzo::Result<Vec<monzo::Account>> {
        match &self.inner_client {
            InnerClient::Quick(client) => client.accounts().await,
            InnerClient::AutoRefresh(client) => client.accounts().await,
        }
    }

    /// Retrieve the balance for the given account
    #[instrument(skip(self))]
    async fn balance(&self, account_id: &str) -> monzo::Result<Balance> {
        match &self.inner_client {
            InnerClient::Quick(client) => client.balance(account_id).await,
            InnerClient::AutoRefresh(client) => client.balance(account_id).await,
        }
    }

    /// Retrieve a list of [`Pot`]s associated with the given account
    #[instrument(skip(self))]
    async fn pots(&self, account_id: &str) -> monzo::Result<Vec<Pot>> {
        match &self.inner_client {
            InnerClient::Quick(client) => client.pots(account_id).await,
            InnerClient::AutoRefresh(client) => client.pots(account_id).await,
        }
    }

    #[instrument(skip(self))]
    async fn withdraw_from_pot(
        &self,
        pot_id: &str,
        destination_account_id: &str,
        amount: u32,
    ) -> monzo::Result<Pot> {
        match &self.inner_client {
            InnerClient::Quick(client) => {
                client
                    .withdraw_from_pot(pot_id, destination_account_id, amount)
                    .await
            }
            InnerClient::AutoRefresh(client) => {
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
        match &self.inner_client {
            InnerClient::Quick(client) => {
                client
                    .deposit_into_pot(pot_id, source_account_id, amount)
                    .await
            }
            InnerClient::AutoRefresh(client) => {
                client
                    .deposit_into_pot(pot_id, source_account_id, amount)
                    .await
            }
        }
    }

    /// Retrieve the current state of the given account
    #[instrument(skip(self))]
    async fn account_state(&self, account_id: &str) -> Result<state::Account, monzo::Error> {
        let balance_fut = self.balance(account_id);
        let pots_fut = self.pots(account_id);
        let (balance, pots) = try_join(balance_fut, pots_fut).await?;

        tracing::event!(Level::INFO, "recieved account data");

        Ok(state::Account { balance, pots })
    }

    /// Retrieve the current state of the given account
    #[instrument(skip(self))]
    pub async fn state(&self) -> Result<State, monzo::Error> {
        let mut state = State::default();
        for account in self.accounts().await? {
            let account_id = account.id;
            let account_state = self.account_state(&account_id).await?;
            state.accounts.insert(account_id, account_state);
        }

        Ok(state)
    }

    /// Complete the pot withdrawals and deposits described by the given
    /// [`Ledger`]
    #[instrument(skip(self))]
    pub async fn process_ledger(&self, ledger: &Ledger<'_>) -> Result<(), monzo::Error> {
        try_join_all(
            ledger.into_iter().map(|(account_id, transactions)| {
                self.process_transactions(account_id, transactions)
            }),
        )
        .await?;

        Ok(())
    }

    /// Complete the pot withdrawals and deposits described by the given
    /// account ID and [`Transactions`]
    #[instrument(skip(self, transactions))]
    async fn process_transactions(
        &self,
        account_id: &str,
        transactions: &Transactions<'_>,
    ) -> Result<(), monzo::Error> {
        let withdrawals = transactions.withdrawals.iter();
        let deposits = transactions.deposits.iter();

        try_join_all(withdrawals.map(|(pot, amount)| {
            self.withdraw_from_pot(&pot.id, &pot.current_account_id, *amount)
        }))
        .await?;

        tracing::event!(Level::DEBUG, "processed withdrawals");

        try_join_all(
            deposits.map(|(pot, amount)| {
                self.deposit_into_pot(&pot.id, &pot.current_account_id, *amount)
            }),
        )
        .await?;

        tracing::event!(Level::DEBUG, "processed deposits");

        Ok(())
    }
}
