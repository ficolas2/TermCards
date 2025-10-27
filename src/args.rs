use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Import a deck")]
    Import { path: String },
    #[command(about = "Review a deck")]
    Review { deck_name: String },
    #[command(about = "Get the state of a deck")]
    State { deck_name: String },
}
