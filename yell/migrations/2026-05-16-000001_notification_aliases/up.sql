CREATE TABLE notification_aliases (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    description VARCHAR(512),
    recipients JSONB NOT NULL,
    is_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
