use std::collections::HashMap;

use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct Strategy<'a> {
    main_pot_target: f32,
    
    #[serde(borrow)]
    pots: Vec<(&'a str, f32)>,
}

impl<'a> Strategy<'a> {
    pub fn calculate_changes(
        &'a self,
        current_main_balance: f32,
        current_pot_balances: &HashMap<&str, f32>,
    ) -> Vec<(&'a str, f32)> {
        let desired_main_balance_change = current_main_balance - self.main_pot_target;

        // separate the 'withdrawals' from pots with more cash than the target. these
        // can be returned without further processing (and should be performed first
        // anyway).
        let (withdrawals, remainder) = withdrawals(&self.pots, current_pot_balances);

        let total_withdrawals: f32 = withdrawals.iter().map(|(_pot_name, amount)| amount).sum();
        let mut spare_cash = -desired_main_balance_change - total_withdrawals;

        let mut deposits = Vec::default();

        for (pot, diff) in remainder {
            if spare_cash - diff >= 0_f32 {
                deposits.push((pot, diff));
                spare_cash -= diff
            } else {
                break;
            }
        }

        withdrawals.into_iter().chain(deposits).collect()
    }
}

type Diff<'a> = (&'a str, f32);
type Withdrawals<'a> = Vec<Diff<'a>>;
type Remainder<'a> = Vec<Diff<'a>>;

fn withdrawals<'a>(
    targets: &[(&'a str, f32)],
    balances: &HashMap<&str, f32>,
) -> (Withdrawals<'a>, Remainder<'a>) {
    targets
        .iter()
        .map(|(pot, target)| {
            let diff = target - balances.get(pot).unwrap();
            (*pot, diff)
        })
        .partition(|(_pot_name, amount)| amount < &0_f32)
}

#[cfg(test)]
mod tests {

    #[test]
    fn withdrawals() {
        let balances = [("savings", 120_f32), ("holiday", 80_f32)]
            .iter()
            .copied()
            .collect();

        let targets = vec![("savings", 100_f32), ("holiday", 100_f32)];

        let (withdrawals, remainder) = super::withdrawals(&targets, &balances);

        assert_eq!(withdrawals, vec![("savings", -20_f32)]);
        assert_eq!(remainder, vec![("holiday", 20_f32)]);
    }
}
