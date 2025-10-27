use args::{Args, Commands};
use clap::Parser;
use crossterm::style::Stylize;
use domain::card_state::{CardLearnStatus, CardState};
use repository::repository::Repository;
use service::service::Service;
use utils::time_utils::now_s;

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

mod utils {
    pub mod time_utils;
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
        Commands::State { deck_name } => {
            print_deck_state(&deck_name, service.get_deck_state(&deck_name).await?)
        }
    };
    Ok(())
}

fn print_deck_state(deck_name: &str, card_state_list: Vec<CardState>) {
    if card_state_list.is_empty() {
        println!("No cards found");
        return;
    }
    let new = card_state_list
        .iter()
        .filter(|cs| cs.status == CardLearnStatus::New)
        .count();
    let learn = card_state_list
        .iter()
        .filter(|cs| cs.status == CardLearnStatus::Learn)
        .count();
    let to_review = card_state_list
        .iter()
        .filter(|cs| cs.status == CardLearnStatus::Review && cs.next_review_s < now_s())
        .count();
    let total_cards = card_state_list.iter().count();

    println!(
        "  {} {} {} {}    {}",
        format!("{deck_name:<20}").bold(),
        format!("{new:>4}").bold().blue(),
        format!("{learn:>4}").bold().red(),
        format!("{to_review:>4}").bold().green(),
        format!("Total cards: {total_cards}").dark_grey(),
    );
}
