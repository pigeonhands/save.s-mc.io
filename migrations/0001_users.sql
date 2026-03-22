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
    encryption_key_id TEXT NOT NULL DEFAULT '',

    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_keys_encryption_key_id ON keys (encryption_key_id);

CREATE VIEW user_public_key AS
    SELECT keys.public_key
    FROM users
    JOIN keys USING (user_id);

CREATE TABLE passkey_credentials (
    credential_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    cose_public_key TEXT NOT NULL,
    sign_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);
