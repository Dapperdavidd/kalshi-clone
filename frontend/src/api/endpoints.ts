import { api, apiText } from "./client";
import type {
  Market,
  OrderBook,
  TradePrint,
  Order,
  Position,
  Portfolio,
  PlaceOrderResult,
  KEvent,
} from "./types";

// --- auth ---
export const signup = (email: string, password: string) =>
  api<{ id: number }>("/v1/auth/signup", {
    method: "POST",
    body: { email, password },
    auth: false,
  });

// login returns the bare JWT as a text body, not JSON — read it as text and
// wrap it so callers get the same {token} shape as Google login.
export const login = async (email: string, password: string): Promise<{ token: string }> => {
  const token = await apiText("/v1/auth/login", {
    method: "POST",
    body: { email, password },
    auth: false,
  });
  return { token };
};

export const googleLogin = (credential: string) =>
  api<{ token: string }>("/v1/auth/google", {
    method: "POST",
    body: { credential },
    auth: false,
  });

// --- events (public) ---
export const getEvents = () => api<KEvent[]>("/v1/events", { auth: false });
export const getEvent = (id: number) => api<KEvent>(`/v1/events/${id}`, { auth: false });
export const createEvent = (input: {
  title: string;
  category: string;
  options: { label: string; initial_price: number }[];
}) => api<{ event_id: number; options: number }>("/v1/events", { method: "POST", body: input });

// --- markets (public) ---
export const getMarkets = () => api<Market[]>("/v1/markets", { auth: false });
export const getMarket = (id: number) => api<Market>(`/v1/markets/${id}`, { auth: false });
export const getOrderBook = (id: number) =>
  api<OrderBook>(`/v1/markets/${id}/orderbook`, { auth: false });
export const getTrades = (id: number) =>
  api<TradePrint[]>(`/v1/markets/${id}/trades`, { auth: false });

// --- trading (authenticated) ---
export const placeOrder = (input: {
  market_id: number;
  side: "buy" | "sell";
  price: number;
  quantity: number;
}) => api<PlaceOrderResult>("/v1/orders", { method: "POST", body: input });

export const getMyOrders = () => api<Order[]>("/v1/orders");
export const cancelOrder = (id: number) =>
  api<{ cancelled: number; refunded: number }>(`/v1/orders/${id}`, { method: "DELETE" });

// --- account (authenticated) ---
export const getPositions = () => api<Position[]>("/v1/positions");
export const getBalance = () => api<{ balance_cents: number }>("/v1/balance");
export const getPortfolio = () => api<Portfolio>("/v1/portfolio");

// --- admin ---
export const resolveMarket = (id: number, outcome: "yes" | "no") =>
  api<{ market_id: number; outcome: string }>(`/v1/markets/${id}/resolve`, {
    method: "POST",
    body: { outcome },
  });
