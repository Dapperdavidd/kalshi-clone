import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router-dom";
import { createEvent } from "../api/endpoints";
import { ApiError } from "../api/client";
import { useAuth } from "../auth/AuthContext";
import { useToast } from "../components/Toast";

const CATEGORIES = [
  "Politics",
  "Sports",
  "Crypto",
  "Economics",
  "Culture",
  "Climate",
  "Finance",
  "Tech & Science",
];

interface OptionInput {
  label: string;
  price: number;
}

export default function CreateMarketPage() {
  const { isAdmin } = useAuth();
  const navigate = useNavigate();
  const toast = useToast();

  const [title, setTitle] = useState("");
  const [category, setCategory] = useState(CATEGORIES[0]);
  const [options, setOptions] = useState<OptionInput[]>([
    { label: "Yes", price: 50 },
    { label: "No", price: 50 },
  ]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  if (!isAdmin) {
    return <p className="dim">Admin only. Resolve/create markets require an admin account.</p>;
  }

  function setOption(i: number, patch: Partial<OptionInput>) {
    setOptions((os) => os.map((o, j) => (j === i ? { ...o, ...patch } : o)));
  }
  function addOption() {
    setOptions((os) => [...os, { label: "", price: 50 }]);
  }
  function removeOption(i: number) {
    setOptions((os) => (os.length > 1 ? os.filter((_, j) => j !== i) : os));
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const payload = {
        title: title.trim(),
        category,
        options: options
          .filter((o) => o.label.trim())
          .map((o) => ({ label: o.label.trim(), initial_price: o.price })),
      };
      const res = await createEvent(payload);
      toast(`Created “${payload.title}” with ${res.options} options`);
      navigate(`/events/${res.event_id}`);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "could not create market");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div style={{ maxWidth: 560, margin: "0 auto" }}>
      <h1>New market</h1>
      <form onSubmit={handleSubmit} className="card col">
        <div>
          <label className="field-label">Question / event title</label>
          <input
            className="input"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="e.g. 2028 Democratic presidential nominee"
            required
          />
        </div>

        <div>
          <label className="field-label">Category</label>
          <select className="select" value={category} onChange={(e) => setCategory(e.target.value)}>
            {CATEGORIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="field-label">Options (each becomes a Yes/No market — price 1–99¢)</label>
          <div className="col" style={{ gap: 8 }}>
            {options.map((o, i) => (
              <div className="opt-row" key={i}>
                <input
                  className="input"
                  value={o.label}
                  onChange={(e) => setOption(i, { label: e.target.value })}
                  placeholder={`Option ${i + 1}`}
                />
                <input
                  className="input"
                  type="number"
                  min={1}
                  max={99}
                  value={o.price}
                  onChange={(e) => setOption(i, { price: Number(e.target.value) })}
                />
                <button type="button" className="btn" onClick={() => removeOption(i)}>
                  ✕
                </button>
              </div>
            ))}
          </div>
          <button type="button" className="btn" style={{ marginTop: 8 }} onClick={addOption}>
            + Add option
          </button>
        </div>

        <button type="submit" disabled={busy} className="btn btn-primary btn-block btn-lg">
          {busy ? "Creating…" : "Create market"}
        </button>

        {error && <p className="error" style={{ margin: 0 }}>{error}</p>}
      </form>
    </div>
  );
}
