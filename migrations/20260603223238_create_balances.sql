-- Add migration script here
CREATE TABLE balances (
    id      BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL UNIQUE,
    amount  BIGINT NOT NULL DEFAULT 0
);

-- Give every existing user a starting balance of $1,000 (100000 cents).
INSERT INTO balances (user_id, amount)
SELECT id, 100000 FROM users;
