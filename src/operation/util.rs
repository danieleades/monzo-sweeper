use crate::operation::Error;
use monzo::{Account, AccountType};

pub fn find_current_account<'a>(
    accounts: &'a [Account],
    current_account_id: Option<&str>,
) -> Result<&'a Account, Error> {
    let account = if let Some(id) = current_account_id {
        accounts
            .iter()
            .find(|account| account.id == id)
            .ok_or_else(|| Error::NotFound("unable to determine current account".to_string()))?
    } else {
        accounts
            .iter()
            .find(|account| matches!(account.account_type, AccountType::UkRetail))
            .ok_or_else(|| Error::NotFound("unable to determine current account".to_string()))?
    };

    Ok(account)
}
