use monzo::{inner_client::Quick, Account, Balance, Pot};
use serde::{Deserialize, Serialize};

mod auto_refresh;

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicAuth {
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
    pub async fn auth(&self) -> Auth {
        match self {
            Client::Quick(client) => Auth::Basic {
                access_token: client.access_token().to_string(),
            },
            Client::AutoRefresh(client) => Auth::Refreshable(client.auth().await),
        }
    }

    pub async fn accounts(&self) -> monzo::Result<Vec<Account>> {
        match self {
            Self::Quick(client) => client.accounts().await,
            Self::AutoRefresh(client) => client.accounts().await,
        }
    }

    pub async fn balance(&self, account_id: &str) -> monzo::Result<Balance> {
        match self {
            Self::Quick(client) => client.balance(account_id).await,
            Self::AutoRefresh(client) => client.balance(account_id).await,
        }
    }

    pub async fn pots(&self, account_id: &str) -> monzo::Result<Vec<Pot>> {
        match self {
            Self::Quick(client) => client.pots(account_id).await,
            Self::AutoRefresh(client) => client.pots(account_id).await,
        }
    }

    pub async fn withdraw_from_pot(
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

    pub async fn deposit_into_pot(
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
}
