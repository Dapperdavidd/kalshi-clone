import type { OrderBook } from "../api/types";

export interface MarketMeta {
  emoji: string;
  category: string;
  color: string; // icon tile background
}

const RULES: { re: RegExp; meta: MarketMeta }[] = [
  { re: /btc|bitcoin|eth|ethereum|crypto|sol|doge|xrp/i, meta: { emoji: "₿", category: "Crypto", color: "#f7931a" } },
  { re: /fed|rate|inflation|gas|oil|gdp|economy|cpi|jobs/i, meta: { emoji: "📈", category: "Economics", color: "#7c5cff" } },
  { re: /afcon|world cup|nba|nfl|game|winner|win\b|vs|match|league|cup/i, meta: { emoji: "🏆", category: "Sports", color: "#16c784" } },
  { re: /president|election|senate|house|governor|trump|nominee|vote|congress/i, meta: { emoji: "🏛️", category: "Politics", color: "#e84142" } },
  { re: /movie|music|oscar|grammy|album|tv|show|culture/i, meta: { emoji: "🎬", category: "Culture", color: "#ff6b9d" } },
];

const FALLBACK: MarketMeta = { emoji: "◆", category: "Markets", color: "#00d09c" };

export function marketMeta(question: string): MarketMeta {
  return RULES.find((r) => r.re.test(question))?.meta ?? FALLBACK;
}

/** Implied YES probability (%) from the order-book mid, or null if no book. */
export function impliedYes(book: OrderBook | null): number | null {
  if (!book) return null;
  const bid = book.bids[0]?.price;
  const ask = book.asks[0]?.price;
  if (bid != null && ask != null) return Math.round((bid + ask) / 2);
  return bid ?? ask ?? null;
}

/** A pseudo "volume" so cards read like Kalshi. Deterministic from id — purely
 *  cosmetic; the backend has no volume column yet. */
export function fakeVolume(id: number): string {
  const v = ((id * 928371) % 9000000) + 120000;
  return `$${v.toLocaleString("en-US")} vol`;
}
