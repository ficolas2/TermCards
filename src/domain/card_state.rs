use sqlx::{prelude::FromRow, Type};

#[derive(Debug, FromRow)]
pub struct CardState {
    pub card_id: i64,
    pub next_review_s: i64,

    pub interval_days: i64,
    pub ease: i64,

    pub reps: i64,
    pub lapses: i64,

    pub status: CardLearnStatus,
    pub learning_step: i64,
}

#[derive(Debug, Clone, Copy, Type)]
#[repr(i64)]
#[sqlx(type_name = "INTEGER")]
pub enum CardLearnStatus {
    New = 0,
    Learn = 1,
    Review = 2,
}

impl From<i64> for CardLearnStatus {
    fn from(value: i64) -> Self {
        match value {
            1 => CardLearnStatus::Learn,
            2 => CardLearnStatus::Review,
            _ => CardLearnStatus::New,
        }
    }
}


