CREATE TABLE saved (
    saved_id TEXT PRIMARY KEY,
    user_id TEXT,
    data_type TEXT NOT NULL,
    description TEXT NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    expires_at DATE GENERATED ALWAYS AS (
        date(created_at, '+30 day')
    ) VIRTUAL,

    FOREIGN KEY(user_id) REFERENCES users(user_id)
        ON DELETE SET NULL,

    CHECK (data_type in ('text', 'file'))
);

CREATE INDEX idx_saved_data_user_id ON saved (user_id);

CREATE TABLE saved_text (
    saved_id TEXT PRIMARY KEY,

    message TEXT NOT NULL,

    FOREIGN KEY(saved_id) REFERENCES saved(saved_id)
);

CREATE TABLE saved_file (
    saved_id TEXT PRIMARY KEY,

    file_hash TEXT NOT NULL,

    FOREIGN KEY(saved_id) REFERENCES saved(saved_id)
);
