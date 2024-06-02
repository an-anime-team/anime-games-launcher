use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommands
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        self.command.execute().await
    }
}

#[derive(Subcommand)]
pub enum CliCommands {
    /// Manipulate packages storage
    Storage {
        #[arg(short, long)]
        /// Path to the storage
        path: Option<PathBuf>,

        #[command(subcommand)]
        subcommand: CliStorageCommands
    }
}

impl CliCommands {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Storage { path, subcommand }
                => subcommand.execute(path).await
        }
    }
}

#[derive(Subcommand)]
pub enum CliStorageCommands {
    /// Generate hash for given URI
    Hash {
        #[arg(short, long)]
        /// File URI
        uri: String,

        #[arg(short, long)]
        /// Hashing algorithm
        algorithm: Option<String>
    }
}

impl CliStorageCommands {
    pub async fn execute(self, storage_path: Option<PathBuf>) -> anyhow::Result<()> {
        let storage_path = match storage_path {
            Some(path) => path,
            None => {
                tracing::info!("No storage path given. Using config value");

                // FIXME
                PathBuf::from("storage")
            }
        };

        tracing::info!("Loading packages storage");

        let storage = crate::packages::storage::Storage::new(storage_path).await?;

        match self {
            Self::Hash { uri, algorithm } => {
                tracing::info!("Fetching URI content: {uri}");

                let content = crate::handlers::handle(uri)?.join().await??;

                let algorithm = algorithm
                    .and_then(crate::packages::hash::HashAlgorithm::from_str)
                    .unwrap_or_default();

                let hash = crate::packages::hash::Hash::from_slice(algorithm, content);

                tracing::info!("Calculated hash: {hash}");
            }
        }

        Ok(())
    }
}
