CREATE TABLE IF NOT EXISTS request_processor
(
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT             NOT NULL,
    language    TEXT             NOT NULL,
    runtime     TEXT             NOT NULL,
    code        TEXT             NOT NULL
);