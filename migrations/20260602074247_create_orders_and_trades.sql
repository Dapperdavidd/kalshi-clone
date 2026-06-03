-- Add migration script here
CREATE TABLE orders (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL,
    market_id   BIGINT NOT NULL,
    side        TEXT NOT NULL,
    price       INTEGER NOT NULL,
    quantity    INTEGER NOT NULL,
    remaining   INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'working',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE trades(
    id                  BIGSERIAL PRIMARY KEY,
    market_id           BIGINT NOT NULL,
    maker_order_id     BIGINT NOT NULL,
    taker_order_id      BIGINT NOT NULL,
    price               INTEGER NOT NULL,
    quantity            INTEGER NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);