import { useState } from "react";
import { resolveMarket } from "../api/endpoints";
import { ApiError } from "../api/client";
import { useAuth } from "../auth/AuthContext";

export default function ResolveControls({
  marketId,
  onResolved,
}: {
  marketId: number;
  onResolved: () => void;
}) {
  const { isAdmin } = useAuth();
  const [error, setError] = useState<string | null>(null);
  if (!isAdmin) return null;

  async function resolve(outcome: "yes" | "no") {
    setError(null);
    try {
      await resolveMarket(marketId, outcome);
      onResolved();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "resolve failed");
    }
  }

  return (
    <div
      className="card"
      style={{ borderStyle: "dashed", borderColor: "#f59e0b", marginTop: 16 }}
    >
      <strong>Admin:</strong> resolve this market{" "}
      <button className="btn btn-green" onClick={() => resolve("yes")}>
        YES
      </button>{" "}
      <button className="btn btn-red" onClick={() => resolve("no")}>
        NO
      </button>
      {error && <p className="error">{error}</p>}
    </div>
  );
}
