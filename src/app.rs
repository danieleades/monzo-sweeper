use clap::Parser;

mod show;

mod run;
use run::Run;

use crate::logging;

#[derive(Debug, Parser, Clone, Copy)]
pub struct App {
    #[clap(short, long, parse(from_occurrences), global = true)]
    pub verbose: u8,

    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(Debug, Parser, Clone, Copy, Default)]
enum Subcommand {
    #[default]
    Show,
    Run(Run),
}

impl App {
    pub fn from_cli() -> Self {
        Self::parse()
    }

    pub async fn run(self) -> anyhow::Result<()> {
        logging::set_up(self.verbose);
        tracing::info!("logging configured");
        match self.subcommand.unwrap_or_default() {
            Subcommand::Show => show::run()?,
            Subcommand::Run(run) => run.run().await?,
        }

        Ok(())
    }
}
