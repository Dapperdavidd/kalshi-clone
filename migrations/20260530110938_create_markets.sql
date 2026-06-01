-- Add migration script here
CREATE TABLE markets (
    id       BIGSERIAL PRIMARY KEY,
    question TEXT NOT NULL,
    status   TEXT NOT NULL DEFAULT 'active',
    rail     TEXT NOT NULL DEFAULT 'native'
);

INSERT INTO markets (question, status, rail) VALUES
    ('Will BTC close above $100k on Dec 31?', 'active', 'native'),
    ('Will the Fed cut rates in Q3?',         'active', 'native'),
    ('Will Nigeria win AFCON 2026?',          'active', 'native');