use clap::Parser;

mod show;
use show::Show;

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

#[derive(Debug, Parser, Clone, Copy)]
enum Subcommand {
    Show(Show),
    Run(Run),
}

impl Default for Subcommand {
    fn default() -> Self {
        Self::Show(Show::default())
    }
}

impl App {
    pub fn from_cli() -> Self {
        Self::parse()
    }

    pub async fn run(self) -> anyhow::Result<()> {
        logging::set_up(self.verbose);
        tracing::info!("logging configured");
        match self.subcommand.unwrap_or_default() {
            Subcommand::Show(_show) => Show::run()?,
            Subcommand::Run(run) => run.run().await?,
        }

        Ok(())
    }
}
