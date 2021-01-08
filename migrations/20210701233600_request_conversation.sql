CREATE TABLE IF NOT EXISTS request_conversation ( 
    id TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,    -- unix epoch UTC
    request_processor TEXT NOT NULL,

    FOREIGN KEY(request_processor) REFERENCES request_processor(id)
);

CREATE TABLE IF NOT EXISTS conversation_audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,    -- unix epoch UTC
    request_conversation TEXT NOT NULL,
    parent TEXT,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,

    FOREIGN KEY(request_conversation) REFERENCES request_conversation(id)
);
