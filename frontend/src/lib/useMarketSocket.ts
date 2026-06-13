import { useEffect, useRef } from "react";
import type { MarketEvent } from "../api/types";

// Derive the ws:// URL from the http(s) API base so dev and prod both work.
function wsUrl(marketId: number): string {
  const base = import.meta.env.VITE_API_URL.replace(/^http/, "ws");
  return `${base}/ws?market_id=${marketId}`;
}

/** Subscribe to a market's live events. `onEvent` is called for each message.
 *  Reconnects with backoff if the socket drops. Closes on unmount. */
export function useMarketSocket(marketId: number, onEvent: (ev: MarketEvent) => void) {
  // Keep the latest callback in a ref so reconnect logic doesn't depend on it.
  const handlerRef = useRef(onEvent);
  handlerRef.current = onEvent;

  useEffect(() => {
    let socket: WebSocket | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let closedByUs = false;
    let backoff = 1000; // ms, grows to a cap on repeated failures

    function connect() {
      socket = new WebSocket(wsUrl(marketId));

      socket.onmessage = (msg) => {
        try {
          const ev = JSON.parse(msg.data) as MarketEvent;
          handlerRef.current(ev);
        } catch {
          // ignore malformed frames
        }
      };

      socket.onopen = () => {
        backoff = 1000; // reset backoff once we're connected
      };

      socket.onclose = () => {
        if (closedByUs) return;
        // Server or network dropped us; retry with capped backoff.
        reconnectTimer = setTimeout(connect, backoff);
        backoff = Math.min(backoff * 2, 15000);
      };
    }

    connect();

    return () => {
      closedByUs = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      socket?.close();
    };
  }, [marketId]); // reconnect when the watched market changes
}
