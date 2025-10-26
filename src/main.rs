use crate::data::deck::Deck;
use anyhow::Result;
use service::review::run_sandboxed_card;

mod data {
    pub mod card;
    pub mod deck;
}

mod service {
    pub mod review_service;
}

fn main() -> Result<()> {
    let deck = Deck::import("./decks/jq.toml")?;
    for card in deck.cards {
        run_sandboxed_card(card)?;
    }
    Ok(())
}
