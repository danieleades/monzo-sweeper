use monzo::{Account, Pot};
use std::collections::HashMap;

type AccountId<'a> = &'a str;
pub struct Transactions<'a> {
    transactions: HashMap<&'a Account, Ledger<'a>>,
}

struct Ledger<'a> {
    withdrawals: Vec<(&'a Pot, u32)>,
    deposits: Vec<(&'a Pot, u32)>,
}
