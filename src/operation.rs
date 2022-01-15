use monz0_lib::{operation::Sweep, Ledger, Operation, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Sweep(Sweep),
    // Ratio(Ratio),
}

impl Op {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sweep(_) => Sweep::NAME,
            // Self::Ratio(op) => op.name(),
        }
    }

    pub fn transactions<'a>(&'a self, state: &'a State) -> anyhow::Result<Ledger> {
        match self {
            Self::Sweep(op) => Ok(op.transactions(state)?),
            // Self::Ratio(op) => op.transactions(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Op;

    #[test]
    fn deserialise_yaml() {
        //     let raw = r#"
        //     - sweep: account_goal: 10000

        //         pots:
        //         - bills
        //         - lottery
        //         - allowance
        //         - student loan
        //         - savings

        //     - ratio: current_account_goal: 10000 pots: savings: 2 holiday: 1
        // "#;

        let raw = r#"
    - sweep:
        account_goal: 10000

        pots:
        - bills
        - lottery
        - allowance
        - student loan
        - savings
"#;

        serde_yaml::from_str::<Vec<Op>>(raw).unwrap();
    }
}
