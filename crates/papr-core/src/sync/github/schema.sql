-- v18: private GitHub synchronization metadata; no credentials or article bodies.
ALTER TABLE feeds ADD COLUMN subscription_active INTEGER NOT NULL DEFAULT 1;
ALTER TABLE feeds ADD COLUMN github_generation INTEGER NOT NULL DEFAULT 1;
ALTER TABLE articles ADD COLUMN metadata_only INTEGER NOT NULL DEFAULT 0;
CREATE TABLE github_connections (
    id INTEGER PRIMARY KEY,
    profile TEXT NOT NULL,
    dataset_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    applying INTEGER NOT NULL DEFAULT 0,
    next_seq INTEGER NOT NULL DEFAULT 1,
    ack_seq INTEGER NOT NULL DEFAULT 0,
    head TEXT,
    last_success_at TEXT,
    last_error_code TEXT,
    retry_at TEXT,
    binding TEXT NOT NULL,
    lease TEXT,
    lease_until TEXT,
    pending_since TEXT,
    last_edit_at TEXT,
    last_publish_at TEXT
);
CREATE UNIQUE INDEX github_one_active_connection ON github_connections(active) WHERE active=1;
CREATE TABLE github_outbox (
    connection_id INTEGER NOT NULL REFERENCES github_connections(id),
    seq INTEGER NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY(connection_id,seq)
);
CREATE TABLE github_versions (
    connection_id INTEGER NOT NULL REFERENCES github_connections(id),
    kind TEXT NOT NULL,
    entity_key TEXT NOT NULL,
    field TEXT NOT NULL,
    version TEXT,
    PRIMARY KEY(connection_id,kind,entity_key,field)
);
CREATE TABLE github_files (
    connection_id INTEGER NOT NULL REFERENCES github_connections(id),
    path TEXT NOT NULL,
    sha TEXT NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY(connection_id,path)
);
CREATE TABLE github_entity_map (
    connection_id INTEGER NOT NULL REFERENCES github_connections(id),
    kind TEXT NOT NULL,
    entity_key TEXT NOT NULL,
    local_id INTEGER NOT NULL,
    catalog TEXT,
    PRIMARY KEY(connection_id,kind,entity_key)
);
CREATE INDEX github_entity_local ON github_entity_map(connection_id,kind,local_id);
CREATE TABLE github_suppressed (
    connection_id INTEGER NOT NULL REFERENCES github_connections(id),
    entity_key TEXT NOT NULL,
    PRIMARY KEY(connection_id,entity_key)
);
CREATE TABLE github_attempt (
    connection_id INTEGER PRIMARY KEY REFERENCES github_connections(id),
    base_head TEXT NOT NULL,
    max_seq INTEGER NOT NULL,
    candidate TEXT
);
CREATE TABLE github_rejections (
    connection_id INTEGER NOT NULL REFERENCES github_connections(id),
    seq INTEGER NOT NULL,
    code TEXT NOT NULL,
    PRIMARY KEY(connection_id,seq)
);
