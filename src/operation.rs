use monz0_lib::{Ledger, Operation, State, operation::Sweep};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Op {
    Sweep(Sweep),
    // Ratio(Ratio),
}

impl Op {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Sweep(_) => Sweep::NAME,
            // Self::Ratio(op) => op.name(),
        }
    }

    pub fn transactions<'a>(&'a self, state: &'a State) -> anyhow::Result<Ledger<'a>> {
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
    fn deserialise_toml() {
        let raw = r#"
type = "sweep"
account_goal = 10000
pots = ["bills", "lottery", "allowance", "student loan", "savings"]
"#;

        toml::from_str::<Op>(raw).unwrap();
    }
}
