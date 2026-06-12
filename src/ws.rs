use actix_web::{HttpRequest, HttpResponse, web};
use futures_util::StreamExt;
use tokio::sync::broadcast::error::RecvError;

use crate::error::AppError;
use crate::events::Broadcaster;

#[derive(serde::Deserialize)]
pub struct WsQuery {
    /// Which market this socket wants events for. If absent, receive all.
    pub market_id: Option<i64>,
}

pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    query: web::Query<WsQuery>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, AppError> {
    // Upgrade the HTTP request to a WebSocket. `session` lets us send frames;
    // `msg_stream` yields frames the client sends us.
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)
        .map_err(|e| AppError::Internal(format!("websocket upgrade failed: {e}").into()))?;

    let filter_market = query.market_id;
    let mut rx = broadcaster.subscribe();

    // Drive the socket in a background task so the HTTP response (the upgrade)
    // can return immediately.
    actix_web::rt::spawn(async move {
        let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(30));

        loop {
            tokio::select! {
                // 1. A market event was published.
                event = rx.recv() => {
                    match event {
                        Ok(ev) => {
                            // Forward only events for the watched market (or all).
                            if filter_market.is_none() || filter_market == Some(ev.market_id()) {
                                let json = serde_json::to_string(&ev).unwrap_or_default();
                                if session.text(json).await.is_err() {
                                    break; // client went away
                                }
                            }
                        }
                        Err(RecvError::Lagged(_)) => {
                            // Too slow; we dropped some events. Keep going —
                            // the client can re-fetch the book over REST.
                            continue;
                        }
                        Err(RecvError::Closed) => break,
                    }
                }

                // 2. The client sent us a frame (ping, close, or text we ignore).
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        actix_ws::Message::Ping(bytes) => {
                            if session.pong(&bytes).await.is_err() { break; }
                        }
                        actix_ws::Message::Close(_) => break,
                        _ => {} // we don't accept commands over the socket
                    }
                }

                // 3. Heartbeat: ping the client so dead connections are detected
                //    and proxies don't time the socket out.
                _ = heartbeat.tick() => {
                    if session.ping(b"").await.is_err() { break; }
                }
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
