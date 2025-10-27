use args::{Args, Commands};
use clap::Parser;
use repository::repository::Repository;
use service::service::Service;

mod args;

mod domain {
    pub mod card;
    pub mod card_state;
    pub mod deck;
}

mod repository {
    pub mod deck_repository;
    pub mod repository;
    pub mod review_repository;
}

mod service {
    pub mod deck_service;
    pub mod review_service;
    pub mod scheduler_service;
    pub mod service;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let repository = Repository::new().await?;
    let service = Service::new(repository);

    match args.command {
        Commands::Import { path } => {
            service.import_deck(path).await?;
        }
        Commands::Review { deck_name } => service.review(deck_name).await?,
    };
    Ok(())
}
