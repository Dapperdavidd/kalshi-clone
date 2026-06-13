import { Link } from "react-router-dom";
import type { KEvent } from "../api/types";
import { categoryMeta, multiplier, ROW_COLORS } from "../lib/marketMeta";
import { fakeVolume } from "../lib/marketMeta";

/** A Kalshi-style event card: category, title, up to 3 option rows (name +
 *  progress underline + multiplier + % pill), and a NEW / N-markets footer. */
export default function EventCard({ event }: { event: KEvent }) {
  const meta = categoryMeta(event.category);
  const rows = event.options.slice(0, 3);

  return (
    <Link to={`/events/${event.id}`}>
      <div className="mcard">
        <div className="mcard-head">
          <div className="mcard-icon" style={{ background: meta.color }}>
            {meta.emoji}
          </div>
          <span className="mcard-cat">{event.category}</span>
        </div>

        <div className="mcard-title">{event.title}</div>

        <div className="ecard-options">
          {rows.map((o, i) => (
            <div className="erow" key={o.market_id}>
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
            </div>
          ))}
        </div>

        <div className="mcard-foot">
          {event.is_new ? (
            <span className="badge-new">NEW</span>
          ) : (
            <span>{fakeVolume(event.id)}</span>
          )}
          <span>{event.market_count} markets</span>
        </div>
      </div>
    </Link>
  );
}
