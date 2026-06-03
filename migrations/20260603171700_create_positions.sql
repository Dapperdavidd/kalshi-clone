-- Add migration script here
CREATE TABLE positions (
    id         BIGSERIAL PRIMARY KEY,
    user_id    BIGINT NOT NULL,
    market_id  BIGINT NOT NULL,
    quantity   INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, market_id)
);
