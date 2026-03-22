-- Migration number: 0001 	 2025-06-21T21:15:11.576Z
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    is_admin BOOL DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_email ON users (email);

CREATE TABLE keys (
    user_id TEXT UNIQUE NOT NULL,

    public_key TEXT NOT NULL,

    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE VIEW user_public_key AS
    SELECT keys.public_key
    FROM users
    JOIN keys USING (user_id);
