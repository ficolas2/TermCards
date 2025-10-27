use std::{
    time::{SystemTime, UNIX_EPOCH},
    usize,
};

use sqlx::{Type, prelude::FromRow};

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

#[derive(Debug, Clone, PartialEq, Eq, Copy, Type)]
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

#[derive(PartialEq, Eq)]
pub enum ReviewResult {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

const MIN: i64 = 60;
const DAY: i64 = 24 * 60 * 60;
const LEARNING_INTERVALS: [i64; 3] = [1 * MIN, 10 * MIN, 1 * DAY];
const LAST_LEARNING_STEP: i64 = LEARNING_INTERVALS.len() as i64 - 1;

const GRADUATING_DAYS: i64 = 1;

const MIN_EASE: i64 = 1300;
const MAX_EASE: i64 = 3500;

const EASY_MULT: f64 = 1.3;
impl CardState {
    pub fn apply_review(&mut self, review_result: ReviewResult) {
        self.learning_step = self.learning_step.clamp(0, LAST_LEARNING_STEP);

        self.reps += 1;
        if ReviewResult::Again == review_result {
            self.lapses += 1
        }

        let now_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        match self.status {
            // Learning
            CardLearnStatus::New | CardLearnStatus::Learn => {
                self.status = CardLearnStatus::Learn;
                match review_result {
                    ReviewResult::Again => {
                        self.learning_step = 0;
                        self.next_review_s = now_s + LEARNING_INTERVALS[0];
                    }
                    ReviewResult::Hard => {
                        self.next_review_s =
                            now_s + LEARNING_INTERVALS[self.learning_step as usize];
                    }
                    ReviewResult::Good => {
                        self.learning_step += 1;
                        if self.learning_step > LAST_LEARNING_STEP {
                            self.status = CardLearnStatus::Review;
                            self.learning_step = 0;
                            self.interval_days = GRADUATING_DAYS;
                            self.next_review_s = now_s + GRADUATING_DAYS * DAY;
                        } else {
                            self.next_review_s =
                                now_s + LEARNING_INTERVALS[self.learning_step as usize];
                        }
                    }
                    ReviewResult::Easy => {
                        self.next_review_s = now_s + 4 * 24 * 60 * 60;
                        self.interval_days = 1;
                        self.status = CardLearnStatus::Review;
                    }
                }
            }
            // Reviewing
            CardLearnStatus::Review => match review_result {
                ReviewResult::Again => {
                    self.status = CardLearnStatus::Learn;
                    self.learning_step = 0;
                    self.ease = (self.ease - 200).max(MIN_EASE);
                    self.interval_days = 1;
                    self.next_review_s = now_s + LEARNING_INTERVALS[0]
                }
                ReviewResult::Hard => {
                    self.ease = (self.ease - 150).max(MIN_EASE);
                    self.interval_days = ((self.interval_days as f64 * 1.2) as i64).max(1);
                    self.next_review_s = now_s + self.interval_days * DAY;
                }
                ReviewResult::Good => {
                    let mult = self.ease as f64 / 1000.0;
                    self.interval_days = ((self.interval_days as f64 * mult) as i64).max(1);
                    self.next_review_s = now_s + self.interval_days * DAY;
                }
                ReviewResult::Easy => {
                    self.ease = (self.ease + 150).min(MAX_EASE);
                    let mult = self.ease as f64 / 1000.0 * EASY_MULT;
                    self.interval_days = (self.interval_days as f64 * mult) as i64;
                    self.next_review_s = now_s + self.interval_days * DAY;
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{CardLearnStatus, CardState, DAY, MIN, ReviewResult};

    fn state_new() -> CardState {
        CardState {
            card_id: 0,
            next_review_s: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            interval_days: 1,
            ease: 2500,
            reps: 0,
            lapses: 0,
            status: CardLearnStatus::New,
            learning_step: 0,
        }
    }

    fn state_review() -> CardState {
        CardState {
            card_id: 0,
            next_review_s: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            interval_days: 1,
            ease: 2500,
            reps: 0,
            lapses: 0,
            status: CardLearnStatus::Review,
            learning_step: 0,
        }
    }

    fn run_test_state(
        mut card_state: CardState,
        apply: Vec<ReviewResult>,
        status: CardLearnStatus,
        learning_step: i64,
        interval_days: i64,
        interval: i64,
        ease: i64,
    ) {
        let now_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        for review in apply {
            card_state.apply_review(review);
        }

        assert_eq!(card_state.status, status);
        assert_eq!(card_state.learning_step, learning_step);
        assert_eq!(card_state.interval_days, interval_days);
        assert!((card_state.next_review_s - now_s - interval).abs() < 2);
        assert_eq!(card_state.ease, ease);
    }

    macro_rules! test_state {
        (
            $func:ident,
            $init:expr,
            [ $($res:ident),* $(,)? ],
            status: $status:ident,
            learning_step: $ls:expr,
            interval_days: $ivl:expr,
            added_time_s: $add:expr,
            ease: $ease:expr $(,)?
        ) => {
            #[test]
            fn $func() {
                run_test_state(
                    $init,
                    vec![$( ReviewResult::$res ),*],
                    CardLearnStatus::$status,
                    $ls,
                    $ivl,
                    $add,
                    $ease
                )
            }
        };
    }

    // New
    // new -> again
    test_state!(
        test_state_new_again,
        state_new(),
        [Again],
        status: Learn,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * MIN,
        ease: 2500,
    );

    // new -> hard
    test_state!(
        test_state_new_hard,
        state_new(),
        [Hard],
        status: Learn,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * MIN,
        ease: 2500,
    );

    // new -> good
    test_state!(
        test_state_new_good,
        state_new(),
        [Good],
        status: Learn,
        learning_step: 1,
        interval_days: 1,
        added_time_s: 10 * MIN,
        ease: 2500,
    );

    // new -> easy
    test_state!(
        test_state_new_easy,
        state_new(),
        [Easy],
        status: Review,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 4 * DAY,
        ease: 2500,
    );

    // New two step
    // new -> again + again
    test_state!(
        test_state_new_again_again,
        state_new(),
        [Again, Again],
        status: Learn,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * MIN,
        ease: 2500,
    );

    // new -> good + good
    test_state!(
        test_state_new_good_good,
        state_new(),
        [Good, Good],
        status: Learn,
        learning_step: 2,
        interval_days: 1,
        added_time_s: 1 * DAY,
        ease: 2500,
    );

    // new -> easy + easy
    test_state!(
        test_state_new_easy_easy,
        state_new(),
        [Easy, Easy],
        status: Review,
        learning_step: 0,
        interval_days: 3,
        added_time_s: 3 * DAY,
        ease: 2650,
    );

    // Review
    // review -> again
    test_state!(
        test_state_review_again,
        state_review(),
        [Again],
        status: Learn,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * MIN,
        ease: 2300,
    );

    // review -> hard
    test_state!(
        test_state_review_hard,
        state_review(),
        [Hard],
        status: Review,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * DAY,
        ease: 2350,
    );

    // review -> good
    test_state!(
        test_state_review_good,
        state_review(),
        [Good],
        status: Review,
        learning_step: 0,
        interval_days: 2,
        added_time_s: 2 * DAY,
        ease: 2500,
    );

    // Review -> easy
    test_state!(
        test_state_review_easy,
        state_review(),
        [Easy],
        status: Review,
        learning_step: 0,
        interval_days: 3,
        added_time_s: 3 * DAY,
        ease: 2650,
    );

    // Review two steps
    // review -> good + good
    test_state!(
        test_state_review_good_good,
        state_review(),
        [Good, Good],
        status: Review,
        learning_step: 0,
        interval_days: 5,
        added_time_s: 5 * DAY,
        ease: 2500,
    );

    // review -> good + easy
    test_state!(
        test_state_review_good_easy,
        state_review(),
        [Good, Easy],
        status: Review,
        learning_step: 0,
        interval_days: 6,
        added_time_s: 6 * DAY,
        ease: 2650,
    );

    // review -> Hard + Good
    test_state!(
        test_state_review_hard_good,
        state_review(),
        [Hard, Good],
        status: Review,
        learning_step: 0,
        interval_days: 2,
        added_time_s: 2 * DAY,
        ease: 2350,
    );

    // review -> Easy + Good
    test_state!(
        test_state_review_easy_good,
        state_review(),
        [Easy, Good],
        status: Review,
        learning_step: 0,
        interval_days: 7,
        added_time_s: 7 * DAY,
        ease: 2650,
    );

    // review -> Again + Good
    test_state!(
        test_state_review_again_good,
        state_review(),
        [Again, Good],
        status: Learn,
        learning_step: 1,
        interval_days: 1,
        added_time_s: 10 * MIN,
        ease: 2300,
    );

    // review -> Again + Easy
    test_state!(
        test_state_review_again_easy,
        state_review(),
        [Again, Easy],
        status: Review,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 4 * DAY,
        ease: 2300,
    );

    // review -> Again + Again
    test_state!(
        test_state_review_again_again,
        state_review(),
        [Again, Again],
        status: Learn,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * MIN,
        ease: 2300,
    );

    // review -> Hard + Hard
    test_state!(
        test_state_review_hard_hard,
        state_review(),
        [Hard, Hard],
        status: Review,
        learning_step: 0,
        interval_days: 1,
        added_time_s: 1 * DAY,
        ease: 2200,
    );
}
