use crate::{client::Auth, operation::Op};

static BIN_NAME: &str = std::env!("CARGO_PKG_NAME");

pub fn operations() -> Result<Vec<Op>, confy::ConfyError> {
    confy::load(BIN_NAME, "config")
}

pub fn auth() -> Result<Auth, confy::ConfyError> {
    confy::load(BIN_NAME, "auth")
}

pub fn save_auth(auth: &Auth) -> Result<(), confy::ConfyError> {
    confy::store(BIN_NAME, "auth", auth)
}
