// TypeScript types mirroring the backend's #[derive(Serialize)] structs,
// field-for-field (snake_case JSON), so no remapping is needed.

export interface Market {
  id: number;
  question: string;
  status: string; // "active" | "resolved"
  rail: string;
  outcome?: string | null; // present only after resolution
}

export interface BookLevel {
  price: number; // 1..99 cents
  quantity: number; // total contracts at this level
}

export interface OrderBook {
  market_id: number;
  bids: BookLevel[]; // highest price first
  asks: BookLevel[]; // lowest price first
}

export interface TradePrint {
  price: number;
  quantity: number;
  created_at: string; // ISO-8601
}

export interface Order {
  id: number;
  market_id: number;
  side: "buy" | "sell";
  price: number;
  quantity: number;
  remaining: number;
  status: string; // working | partially_filled | filled | cancelled
  created_at: string;
}

export interface Position {
  market_id: number;
  quantity: number; // signed
}

export interface PortfolioPosition {
  market_id: number;
  question: string;
  quantity: number;
  mark_price: number;
  value_cents: number;
}

export interface Portfolio {
  balance_cents: number;
  positions: PortfolioPosition[];
  positions_value_cents: number;
  equity_cents: number;
}

/** Response of POST /v1/orders */
export interface PlaceOrderResult {
  order_id: number;
  filled: number;
  remaining: number;
  fills: number;
}

/** Live events pushed over the WebSocket. Discriminated union mirroring the
 *  Rust #[serde(tag = "type")] enum. */
export type MarketEvent =
  | { type: "trade"; market_id: number; price: number; quantity: number }
  | { type: "book"; market_id: number; best_bid: number | null; best_ask: number | null }
  | { type: "resolved"; market_id: number; outcome: string };
