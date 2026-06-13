import { useState, type FormEvent } from "react";
import { placeOrder } from "../api/endpoints";
import { ApiError } from "../api/client";
import { useAuth } from "../auth/AuthContext";
import { formatDollars } from "../lib/format";
import { Link } from "react-router-dom";
import { useToast } from "./Toast";

/**
 * Kalshi-style order ticket. The user picks YES or NO and a price (in ¢) for
 * that side, plus a quantity. We map it onto the backend's buy/sell model:
 *   - YES at P¢  → side="buy",  price=P
 *   - NO  at Q¢  → side="sell", price=100-Q   (shorting YES == buying NO)
 * In both cases the collateral the backend locks equals (chosen price × qty),
 * and a win pays 100¢/contract — so "Max payout" reads like Kalshi.
 */
export default function OrderTicket({
  marketId,
  marketTitle,
  onPlaced,
}: {
  marketId: number;
  marketTitle: string;
  onPlaced: () => void;
}) {
  const { isLoggedIn } = useAuth();
  const toast = useToast();
  const [outcome, setOutcome] = useState<"yes" | "no">("yes");
  const [price, setPrice] = useState(50); // ¢ for the chosen side
  const [quantity, setQuantity] = useState(10);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const cost = price * quantity; // cents locked as collateral
  const maxPayout = quantity * 100; // cents if it wins

  if (!isLoggedIn) {
    return (
      <div className="ticket">
        <div className="dim">{marketTitle}</div>
        <p style={{ margin: 0 }}>
          <Link to="/login">Log in</Link> to trade.
        </p>
      </div>
    );
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const side = outcome === "yes" ? "buy" : "sell";
      const backendPrice = outcome === "yes" ? price : 100 - price;
      const result = await placeOrder({ market_id: marketId, side, price: backendPrice, quantity });
      toast(`Order placed — ${result.filled} filled, ${result.remaining} resting`);
      onPlaced();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "order failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="ticket">
      <div className="ticket-tabs">
        <button type="button" className="ticket-tab active">
          Buy
        </button>
      </div>

      <div className="dim" style={{ fontWeight: 600, color: "var(--text)" }}>
        {marketTitle}
      </div>

      <div className="yn-toggle">
        <button
          type="button"
          className={`yn-btn yes ${outcome === "yes" ? "active" : ""}`}
          onClick={() => setOutcome("yes")}
        >
          <span>Yes</span>
          <span className="yn-sub">{price}¢</span>
        </button>
        <button
          type="button"
          className={`yn-btn no ${outcome === "no" ? "active" : ""}`}
          onClick={() => setOutcome("no")}
        >
          <span>No</span>
          <span className="yn-sub">{price}¢</span>
        </button>
      </div>

      <label className="col" style={{ gap: 6 }}>
        <span className="dim" style={{ fontSize: 13 }}>
          Limit price: {price}¢
        </span>
        <input
          type="range"
          min={1}
          max={99}
          value={price}
          onChange={(e) => setPrice(Number(e.target.value))}
          style={{ width: "100%", accentColor: "var(--accent)" }}
        />
      </label>

      <label className="col" style={{ gap: 6 }}>
        <span className="dim" style={{ fontSize: 13 }}>
          Contracts
        </span>
        <input
          type="number"
          min={1}
          max={10000}
          value={quantity}
          onChange={(e) => setQuantity(Number(e.target.value))}
          className="input"
        />
      </label>

      <div className="col" style={{ gap: 8 }}>
        <div className="ticket-line">
          <span>Cost</span>
          <strong className="mono">{formatDollars(cost)}</strong>
        </div>
        <div className="ticket-line">
          <span>Max payout</span>
          <strong className="mono pos">{formatDollars(maxPayout)}</strong>
        </div>
      </div>

      <button type="submit" disabled={busy} className="btn btn-primary btn-block btn-lg">
        {busy ? "Placing…" : `Buy ${outcome === "yes" ? "Yes" : "No"} · ${formatDollars(cost)}`}
      </button>

      {error && <p className="error" style={{ margin: 0 }}>{error}</p>}
    </form>
  );
}
