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
