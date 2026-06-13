import { Link } from "react-router-dom";
import type { Market } from "../api/types";
import { getOrderBook } from "../api/endpoints";
import { useAsync } from "../lib/useAsync";
import { marketMeta, impliedYes, fakeVolume } from "../lib/marketMeta";

/** A single market rendered as a Kalshi-style event card. Fetches its own
 *  order book to show an implied YES % pill (the signature Kalshi element). */
export default function MarketCard({ market }: { market: Market }) {
  const meta = marketMeta(market.question);
  const book = useAsync(() => getOrderBook(market.id), [market.id]);
  const yes = impliedYes(book.data);
  const resolved = market.status === "resolved";

  return (
    <Link to={`/markets/${market.id}`}>
      <div className="mcard">
        <div className="mcard-head">
          <div className="mcard-icon" style={{ background: meta.color }}>
            {meta.emoji}
          </div>
          <span className="mcard-cat">{meta.category}</span>
          {resolved && (
            <span className="mcard-cat" style={{ marginLeft: "auto" }}>
              Resolved{market.outcome ? ` · ${market.outcome.toUpperCase()}` : ""}
            </span>
          )}
        </div>

        <div className="mcard-title">{market.question}</div>

        <div className="mcard-outcome">
          <span className="mcard-outcome-name">Yes</span>
          {yes != null ? (
            <span className="pct">{yes}%</span>
          ) : (
            <span className="pct pct-empty">—</span>
          )}
        </div>

        <div className="mcard-foot">
          <span>{fakeVolume(market.id)}</span>
          <span>{resolved ? "Settled" : "Open"}</span>
        </div>
      </div>
    </Link>
  );
}
