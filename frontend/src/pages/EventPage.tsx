import { Link, useParams } from "react-router-dom";
import { getEvent } from "../api/endpoints";
import { useAsync } from "../lib/useAsync";
import { categoryMeta, multiplier, ROW_COLORS } from "../lib/marketMeta";

/** An event = a group of binary markets. Lists every option; clicking one opens
 *  that option's market detail page to trade it. */
export default function EventPage() {
  const { id } = useParams();
  const eventId = Number(id);
  const event = useAsync(() => getEvent(eventId), [eventId]);

  if (event.loading) return <p>Loading…</p>;
  if (event.error) return <p className="error">{event.error}</p>;
  if (!event.data) return <p className="dim">Event not found.</p>;

  const meta = categoryMeta(event.data.category);

  return (
    <div style={{ maxWidth: 760, margin: "0 auto" }}>
      <Link to="/" className="dim" style={{ fontSize: 14 }}>
        ← Markets
      </Link>

      <div className="row" style={{ alignItems: "center", marginTop: 12, marginBottom: 4 }}>
        <div className="mcard-icon" style={{ background: meta.color }}>
          {meta.emoji}
        </div>
        <span className="mcard-cat">{event.data.category}</span>
        {event.data.is_new && <span className="badge-new">NEW</span>}
      </div>
      <h1>{event.data.title}</h1>

      <div className="card" style={{ padding: 0 }}>
        {event.data.options.map((o, i) => (
          <Link
            key={o.market_id}
            to={`/markets/${o.market_id}`}
            className="erow-link"
          >
            <div className="erow-name">
              <span>{o.label}</span>
              <div className="erow-bar">
                <div
                  style={{
                    width: `${o.yes_price ?? 0}%`,
                    background: ROW_COLORS[i % ROW_COLORS.length],
                  }}
                />
              </div>
            </div>
            <span className="erow-mult">{multiplier(o.yes_price)}</span>
            <span className={`pct ${i === 0 ? "pct-lead" : ""} ${o.yes_price == null ? "pct-empty" : ""}`}>
              {o.yes_price != null ? `${o.yes_price}%` : "—"}
            </span>
          </Link>
        ))}
      </div>
    </div>
  );
}
