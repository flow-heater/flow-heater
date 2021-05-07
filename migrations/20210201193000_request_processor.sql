CREATE TABLE IF NOT EXISTS request_processor (
    id          uuid NOT NULL,
    name        TEXT NOT NULL,
    language    TEXT NOT NULL,
    runtime     TEXT NOT NULL,
    code        TEXT NOT NULL,
    PRIMARY KEY (id)
);
