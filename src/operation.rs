use crate::{state::State, transactions::Ledger};
use ratio::Ratio;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod sweep;
pub use sweep::Sweep;
mod ratio;
mod util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Monzo(#[from] monzo::Error),

    #[error("not found: {0}")]
    NotFound(String),
}

pub(crate) trait Operation: DeserializeOwned {
    fn name(&self) -> &'static str;
    fn transactions<'a>(&'a self, state: &'a State) -> Result<Ledger<'a>, Error>;
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

    fn transactions<'a>(&'a self, state: &'a State) -> Result<Ledger<'a>, Error> {
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

        let _: Vec<Op> = serde_yaml::from_str(raw).unwrap();
    }
}
