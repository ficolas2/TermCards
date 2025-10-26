use crate::data::deck::Deck;
use sqlx::{Sqlite, Transaction};

use super::repository::{Repository, RepositoryError};

impl Repository {
    pub async fn save_deck(&self, mut deck: Deck) -> Result<Deck, RepositoryError> {
        let mut tx: Transaction<'_, Sqlite> = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO decks
                (name, description)
            VALUES
                (?,    ?)
            "#,
        )
        .bind(&deck.name)
        .bind(&deck.description)
        .execute(&mut *tx)
        .await?;

        let mut ord = 0;
        for card in deck.cards.iter_mut() {
            card.id = sqlx::query_scalar(r#"
                INSERT INTO cards
                    (deck_name, ord, expected_output, expected_input, command, docker_image, work_dir, volume_mounts)
                VALUES
                    (?,         ?,   ?,               ?,              ?,       ?,            ?,        ?)
                RETURNING id
                "#)
                .bind(&deck.name)
                .bind(ord)
                .bind(&card.expected_output)
                .bind(&card.expected_input)
                .bind(&card.command)
                .bind(&card.docker_image)
                .bind(&card.work_dir)
                .bind(serde_json::to_string(&card.volume_mounts).unwrap())
                .fetch_one(&mut *tx)
                .await?;
            ord += 1;
        }

        tx.commit().await?;

        Ok(deck)
    }
}
