-- migrate:up
CREATE TABLE rate_limits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    identifier TEXT NOT NULL,  -- IP address or user ID
    endpoint TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_rate_limits_identifier ON rate_limits(identifier, endpoint, window_start);

-- migrate:down
DROP TABLE IF EXISTS rate_limits;
