use args::{Args, Commands};
use clap::Parser;
use repository::repository::Repository;
use service::service::Service;

mod args;

mod data {
    pub mod card;
    pub mod deck;
}

mod repository {
    pub mod deck_repository;
    pub mod repository;
}

mod service {
    pub mod deck_service;
    pub mod review_service;
    pub mod service;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let repository = Repository::new().await?;
    let service = Service::new(repository);

    match args.command {
        Commands::Import { path } => service.import_deck(path).await?,
    };
    Ok(())
}
