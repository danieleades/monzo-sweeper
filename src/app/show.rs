use confy::ConfyError;

use crate::config;

pub fn run() -> Result<(), ConfyError> {
    let operations = config::operations()?;
    println!("{:#?}", operations);
    Ok(())
}
