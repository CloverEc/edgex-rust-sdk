use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::client::ClientError;
use std::time::{SystemTime, UNIX_EPOCH};

const WS_URL: &str = "wss://quote.edgex.exchange";

#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<u64>,
    #[serde(flatten)]
    pub payload: Value,
}

pub struct EdgeXWebSocket {
    // For now, expose basic stream handling or a loop.
    // In SDKs, usually we provide a callback or channel.
    // Simplifying for this task: connect and return stream? 
    // Or provide a run loop?
}

impl EdgeXWebSocket {
    pub async fn connect() -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, ClientError> {
        let (ws_stream, _) = connect_async(Url::parse(WS_URL).unwrap()).await
            .map_err(|e| ClientError::ApiError(e.to_string()))?;
        Ok(ws_stream)
    }

    pub async fn subscribe(stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, channel: &str) -> Result<(), ClientError> {
        let msg = serde_json::json!({
            "type": "subscribe",
            "channel": channel
        });
        stream.send(Message::Text(msg.to_string())).await
            .map_err(|e| ClientError::ApiError(e.to_string()))?;
        Ok(())
    }
    
    // Helper to handle ping/pong automatically if wrapped in a loop.
    // User of SDK will likely consume the stream.
    // We can provide a helper "handle_ping"
    pub async fn handle_ping(stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, msg: &Message) -> Result<bool, ClientError> {
        if let Message::Text(text) = msg {
            if let Ok(v) = serde_json::from_str::<Value>(text) {
                if v["type"] == "ping" {
                    // Send Pong
                    let time = v["time"].as_u64().or_else(|| v["time"].as_str().and_then(|s| s.parse().ok())).unwrap_or(0);
                    let pong = serde_json::json!({
                        "type": "pong",
                        "time": time
                    });
                    stream.send(Message::Text(pong.to_string())).await
                        .map_err(|e| ClientError::ApiError(e.to_string()))?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
