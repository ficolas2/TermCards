use serde::{Deserialize, Serialize};

use super::card::Card;

#[derive(Deserialize, Serialize, Debug)]
pub struct Deck {
    pub name: String,
    pub description: String,
    pub cards: Vec<Card>,
}
