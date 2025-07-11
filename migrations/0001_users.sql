-- Migration number: 0001 	 2025-06-21T21:15:11.576Z
CREATE TABLE users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT,
    is_admin BOOL DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_email ON users (email);

CREATE TABLE keys (
    user_id INTEGER UNIQUE,

    public_key TEXT NOT NULL,

    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE VIEW user_public_key AS
    SELECT keys.public_key
    FROM users
    JOIN keys USING (user_id);

--
-- INSERT INTO users (email, is_admin) VALUES (
--     ("sam@s-mc.io", true)
-- ); 
--
-- INSERT INTO keys (user_id, public_key) VALUES (
--     (last_insert_rowid(), "-----BEGIN PGP PUBLIC KEY BLOCK-----
--
-- mDMEZ8zPYhYJKwYBBAHaRw8BAQdAPZIcUd/BGPEuiUp3e2U7dIWFl+n4DsOFjXj2
-- dGhGA4q0E1NhbSBNIDxzYW1Acy1tYy5pbz6IlAQTFgoAPBYhBDTql6996KUygFkd
-- o1Qh3Pq6DpKiBQJnzM9iAhsDBQkFo5qABAsJCAcEFQoJCAUWAgMBAAIeBQIXgAAK
-- CRBUIdz6ug6SoqPnAQCyeTu+5J7n/nDDI1G1glQisdY07A1hotef0LwbYUvKqwD/
-- XrNvzYLPRf359FHbTwxElnf8PEknEUGf3M86+oxvYgm4OARnzM9iEgorBgEEAZdV
-- AQUBAQdAlTvNg+yH+A4JJg9UF/IPSCc+YAUTetg2BV680xTUvQoDAQgHiH4EGBYK
-- ACYWIQQ06pevfeilMoBZHaNUIdz6ug6SogUCZ8zPYgIbDAUJBaOagAAKCRBUIdz6
-- ug6Soh3yAQDFbDN9l9qO5cP/XQyyJb+X8qHw0sr9H83fmeQuwGk3UgD9Gi2p61IS
-- MnoqIPI3PoRPupuvEpS1U9pFt3rgkQelogS4MwRoUU9SFgkrBgEEAdpHDwEBB0BH
-- 0Arixqe7C9xq0kjJL/NlsVkgM8pqeWsWPzUZIPvZ+Ij1BBgWCgAmFiEENOqXr33o
-- pTKAWR2jVCHc+roOkqIFAmhRT1ICGwIFCQWjmoAAgQkQVCHc+roOkqJ2IAQZFgoA
-- HRYhBMXcl3UaCYB98ur3c8+BxxUzcS3LBQJoUU9SAAoJEM+BxxUzcS3LKp4BAIqt
-- WADQS6K25PsczlhRHS2RrxN0OqQDDDlnlVBW2tM1AP9PapZA8EcPm7113cNiGbWx
-- fr/8cV2vm52TZ0ATI+0KA6GRAP9/6TvjM0FnyaZppEZV7epXwTW2PkquUpDu7LKw
-- W+rfJQD/biS/tV9TRxiG91IWwi5ETzexjPCdG+AINYFl80MHugW4MwRoUdgpFgkr
-- BgEEAdpHDwEBB0CGbOhHgcyJif7P5wXiebzlu8sYkmUl/K+xbw8bNh07xIh+BBgW
-- CgAmFiEENOqXr33opTKAWR2jVCHc+roOkqIFAmhR2CkCGyAFCQWjmoAACgkQVCHc
-- +roOkqLOgQD9FPXGjjrhZjoKWQmWmJOGKw24v0bh2cKq94Fqt3hhwwgBAKuThIFg
-- qc4Y/iqG99yF2DwV/RmyaRRqqjeALeBgF3IB
-- =rRWJ
-- -----END PGP PUBLIC KEY BLOCK-----")
-- );
--


