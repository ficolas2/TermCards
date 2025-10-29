use crate::domain::{card::Card, card_state::CardState};

use super::repository::{Repository, RepositoryError};

impl Repository {
    pub async fn get_next_card_to_review(
        &self,
        prefix: &str,
    ) -> Result<Option<Card>, RepositoryError> {
        let res = sqlx::query!(
            r#"
                SELECT 
                    id,
                    deck_name,
                    expected_output,
                    expected_input,
                    command,
                    docker_image,
                    work_dir,
                    volume_mounts,
                    one_time,
                    created_at,
                    updated_at
                FROM cards
                JOIN card_state ON card_id = id
                WHERE 
                    deck_name LIKE ? || '%'
                    AND next_review_s < strftime('%s', 'now')
                    AND status >= 0
                ORDER BY next_review_s, ord
            "#,
            prefix
        )
        .fetch_optional(&self.pool)
        .await?;
        // TODO: fetch learning first
        // TODO: fetch learning even if time hasn't arrived yet

        Ok(res.map(|res| Card {
            id: res.id,
            volume_mounts: serde_json::from_str(&res.volume_mounts)
                .expect("Invalid JSON in volume_mounts for card"),
            expected_output: res.expected_output,
            expected_input: res.expected_input,
            command: res.command,
            docker_image: res.docker_image,
            work_dir: res.work_dir,
            one_time: res.one_time,
        }))
    }

    pub async fn get_card_state(&self, id: i64) -> Result<CardState, RepositoryError> {
        sqlx::query_as!(
            CardState,
            r#"
                SELECT 
                    card_id,
                    next_review_s,
                    interval_days,
                    ease,
                    reps,
                    lapses,
                    status,
                    learning_step
                FROM card_state
                WHERE 
                    card_id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn get_deck_card_states(
        &self,
        deck_name: &str,
    ) -> Result<Vec<CardState>, RepositoryError> {
        sqlx::query_as!(
            CardState,
            r#"
                SELECT
                  cs.card_id         as "card_id!: i64",
                  cs.next_review_s   as "next_review_s!: i64",
                  cs.interval_days   as "interval_days!: i64",
                  cs.ease            as "ease!: i64",
                  cs.reps            as "reps!: i64",
                  cs.lapses          as "lapses!: i64",
                  cs.status          as "status!: i64",
                  cs.learning_step
                FROM card_state cs
                INNER JOIN cards ON cards.id = cs.card_id
                WHERE cards.deck_name = ?
            "#,
            deck_name
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn set_card_state(&self, card_state: CardState) -> Result<(), RepositoryError> {
        let res = sqlx::query!(
            r#"
                UPDATE card_state
                SET 
                    next_review_s = ?,
                    interval_days = ?,
                    ease = ?,
                    reps = ?,
                    lapses = ?,
                    status = ?,
                    learning_step = ?
                WHERE card_id = ?
            "#,
            card_state.next_review_s,
            card_state.interval_days,
            card_state.ease,
            card_state.reps,
            card_state.lapses,
            card_state.status,
            card_state.learning_step,
            card_state.card_id
        )
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            Err(RepositoryError::NotFound(
                "card_state".to_string(),
                card_state.card_id.to_string(),
            ))?
        }
        Ok(())
    }
}
