-- Events group several independent binary (Yes/No) markets, the way Kalshi
-- models an "event" (e.g. "2028 Democratic nominee") with one sub-market per
-- option. The matching engine is unchanged: each option is still a normal
-- market row that trades Yes/No on its own book.
CREATE TABLE events (
    id         BIGSERIAL PRIMARY KEY,
    title      TEXT NOT NULL,
    category   TEXT NOT NULL DEFAULT 'Trending',
    status     TEXT NOT NULL DEFAULT 'active',
    is_new     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Link each market to its event, give it a short option label for the card row,
-- and cache a "current Yes price" (1..99) so cards show a % even on a thin book.
ALTER TABLE markets ADD COLUMN event_id    BIGINT REFERENCES events(id);
ALTER TABLE markets ADD COLUMN option_label TEXT;
ALTER TABLE markets ADD COLUMN last_price   INTEGER;  -- 1..99, cached Yes price

CREATE INDEX idx_markets_event ON markets(event_id);

-- Wrap the 3 legacy standalone markets into single-option events so the whole
-- catalogue is uniform (everything belongs to an event).
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will BTC close above $100k on Dec 31?', 'Crypto', false) RETURNING id)
UPDATE markets SET event_id = (SELECT id FROM e), option_label = 'Yes', last_price = 63 WHERE question = 'Will BTC close above $100k on Dec 31?';

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will the Fed cut rates in Q3?', 'Economics', false) RETURNING id)
UPDATE markets SET event_id = (SELECT id FROM e), option_label = 'Yes', last_price = 41 WHERE question = 'Will the Fed cut rates in Q3?';

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will Nigeria win AFCON 2026?', 'Sports', false) RETURNING id)
UPDATE markets SET event_id = (SELECT id FROM e), option_label = 'Yes', last_price = 18 WHERE question = 'Will Nigeria win AFCON 2026?';

-- ───────────────────────── Seed: 30+ events / 100+ markets ─────────────────────────
-- Helper pattern: insert an event, then one market per option. The market's
-- `question` is "<event title> — <option label>"; %s are independent per option.

-- Politics
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('2028 Democratic presidential nominee','Politics', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Gavin Newsom',22),('Alexandria Ocasio-Cortez',9),('Pete Buttigieg',14),('Josh Shapiro',12),('Gretchen Whitmer',8)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('2028 Republican presidential nominee','Politics', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('J.D. Vance',31),('Marco Rubio',30),('Ron DeSantis',12),('Donald Trump Jr.',7)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('2028 U.S. Presidential Election winner','Politics', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Republican Party',52),('Democratic Party',46),('Independent',3)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Balance of power after 2026 midterms','Politics', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Republican House & Senate',38),('Split Congress',41),('Democratic House & Senate',21)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will Congressional salaries increase?','Politics', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Before Jan 1, 2030',47),('Before Jan 1, 2029',37),('Before Jan 1, 2028',19),('Before Jan 1, 2027',6)) o(lbl,p);

-- Sports
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('2026 Men''s World Cup Winner','Sports', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('France',17),('Spain',17),('Brazil',15),('Argentina',13),('England',11)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('NBA Champion 2026','Sports', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Boston Celtics',24),('Oklahoma City Thunder',21),('Denver Nuggets',14),('New York Knicks',12),('Dallas Mavericks',9)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Super Bowl LXI Winner','Sports', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Kansas City Chiefs',18),('San Francisco 49ers',15),('Baltimore Ravens',13),('Detroit Lions',11)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Ballon d''Or 2026','Sports', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Lamine Yamal',28),('Jude Bellingham',19),('Kylian Mbappé',17),('Erling Haaland',14)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('RBC Canadian Open Winner','Sports', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Sam Burns',17),('Tommy Fleetwood',12),('Ludvig Åberg',11),('Rory McIlroy',10)) o(lbl,p);

-- Crypto
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Bitcoin price on Dec 31, 2026','Crypto', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Above $150k',38),('Above $120k',54),('Above $100k',71),('Below $80k',22)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('When will Bitcoin cross $100k again?','Crypto', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Before July 2026',31),('Before October 2026',49),('Before January 2027',68)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Which coin makes a new ATH first?','Crypto', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Bitcoin',44),('Ethereum',23),('Solana',19),('XRP',12)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Ethereum price end of 2026','Crypto', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Above $5,000',33),('Above $3,000',57),('Below $2,000',18)) o(lbl,p);

-- Economics
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Fed decision at the next FOMC meeting','Economics', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Cut 25 bps',49),('Hold rates',44),('Cut 50 bps',9)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('US recession in 2026?','Economics', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Yes',34),('No',66)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('US inflation rate end of 2026','Economics', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Above 3.5%',29),('Between 2-3.5%',55),('Below 2%',16)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('How high will US gas prices get this year?','Economics', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Above $4.60',50),('Above $4.80',38),('Above $5.00',21)) o(lbl,p);

-- Culture
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Best Picture at the 2027 Oscars','Culture', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Sinners',26),('One Battle After Another',22),('Wicked: For Good',15),('Hamnet',12)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Grammy Album of the Year','Culture', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Billie Eilish',24),('Kendrick Lamar',21),('Taylor Swift',18),('Charli XCX',14)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Highest grossing film of 2026','Culture', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Avatar: Fire and Ash',31),('Toy Story 5',19),('The Mandalorian & Grogu',16),('Avengers: Doomsday',28)) o(lbl,p);

-- Climate
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will 2026 be the hottest year on record?','Climate', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Yes',57),('No',43)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Number of Atlantic hurricanes in 2026','Climate', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('More than 18',32),('13 to 18',46),('Fewer than 13',22)) o(lbl,p);

-- Finance
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('S&P 500 level on Dec 31, 2026','Finance', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Above 7,000',41),('Above 6,500',63),('Below 6,000',24)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Which company hits a $5T market cap first?','Finance', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Nvidia',47),('Apple',24),('Microsoft',19),('Alphabet',8)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will OpenAI or Anthropic IPO first?','Finance', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Anthropic',78),('OpenAI',19)) o(lbl,p);

-- Tech & Science
WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Top-ranked AI model at end of 2026','Tech & Science', true) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Anthropic',39),('OpenAI',31),('Google DeepMind',22),('xAI',6)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will GPT-6 release in 2026?','Tech & Science', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Yes',43),('No',57)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('Will the US confirm that aliens exist?','Tech & Science', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Before Jan 20, 2029',28),('Before 2028',21),('Before 2027',9)) o(lbl,p);

WITH e AS (INSERT INTO events (title, category, is_new) VALUES ('When will traffic at the Strait of Hormuz return to normal?','Tech & Science', false) RETURNING id, title)
INSERT INTO markets (question, event_id, option_label, last_price) SELECT e.title||' — '||o.lbl, e.id, o.lbl, o.p FROM e CROSS JOIN (VALUES ('Before Sep 1, 2026',52),('Before Aug 1, 2026',44),('Before Jul 1, 2026',27)) o(lbl,p);
