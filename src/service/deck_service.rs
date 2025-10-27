use std::{fs, io, path::Path};
use thiserror::Error;

use crate::{domain::{card_state::CardState, deck::Deck}, repository::repository::RepositoryError};

use super::service::Service;

#[derive(Debug, Error)]
pub enum CardImportError {
    #[error("failed to read file: {0}")]
    Io(#[from] io::Error),

    #[error("failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

impl Service {
    pub async fn import_deck<P: AsRef<Path>>(&self, path: P) -> Result<Deck, CardImportError> {
        let data = fs::read_to_string(path)?;
        let deck: Deck = toml::from_str(&data)?;

        let deck = self.repository.save_deck(deck).await?;

        Ok(deck)
    }

    pub async fn get_deck_state(&self, deck_name: &str) -> Result<Vec<CardState>, RepositoryError> {
        self.repository.get_deck_card_states(deck_name).await
    }
}
