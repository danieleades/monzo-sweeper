use serde::Deserialize;

mod sweep;
pub use sweep::Config as Sweep;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Sweep(Sweep),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("'Sweep' operation failed")]
    Sweep(#[from] sweep::Error),
}

#[cfg(test)]
mod tests {
    use super::Operation;

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

        let _: Operation = serde_yaml::from_str(raw).unwrap();
    }

    #[test]
    fn deserialise_yaml_vec() {
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

        let _: Operation = serde_yaml::from_str(raw).unwrap();
    }
}
