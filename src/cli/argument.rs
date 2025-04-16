use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    ///Serve a directory where a registry is located
    Serve {
        ///The directory which will be served 
        #[arg(long, default_value_t = String::from("."))]
        dir_to_serve: String,
    }
}
