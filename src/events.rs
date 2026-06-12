use tokio::sync::broadcast;

/// Everything we push to clients. `market_id` lets each socket filter to the
/// market it's watching. Serializes to JSON like:
///   {"type":"trade","market_id":1,"price":60,"quantity":4}
///   {"type":"book","market_id":1,"best_bid":60,"best_ask":null}
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarketEvent {
    Trade { market_id: i64, price: i32, quantity: i32 },
    Book { market_id: i64, best_bid: Option<i32>, best_ask: Option<i32> },
    Resolved { market_id: i64, outcome: String },
}

impl MarketEvent {
    pub fn market_id(&self) -> i64 {
        match self {
            MarketEvent::Trade { market_id, .. }
            | MarketEvent::Book { market_id, .. }
            | MarketEvent::Resolved { market_id, .. } => *market_id,
        }
    }
}

/// Thin wrapper around a broadcast Sender, shared as app data. Cloning it is
/// cheap (it's an Arc inside). `publish` never blocks and never errors when
/// there are no subscribers — a fire-and-forget notification.
#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<MarketEvent>,
}

impl Broadcaster {
    pub fn new() -> Self {
        // Capacity = how many unread messages a slow client can lag before it
        // starts dropping the oldest. 256 is plenty for a demo.
        let (tx, _rx) = broadcast::channel(256);
        Broadcaster { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MarketEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: MarketEvent) {
        // Err just means "no subscribers right now" — fine, drop it.
        let _ = self.tx.send(event);
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}
