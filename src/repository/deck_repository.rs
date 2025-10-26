use crate::data::{card::Card, deck::Deck};
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

    pub async fn get_deck(&self, name: &str) -> Result<Deck, RepositoryError> {
        // Fetch deck info
        let deck = sqlx::query!(
            r#"
            SELECT name, description
            FROM decks
            WHERE name = ?
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound("deck".to_string(), "name".to_string()))?;

        // Fetch cards
        let card_rows = sqlx::query!(
            r#"
            SELECT
                id,
                ord,
                expected_output,
                expected_input,
                command,
                docker_image,
                work_dir,
                volume_mounts
            FROM cards
            WHERE deck_name = ?
            ORDER BY ord
            "#,
            name
        )
        .fetch_all(&self.pool)
        .await?;

        let mut cards = Vec::new();
        for row in card_rows {
            let mounts: Vec<(String, String)> =
                serde_json::from_str(&row.volume_mounts).unwrap_or_default();

            cards.push(Card {
                id: row.id.unwrap_or(0),
                expected_output: row.expected_output,
                expected_input: row.expected_input,
                command: row.command,
                docker_image: row.docker_image,
                work_dir: row.work_dir,
                volume_mounts: mounts,
            });
        }

        Ok(Deck {
            name: deck.name,
            description: deck.description,
            cards,
        })
    }
}
