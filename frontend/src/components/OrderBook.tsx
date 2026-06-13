import type { OrderBook as Book } from "../api/types";
import { formatPrice } from "../lib/format";

export default function OrderBook({ book }: { book: Book }) {
  // Asks displayed highest->lowest so the cheapest ask sits just above the spread.
  const asks = [...book.asks].reverse();
  const bids = book.bids; // already highest->lowest

  return (
    <div className="card" style={{ minWidth: 220 }}>
      <h3>Order book</h3>

      <table>
        <thead>
          <tr>
            <th>Price</th>
            <th style={{ textAlign: "right" }}>Qty</th>
          </tr>
        </thead>
        <tbody>
          {asks.map((lvl) => (
            <tr key={`a${lvl.price}`} className="neg">
              <td className="mono">{formatPrice(lvl.price)}</td>
              <td className="mono" style={{ textAlign: "right" }}>
                {lvl.quantity}
              </td>
            </tr>
          ))}

          {/* the spread row */}
          <tr>
            <td colSpan={2} className="dim" style={{ textAlign: "center" }}>
              {spreadLabel(book)}
            </td>
          </tr>

          {bids.map((lvl) => (
            <tr key={`b${lvl.price}`} className="pos">
              <td className="mono">{formatPrice(lvl.price)}</td>
              <td className="mono" style={{ textAlign: "right" }}>
                {lvl.quantity}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {asks.length === 0 && bids.length === 0 && <p className="dim">Empty book</p>}
    </div>
  );
}

function spreadLabel(book: Book): string {
  const bestBid = book.bids[0]?.price;
  const bestAsk = book.asks[0]?.price;
  if (bestBid == null || bestAsk == null) return "—";
  return `spread ${bestAsk - bestBid}¢`;
}
