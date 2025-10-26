use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::card::Card;

#[derive(Debug, Error)]
pub enum CardImportError {
    #[error("failed to read file: {0}")]
    Io(#[from] io::Error),

    #[error("failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Deck {
    pub name: String,
    pub description: String,
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn import<P: AsRef<Path>>(path: P) -> Result<Deck, CardImportError> {
        let data = fs::read_to_string(path)?;
        let cards_file: Deck = toml::from_str(&data)?;
        Ok(cards_file)
    }
}
