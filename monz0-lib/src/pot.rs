/// A Monzo pot
#[derive(Debug, Default)]
pub struct Pot {
    /// The name of the pot
    pub name: String,

    /// The unique ID associated with the pot
    pub id: String,

    /// The currency code for this pot
    pub currency: String,
}
