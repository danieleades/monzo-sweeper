use clap::Parser;
use confy::ConfyError;

use crate::config;

#[derive(Debug, Default, Parser, Clone, Copy)]
pub struct Show;

impl Show {
    pub fn run() -> Result<(), ConfyError> {
        let operations = config::operations()?;
        println!("{:#?}", operations);
        Ok(())
    }
}
