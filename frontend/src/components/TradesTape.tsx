import type { TradePrint } from "../api/types";
import { formatPrice } from "../lib/format";

export default function TradesTape({ trades }: { trades: TradePrint[] }) {
  if (trades.length === 0) return <p className="dim">No trades yet</p>;
  return (
    <div className="card" style={{ minWidth: 220 }}>
      <h3>Recent trades</h3>
      <table>
        <tbody>
          {trades.map((t, i) => (
            <tr key={i}>
              <td className="mono">{formatPrice(t.price)}</td>
              <td className="mono" style={{ textAlign: "right" }}>
                {t.quantity}
              </td>
              <td className="dim" style={{ textAlign: "right" }}>
                {new Date(t.created_at).toLocaleTimeString()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
