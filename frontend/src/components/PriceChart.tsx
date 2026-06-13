import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import type { TradePrint } from "../api/types";

export default function PriceChart({ trades }: { trades: TradePrint[] }) {
  if (trades.length === 0) {
    return <div className="dim" style={{ height: 200 }}>No price history yet</div>;
  }

  // Backend returns newest-first; chart wants oldest-first (left to right).
  const data = [...trades].reverse().map((t) => ({
    time: new Date(t.created_at).toLocaleTimeString(),
    price: t.price,
  }));

  return (
    <ResponsiveContainer width="100%" height={200}>
      <LineChart data={data}>
        <XAxis dataKey="time" tick={{ fontSize: 10 }} stroke="#9aa3b2" />
        <YAxis domain={[0, 100]} tick={{ fontSize: 10 }} width={30} stroke="#9aa3b2" />
        <Tooltip
          contentStyle={{
            background: "#21252e",
            border: "1px solid #2c313c",
            borderRadius: 8,
            color: "#e6e8ec",
          }}
        />
        <Line type="monotone" dataKey="price" stroke="#00d09c" dot={false} isAnimationActive={false} />
      </LineChart>
    </ResponsiveContainer>
  );
}
