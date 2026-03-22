-- Migration number: 0003 - store passkey credentials for authentication
CREATE TABLE passkey_credentials (
    credential_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    cose_public_key TEXT NOT NULL,
    sign_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);
