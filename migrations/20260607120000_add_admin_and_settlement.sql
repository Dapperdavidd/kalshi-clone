-- Admin flag: gates who can resolve markets.
ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- Settlement columns on markets: the resolved outcome and when it happened.
-- status already exists ('active' by default); settlement flips it to 'resolved'.
ALTER TABLE markets ADD COLUMN outcome TEXT;          -- 'yes' | 'no' | NULL
ALTER TABLE markets ADD COLUMN resolved_at TIMESTAMPTZ;
