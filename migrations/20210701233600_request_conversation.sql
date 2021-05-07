CREATE TABLE IF NOT EXISTS request_conversation ( 
    id uuid NOT NULL,
    created_at TEXT NOT NULL,    -- RFC3339 string
    request_processor uuid NOT NULL REFERENCES request_processor (id),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS conversation_audit_item (
    id uuid NOT NULL,
    kind TEXT NOT NULL,
    created_at TEXT NOT NULL,    -- RFC3339 string
    request_conversation uuid NOT NULL REFERENCES request_conversation (id),
    parent uuid NULL REFERENCES conversation_audit_item (id),
    inc INTEGER NULL,
    payload TEXT NOT NULL,

    PRIMARY KEY (id)
);
