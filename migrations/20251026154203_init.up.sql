PRAGMA journal_mode=WAL;

PRAGMA foreign_keys = ON;

CREATE TABLE decks (
    name         TEXT NOT NULL PRIMARY KEY,
    description  TEXT NOT NULL,
    created_at   INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at   INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE cards (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_name       TEXT NOT NULL REFERENCES decks(name) ON DELETE CASCADE,
    ord             INTEGER NOT NULL,
    expected_output TEXT NOT NULL,
    expected_input  TEXT NOT NULL,
    command         TEXT,
    docker_image    TEXT NOT NULL,
    work_dir        TEXT,
    volume_mounts   TEXT NOT NULL, -- stored as JSON string

    created_at      INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at      INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX idx_cards_deck_name ON cards(deck_name);

-- current scheduling state
CREATE TABLE card_state (
  card_id         INTEGER PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
  next_review_s   INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  interval_days   INTEGER NOT NULL DEFAULT 0,
  ease            INTEGER NOT NULL DEFAULT 2500,
  reps            INTEGER NOT NULL DEFAULT 0,
  lapses          INTEGER NOT NULL DEFAULT 0,
                  
  status          INTEGER NOT NULL DEFAULT 0, -- 0 new, 1 learn, 2 review, -1 suspended
  learning_step   INTEGER NOT NULL DEFAULT 0 -- learning step, [1m, 10m, 1d] - +1 on Good, +2 on easy, queue set to review (2) if ok on 1d
);

CREATE INDEX idx_state_due ON card_state(next_review_s);

-- append-only review log
CREATE TABLE review_log (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  card_id      INTEGER NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  timestamp    INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  rating       INTEGER NOT NULL, -- 1..4, 1 meaning failure, 4 meaning easy
  prev_ivl     INTEGER NOT NULL,
  new_ivl      INTEGER NOT NULL,
  prev_ease    INTEGER NOT NULL,
  new_ease     INTEGER NOT NULL
);

CREATE INDEX idx_log_card_ts ON review_log(card_id, timestamp);
