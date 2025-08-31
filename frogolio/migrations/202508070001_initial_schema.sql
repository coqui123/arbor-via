-- 2025-08-07-0001_users.sql
CREATE TABLE users (
  id        TEXT PRIMARY KEY,
  email       TEXT NOT NULL UNIQUE,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE frogols (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE links (
    id TEXT PRIMARY KEY,
    frogol_id TEXT NOT NULL,
    url TEXT NOT NULL,
    label TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    FOREIGN KEY(frogol_id) REFERENCES frogols(id)
);

CREATE TABLE leads (
    id TEXT PRIMARY KEY,
    frogol_id TEXT NOT NULL,
    email TEXT NOT NULL,
    source TEXT,
    score INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    FOREIGN KEY(frogol_id) REFERENCES frogols(id)
);

CREATE TABLE clicks (
    id TEXT PRIMARY KEY,
    link_id TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    FOREIGN KEY(link_id) REFERENCES links(id)
);
