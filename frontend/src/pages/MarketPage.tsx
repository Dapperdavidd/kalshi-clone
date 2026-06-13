import { useCallback } from "react";
import { useParams } from "react-router-dom";
import { getMarket, getOrderBook, getTrades } from "../api/endpoints";
import { useAsync } from "../lib/useAsync";
import { useMarketSocket } from "../lib/useMarketSocket";
import type { MarketEvent } from "../api/types";
import { marketMeta, impliedYes } from "../lib/marketMeta";
import OrderBook from "../components/OrderBook";
import TradesTape from "../components/TradesTape";
import PriceChart from "../components/PriceChart";
import OrderTicket from "../components/OrderTicket";
import ResolveControls from "../components/ResolveControls";

export default function MarketPage() {
  const { id } = useParams();
  const marketId = Number(id);

  const market = useAsync(() => getMarket(marketId), [marketId]);
  const book = useAsync(() => getOrderBook(marketId), [marketId]);
  const trades = useAsync(() => getTrades(marketId), [marketId]);

  const refresh = () => {
    book.reload();
    trades.reload();
    market.reload();
  };

  const onEvent = useCallback(
    (ev: MarketEvent) => {
      if (ev.type === "resolved") {
        market.reload();
      } else {
        refresh();
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  useMarketSocket(marketId, onEvent);

  if (market.loading) return <p>Loading…</p>;
  if (market.error) return <p className="error">{market.error}</p>;
  if (!market.data) return <p className="dim">Market not found.</p>;

  const meta = marketMeta(market.data.question);
  const yes = impliedYes(book.data);
  const resolved = market.data.status === "resolved";

  return (
    <div>
      <div className="row" style={{ alignItems: "center", marginBottom: 8 }}>
        <div className="mcard-icon" style={{ background: meta.color }}>
          {meta.emoji}
        </div>
        <span className="mcard-cat">{meta.category}</span>
        {resolved && (
          <span className="mcard-cat">
            · Resolved{market.data.outcome ? ` ${market.data.outcome.toUpperCase()}` : ""}
          </span>
        )}
      </div>

      <div className="spread" style={{ alignItems: "flex-start", marginBottom: 16 }}>
        <h1 style={{ margin: 0, maxWidth: 640 }}>{market.data.question}</h1>
        {yes != null && (
          <div style={{ textAlign: "right" }}>
            <div className="mono" style={{ fontSize: 30, fontWeight: 700, color: "var(--accent)" }}>
              {yes}%
            </div>
            <div className="dim" style={{ fontSize: 13 }}>
              Yes
            </div>
          </div>
        )}
      </div>

      {trades.data && <PriceChart trades={trades.data} />}

      <div className="market-grid" style={{ marginTop: 16 }}>
        <div className="col">
          <div className="row" style={{ flexWrap: "wrap", alignItems: "flex-start" }}>
            {book.data && <OrderBook book={book.data} />}
            {trades.data && <TradesTape trades={trades.data} />}
          </div>
          <ResolveControls marketId={marketId} onResolved={refresh} />
        </div>
        <OrderTicket marketId={marketId} marketTitle={market.data.question} onPlaced={refresh} />
      </div>
    </div>
  );
}
