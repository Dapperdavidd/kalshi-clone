import { useState } from "react";
import { Link } from "react-router-dom";
import { getPortfolio, getMyOrders, cancelOrder } from "../api/endpoints";
import { useAsync } from "../lib/useAsync";
import { formatDollars, formatPrice } from "../lib/format";
import { ApiError } from "../api/client";
import type { PortfolioPosition, Order } from "../api/types";

export default function PortfolioPage() {
  const portfolio = useAsync(getPortfolio, []);
  const orders = useAsync(getMyOrders, []);

  // Re-pull both panels after a successful cancel.
  function reloadAll() {
    portfolio.reload();
    orders.reload();
  }

  if (portfolio.loading) return <p>Loading portfolio…</p>;
  if (portfolio.error) return <p className="error">Couldn’t load portfolio: {portfolio.error}</p>;
  if (!portfolio.data) return null;

  const { balance_cents, positions, positions_value_cents, equity_cents } = portfolio.data;

  return (
    <div style={{ maxWidth: 720, margin: "0 auto" }}>
      <h1>Portfolio</h1>

      <Summary balance={balance_cents} positionsValue={positions_value_cents} equity={equity_cents} />

      <h2>Positions</h2>
      {positions.length === 0 ? (
        <p className="dim">No open positions yet.</p>
      ) : (
        <PositionsTable positions={positions} />
      )}

      <h2>Open orders</h2>
      <OpenOrders
        orders={orders.data ?? []}
        loading={orders.loading}
        error={orders.error}
        onCancelled={reloadAll}
      />
    </div>
  );
}

function Summary({
  balance,
  positionsValue,
  equity,
}: {
  balance: number;
  positionsValue: number;
  equity: number;
}) {
  return (
    <div className="card row" style={{ gap: 24, marginBottom: 24 }}>
      <Stat label="Cash" value={formatDollars(balance)} />
      <Stat label="Positions" value={formatDollars(positionsValue)} />
      <Stat label="Equity" value={formatDollars(equity)} emphasize />
    </div>
  );
}

function Stat({ label, value, emphasize }: { label: string; value: string; emphasize?: boolean }) {
  return (
    <div>
      <div className="dim" style={{ fontSize: 13 }}>
        {label}
      </div>
      <div className="mono" style={{ fontSize: emphasize ? 24 : 20, fontWeight: emphasize ? 700 : 500 }}>
        {value}
      </div>
    </div>
  );
}

function PositionsTable({ positions }: { positions: PortfolioPosition[] }) {
  return (
    <table style={{ marginBottom: 24 }}>
      <thead>
        <tr>
          <th>Market</th>
          <th>Side</th>
          <th>Contracts</th>
          <th>Mark</th>
          <th style={{ textAlign: "right" }}>Value</th>
        </tr>
      </thead>
      <tbody>
        {positions.map((p) => {
          const isLong = p.quantity >= 0;
          return (
            <tr key={p.market_id}>
              <td>
                <Link to={`/markets/${p.market_id}`}>{p.question}</Link>
              </td>
              <td className={isLong ? "pos" : "neg"}>{isLong ? "YES" : "NO"}</td>
              <td className="mono">{Math.abs(p.quantity)}</td>
              <td className="mono">{formatPrice(p.mark_price)}</td>
              <td className="mono" style={{ textAlign: "right" }}>
                {formatDollars(p.value_cents)}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

function OpenOrders({
  orders,
  loading,
  error,
  onCancelled,
}: {
  orders: Order[];
  loading: boolean;
  error: string | null;
  onCancelled: () => void;
}) {
  const [busyId, setBusyId] = useState<number | null>(null);
  const [cancelError, setCancelError] = useState<string | null>(null);

  // Only resting orders can be cancelled; filled/cancelled ones are history.
  const resting = orders.filter(
    (o) => o.status === "working" || o.status === "partially_filled",
  );

  async function handleCancel(id: number) {
    setCancelError(null);
    setBusyId(id);
    try {
      await cancelOrder(id);
      onCancelled(); // refresh balance + orders
    } catch (err) {
      setCancelError(err instanceof ApiError ? err.message : "couldn’t cancel order");
    } finally {
      setBusyId(null);
    }
  }

  if (loading) return <p>Loading orders…</p>;
  if (error) return <p className="error">Couldn’t load orders: {error}</p>;
  if (resting.length === 0) return <p className="dim">No open orders.</p>;

  return (
    <div>
      {cancelError && <p className="error">{cancelError}</p>}
      <table>
        <thead>
          <tr>
            <th>Market</th>
            <th>Side</th>
            <th>Price</th>
            <th>Remaining</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {resting.map((o) => (
            <tr key={o.id}>
              <td>
                <Link to={`/markets/${o.market_id}`}>#{o.market_id}</Link>
              </td>
              <td className={o.side === "buy" ? "pos" : "neg"}>
                {o.side === "buy" ? "Buy YES" : "Sell YES"}
              </td>
              <td className="mono">{formatPrice(o.price)}</td>
              <td className="mono">
                {o.remaining}
                {o.status === "partially_filled" && <span className="dim"> / {o.quantity}</span>}
              </td>
              <td style={{ textAlign: "right" }}>
                <button className="btn" onClick={() => handleCancel(o.id)} disabled={busyId === o.id}>
                  {busyId === o.id ? "Cancelling…" : "Cancel"}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
