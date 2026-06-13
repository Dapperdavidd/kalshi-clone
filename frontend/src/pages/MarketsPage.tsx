import { useSearchParams } from "react-router-dom";
import { getMarkets } from "../api/endpoints";
import { useAsync } from "../lib/useAsync";
import { marketMeta } from "../lib/marketMeta";
import MarketCard from "../components/MarketCard";
import Skeleton from "../components/Skeleton";

export default function MarketsPage() {
  const { data: markets, loading, error } = useAsync(getMarkets, []);
  const [params] = useSearchParams();
  const q = (params.get("q") ?? "").toLowerCase();
  const cat = params.get("cat");

  if (loading) {
    return (
      <div>
        <h1>Trending markets</h1>
        <div className="market-grid-list">
          {[0, 1, 2, 3].map((i) => (
            <Skeleton key={i} height={150} />
          ))}
        </div>
      </div>
    );
  }
  if (error) return <p className="error">Couldn’t load markets: {error}</p>;
  if (!markets || markets.length === 0) return <p className="dim">No markets yet.</p>;

  const filtered = markets.filter((m) => {
    if (q && !m.question.toLowerCase().includes(q)) return false;
    if (cat && cat !== "Trending" && marketMeta(m.question).category !== cat) return false;
    return true;
  });

  return (
    <div>
      <h1>{cat && cat !== "Trending" ? cat : "Trending markets"}</h1>
      {filtered.length === 0 ? (
        <p className="dim">No markets match your search.</p>
      ) : (
        <div className="market-grid-list">
          {filtered.map((m) => (
            <MarketCard key={m.id} market={m} />
          ))}
        </div>
      )}
    </div>
  );
}
