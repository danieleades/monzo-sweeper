use crate::{
    client::State,
    operation::{Operation},
    transactions::Transactions,
};
use monzo::Pot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::NotFoundError;

#[derive(Debug, Deserialize, Serialize)]
pub struct Ratio {
    #[serde(default)]
    current_account_id: String,
    #[serde(default)]
    current_account_goal: Option<f32>,
    pots: HashMap<String, u32>,
}

impl Operation for Ratio {
    type Err = NotFoundError;
    fn name(&self) -> &'static str {
        "Ratio"
    }

    fn account_id(&self) -> &str {
        &self.current_account_id
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn transactions<'a>(&'a self, state: &'a State) -> Result<Transactions, Self::Err> {
        let spare_cash = state.balance.balance;

        let denominator: u32 = self.pots.values().sum();

        let deposits = self.pots.iter().map(|(name, numerator)| {
            let pot = find_pot_by_name(&state.pots, name).unwrap();
            let deposit: i64 = ((f64::from(*numerator)) / (f64::from(denominator))
                * spare_cash as f64)
                .floor() as i64;
            (pot, deposit)
        });

        Ok(deposits.collect())
    }
}

fn find_pot_by_name<'a>(pots: &'a [Pot], name: &'a str) -> Option<&'a Pot> {
    pots.iter().find(|pot| pot.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialise_yaml() {
        let raw = r#"
        current_account_goal: 10000

        pots:
          savings: 2
          holiday: 1
        "#;

        serde_yaml::from_str::<Ratio>(raw).unwrap();
    }
}
