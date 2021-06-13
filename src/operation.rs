use monzo::Pot;
use serde::{de::DeserializeOwned, Deserialize};

mod sweep;
pub use sweep::Sweep;

use crate::state::State;

pub(crate) trait Operation: DeserializeOwned {
    type Err;
    fn name(&self) -> &'static str;
    fn transactions<'a>(&'a self, state: &'a State) -> Result<Vec<(&'a Pot, i64)>, Self::Err>;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Sweep(Sweep),
}

impl Operation for Op {
    type Err = Error;

    fn name(&self) -> &'static str {
        match self {
            Self::Sweep(op) => op.name(),
        }
    }

    fn transactions<'a>(&'a self, state: &'a State) -> Result<Vec<(&'a Pot, i64)>, Self::Err> {
        let op = match self {
            Self::Sweep(op) => op,
        };

        Ok(op.transactions(state)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("'Sweep' operation failed")]
    Sweep(#[from] sweep::Error),
}

#[cfg(test)]
mod tests {
    use super::Op;

    #[test]
    fn deserialise_yaml() {
        let raw = r#"
        sweep:
            current_account_goal: 10000

            pots:
            - bills
            - lottery
            - allowance
            - student loan
            - savings
    "#;

        let _: Op = serde_yaml::from_str(raw).unwrap();
    }
}
