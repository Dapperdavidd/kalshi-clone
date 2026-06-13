import { useSearchParams } from "react-router-dom";
import { getEvents } from "../api/endpoints";
import { useAsync } from "../lib/useAsync";
import EventCard from "../components/EventCard";
import Skeleton from "../components/Skeleton";

export default function MarketsPage() {
  const { data: events, loading, error } = useAsync(getEvents, []);
  const [params] = useSearchParams();
  const q = (params.get("q") ?? "").toLowerCase();
  const cat = params.get("cat");

  if (loading) {
    return (
      <div>
        <h1>Trending markets</h1>
        <div className="market-grid-list">
          {[0, 1, 2, 3].map((i) => (
            <Skeleton key={i} height={210} />
          ))}
        </div>
      </div>
    );
  }
  if (error) return <p className="error">Couldn’t load markets: {error}</p>;
  if (!events || events.length === 0) return <p className="dim">No markets yet.</p>;

  const filtered = events.filter((e) => {
    if (cat && cat !== "Trending" && e.category !== cat) return false;
    if (q) {
      const hay = (e.title + " " + e.options.map((o) => o.label).join(" ")).toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });

  return (
    <div>
      <h1>{cat && cat !== "Trending" ? cat : "Trending markets"}</h1>
      {filtered.length === 0 ? (
        <p className="dim">No markets match your search.</p>
      ) : (
        <div className="market-grid-list">
          {filtered.map((e) => (
            <EventCard key={e.id} event={e} />
          ))}
        </div>
      )}
    </div>
  );
}
