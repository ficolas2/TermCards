use args::{Args, Commands};
use clap::Parser;
use crossterm::style::Stylize;
use domain::{card::Card, card_state::{CardStatus, CardState}};
use repository::repository::Repository;
use service::service::Service;
use utils::time_utils::{format_until_duration, now_s};

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
        Commands::TestDeck { path } => {
            let deck = service.read_deck_from_file(path)?;
            service.review_full_deck(deck);
        }
    };
    Ok(())
}

fn print_deck_state(deck_name: &str, card_state_list: Vec<(Card, CardState)>) {
    if card_state_list.is_empty() {
        println!("No cards found");
        return;
    }
    let new = card_state_list
        .iter()
        .filter(|cs| cs.1.status == CardStatus::New)
        .count();
    let learn = card_state_list
        .iter()
        .filter(|cs| cs.1.status == CardStatus::Learn)
        .count();
    let to_review = card_state_list
        .iter()
        .filter(|cs| cs.1.status == CardStatus::Review && cs.1.next_review_s < now_s())
        .count();
    let total_cards = card_state_list.iter().count();

    println!(
        "{}   {} {} {}    {}",
        format!("{deck_name}").bold(),
        format!("{new:>4}").bold().blue(),
        format!("{learn:>4}").bold().red(),
        format!("{to_review:>4}").bold().green(),
        format!("Total cards: {total_cards}").dark_grey(),
    );

    for (i, (_, card_state)) in card_state_list.iter().enumerate() {
        let status_str = match card_state.status {
            CardStatus::New => "New".to_string().blue().bold(),
            CardStatus::Learn => "Learn".to_string().red().bold(),
            CardStatus::Review => {
                if card_state.next_review_s < now_s() {
                    "Review".to_string().green().bold()
                } else {
                    format_until_duration(card_state.next_review_s - now_s()).dark_grey()
                }
            },
            CardStatus::OneTimeLearned => continue,
        };
        println!(
            "    {} {}",
            i, status_str
        )
    }
}
