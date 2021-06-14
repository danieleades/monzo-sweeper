use std::collections::HashMap;

use crate::{
    operation::{Error, Operation},
    state::State,
    transactions::Ledger,
};
use monzo::Pot;
use serde::Deserialize;

use super::util::find_current_account;

#[derive(Debug, Deserialize)]
pub struct Ratio {
    #[serde(default)]
    current_account_id: Option<String>,
    #[serde(default)]
    current_account_goal: Option<f32>,
    pots: HashMap<String, u32>,
}

impl Operation for Ratio {
    fn name(&self) -> &'static str {
        "Ratio"
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn transactions<'a>(&'a self, state: &'a State) -> Result<Ledger<'a>, Error> {
        let account = find_current_account(&state.accounts, self.current_account_id.as_deref())?;

        let spare_cash = state.balance.get(&account.id).unwrap().balance();

        let denominator: u32 = self.pots.values().sum();

        let deposits = self.pots.iter().map(|(name, numerator)| {
            let pot = find_pot_by_name(state.pots.get(&account.id).unwrap(), name).unwrap();
            let deposit: i64 = ((f64::from(*numerator)) / (f64::from(denominator))
                * spare_cash as f64)
                .floor() as i64;
            (pot, deposit)
        });

        let mut ledger = Ledger::default();

        for t in deposits {
            ledger.push(account, t);
        }

        Ok(ledger)
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

        let _: Ratio = serde_yaml::from_str(raw).unwrap();
    }
}
