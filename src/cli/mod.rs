use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommands
}

impl Cli {
    #[inline]
    pub async fn execute(self) -> anyhow::Result<()> {
        self.command.execute().await
    }
}

#[derive(Subcommand)]
pub enum CliCommands {
    /// Packages system commands.
    Store {
        #[arg(short, long)]
        /// Path to the resources store.
        path: Option<PathBuf>,

        #[command(subcommand)]
        subcommand: CliStorageCommands
    }
}

impl CliCommands {
    #[inline]
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Store { path, subcommand }
                => subcommand.execute(path).await
        }
    }
}

#[derive(Subcommand)]
pub enum CliStorageCommands {

}

impl CliStorageCommands {
    pub async fn execute(self, store: Option<PathBuf>) -> anyhow::Result<()> {
        let store_path = match store {
            Some(path) => path,
            None => {
                tracing::info!("No store path given. Using config value");

                // FIXME
                PathBuf::from("store")
            }
        };

        tracing::info!("Loading packages storage");

        Ok(())
    }
}
