use crate::{state::State, transactions::Transactions};
use ratio::Ratio;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod sweep;
pub use sweep::Sweep;
mod ratio;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Monzo(#[from] monzo::Error),

    #[error("not found: {0}")]
    NotFound(String),
}

pub(crate) trait Operation: DeserializeOwned {
    fn name(&self) -> &'static str;
    fn account_id(&self) -> &str;
    fn transactions<'a>(&'a self, state: &'a State) -> Result<Transactions, Error>;
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Sweep(Sweep),
    Ratio(Ratio),
}

impl Operation for Op {
    fn name(&self) -> &'static str {
        match self {
            Self::Sweep(op) => op.name(),
            Self::Ratio(op) => op.name(),
        }
    }

    fn account_id(&self) -> &str {
        match self {
            Self::Sweep(op) => op.account_id(),
            Self::Ratio(op) => op.name(),
        }
    }

    fn transactions<'a>(&'a self, state: &'a State) -> Result<Transactions, Error> {
        match self {
            Self::Sweep(op) => op.transactions(state),
            Self::Ratio(op) => op.transactions(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Op;

    #[test]
    fn deserialise_yaml() {
        let raw = r#"
        - sweep:
            current_account_goal: 10000

            pots:
            - bills
            - lottery
            - allowance
            - student loan
            - savings

        - ratio:
            current_account_goal: 10000
            pots:
              savings: 2
              holiday: 1
    "#;

        serde_yaml::from_str::<Vec<Op>>(raw).unwrap();
    }
}
