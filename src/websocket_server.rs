use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tracing::{info, error, debug, warn};
use std::sync::Arc;
use serde_json::{json, Value};
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::config::Config;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_config::{RpcTransactionLogsFilter, RpcTransactionLogsConfig};

/// WebSocket server for Solana RPC subscriptions
pub struct SolanaWebSocketServer {
    port: u16,
    config: Arc<Config>,
}

/// Represents an active subscription
#[derive(Debug, Clone)]
struct Subscription {
    id: u64,
    method: String,
    params: Value,
    client_tx: mpsc::UnboundedSender<Message>,
}

/// Manages active subscriptions for a WebSocket connection
type SubscriptionManager = Arc<DashMap<u64, Subscription>>;

/// Global subscription counter
static SUBSCRIPTION_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl SolanaWebSocketServer {
    pub fn new(port: u16, config: Arc<Config>) -> Self {
        Self { port, config }
    }

    /// Start the WebSocket server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = Router::new()
            .route("/", get(websocket_handler))
            .with_state(self.config.clone());

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting WebSocket server on {}", addr);

        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(config): State<Arc<Config>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, config))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, config: Arc<Config>) {
    let (mut sender, mut receiver) = socket.split();
    let subscriptions: SubscriptionManager = Arc::new(DashMap::new());
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Spawn task to forward messages from subscriptions to WebSocket
    let forward_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    // Process incoming WebSocket messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_message(&text, &subscriptions, &tx, &config).await {
                    error!("Error handling WebSocket message: {}", e);
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32603,
                            "message": format!("Internal error: {}", e)
                        },
                        "id": null
                    });
                    if let Ok(error_msg) = serde_json::to_string(&error_response) {
                        let _ = tx.send(Message::Text(error_msg));
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup: cancel all subscriptions
    cleanup_subscriptions(&subscriptions).await;
    forward_task.abort();
}

/// Handle incoming JSON-RPC message
async fn handle_message(
    text: &str,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request: Value = serde_json::from_str(text)?;
    
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .ok_or("Invalid request: missing method")?;
    
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let params = request.get("params").cloned().unwrap_or(Value::Array(vec![]));

    debug!("Handling WebSocket method: {}", method);

    match method {
        // Subscription methods
        "accountSubscribe" => handle_account_subscribe(params, id, subscriptions, tx, config).await?,
        "blockSubscribe" => handle_block_subscribe(params, id, subscriptions, tx, config).await?,
        "logsSubscribe" => handle_logs_subscribe(params, id, subscriptions, tx, config).await?,
        "programSubscribe" => handle_program_subscribe(params, id, subscriptions, tx, config).await?,
        "rootSubscribe" => handle_root_subscribe(params, id, subscriptions, tx, config).await?,
        "signatureSubscribe" => handle_signature_subscribe(params, id, subscriptions, tx, config).await?,
        "slotSubscribe" => handle_slot_subscribe(params, id, subscriptions, tx, config).await?,
        "slotsUpdatesSubscribe" => handle_slots_updates_subscribe(params, id, subscriptions, tx, config).await?,
        "voteSubscribe" => handle_vote_subscribe(params, id, subscriptions, tx, config).await?,

        // Unsubscribe methods
        "accountUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "blockUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "logsUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "programUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "rootUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "signatureUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "slotUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "slotsUpdatesUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,
        "voteUnsubscribe" => handle_unsubscribe(params, id, subscriptions, tx).await?,

        _ => {
            let error_response = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": format!("Unknown method: {}", method)
                },
                "id": id
            });
            let error_msg = serde_json::to_string(&error_response)?;
            tx.send(Message::Text(error_msg))?;
        }
    }

    Ok(())
}

/// Handle account subscription
async fn handle_account_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let params_array = params.as_array().ok_or("Invalid params")?;
    if params_array.is_empty() {
        return Err("Missing account pubkey parameter".into());
    }

    let pubkey_str = params_array[0].as_str().ok_or("Invalid pubkey")?;
    let pubkey: Pubkey = pubkey_str.parse()?;

    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        match pubsub_client.account_subscribe(&pubkey, None).await {
            Ok((mut stream, _unsubscriber)) => {
                while let Some(account_info) = stream.next().await {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "accountNotification",
                        "params": {
                            "result": account_info,
                            "subscription": subscription_id_clone
                        }
                    });
                    
                    if let Ok(msg) = serde_json::to_string(&notification) {
                        if tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create account subscription: {}", e);
            }
        }
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "accountSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

/// Handle block subscription
async fn handle_block_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Parse block subscription filter
    let filter = params.as_array()
        .and_then(|arr| arr.first())
        .unwrap_or(&Value::String("all".to_string()));

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        // Note: Block subscription is unstable and may not be available
        // For now, we'll send a basic response and implement when available
        warn!("Block subscription is unstable and may not be supported on all RPC endpoints");
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "blockSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

/// Handle logs subscription
async fn handle_logs_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Parse logs subscription filter
    let filter = if let Some(params_array) = params.as_array() {
        if let Some(first_param) = params_array.first() {
            if let Some(filter_str) = first_param.as_str() {
                match filter_str {
                    "all" => RpcTransactionLogsFilter::All,
                    "allWithVotes" => RpcTransactionLogsFilter::AllWithVotes,
                    _ => RpcTransactionLogsFilter::All,
                }
            } else if let Some(mentions_obj) = first_param.as_object() {
                if let Some(mentions_array) = mentions_obj.get("mentions")
                    .and_then(|v| v.as_array()) 
                {
                    if let Some(mention_str) = mentions_array.first()
                        .and_then(|v| v.as_str()) 
                    {
                        if let Ok(pubkey) = mention_str.parse::<Pubkey>() {
                            RpcTransactionLogsFilter::Mentions(vec![pubkey.to_string()])
                        } else {
                            RpcTransactionLogsFilter::All
                        }
                    } else {
                        RpcTransactionLogsFilter::All
                    }
                } else {
                    RpcTransactionLogsFilter::All
                }
            } else {
                RpcTransactionLogsFilter::All
            }
        } else {
            RpcTransactionLogsFilter::All
        }
    } else {
        RpcTransactionLogsFilter::All
    };

    let config = RpcTransactionLogsConfig {
        commitment: None,
    };

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        match pubsub_client.logs_subscribe(filter, config).await {
            Ok((mut stream, _unsubscriber)) => {
                while let Some(log_info) = stream.next().await {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "logsNotification",
                        "params": {
                            "result": log_info,
                            "subscription": subscription_id_clone
                        }
                    });
                    
                    if let Ok(msg) = serde_json::to_string(&notification) {
                        if tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create logs subscription: {}", e);
            }
        }
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "logsSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

/// Handle program subscription
async fn handle_program_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let params_array = params.as_array().ok_or("Invalid params")?;
    if params_array.is_empty() {
        return Err("Missing program pubkey parameter".into());
    }

    let pubkey_str = params_array[0].as_str().ok_or("Invalid pubkey")?;
    let pubkey: Pubkey = pubkey_str.parse()?;

    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        match pubsub_client.program_subscribe(&pubkey, None).await {
            Ok((mut stream, _unsubscriber)) => {
                while let Some(program_info) = stream.next().await {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "programNotification",
                        "params": {
                            "result": program_info,
                            "subscription": subscription_id_clone
                        }
                    });
                    
                    if let Ok(msg) = serde_json::to_string(&notification) {
                        if tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create program subscription: {}", e);
            }
        }
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "programSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

/// Handle root subscription
async fn handle_root_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        match pubsub_client.root_subscribe().await {
            Ok((mut stream, _unsubscriber)) => {
                while let Some(root_info) = stream.next().await {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "rootNotification",
                        "params": {
                            "result": root_info,
                            "subscription": subscription_id_clone
                        }
                    });
                    
                    if let Ok(msg) = serde_json::to_string(&notification) {
                        if tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create root subscription: {}", e);
            }
        }
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "rootSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

// Implement remaining subscription handlers...
async fn handle_signature_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let params_array = params.as_array().ok_or("Invalid params")?;
    if params_array.is_empty() {
        return Err("Missing signature parameter".into());
    }

    let signature_str = params_array[0].as_str().ok_or("Invalid signature")?;
    let signature = signature_str.parse().map_err(|e| format!("Invalid signature: {}", e))?;

    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        match pubsub_client.signature_subscribe(&signature, None).await {
            Ok((mut stream, _unsubscriber)) => {
                while let Some(signature_info) = stream.next().await {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "signatureNotification",
                        "params": {
                            "result": signature_info,
                            "subscription": subscription_id_clone
                        }
                    });
                    
                    if let Ok(msg) = serde_json::to_string(&notification) {
                        if tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create signature subscription: {}", e);
            }
        }
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "signatureSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

async fn handle_slot_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Create PubsubClient for this subscription
    let ws_url = config.rpc_url.replace("https://", "wss://").replace("http://", "ws://");
    let pubsub_client = PubsubClient::new(&ws_url).await?;

    // Start the subscription
    let tx_clone = tx.clone();
    let subscription_id_clone = subscription_id;
    tokio::spawn(async move {
        match pubsub_client.slot_subscribe().await {
            Ok((mut stream, _unsubscriber)) => {
                while let Some(slot_info) = stream.next().await {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "slotNotification",
                        "params": {
                            "result": slot_info,
                            "subscription": subscription_id_clone
                        }
                    });
                    
                    if let Ok(msg) = serde_json::to_string(&notification) {
                        if tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create slot subscription: {}", e);
            }
        }
    });

    // Store subscription info
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "slotSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

async fn handle_slots_updates_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Store subscription info (slots updates subscription is unstable)
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "slotsUpdatesSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    warn!("Slots updates subscription is unstable and may not be supported on all RPC endpoints");

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

async fn handle_vote_subscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
    config: &Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscription_id = SUBSCRIPTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    
    // Store subscription info (vote subscription is unstable)
    subscriptions.insert(subscription_id, Subscription {
        id: subscription_id,
        method: "voteSubscribe".to_string(),
        params,
        client_tx: tx.clone(),
    });

    warn!("Vote subscription is unstable and may not be supported on all RPC endpoints");

    // Send success response
    let response = json!({
        "jsonrpc": "2.0",
        "result": subscription_id,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

/// Handle unsubscribe requests
async fn handle_unsubscribe(
    params: Value,
    id: Value,
    subscriptions: &SubscriptionManager,
    tx: &mpsc::UnboundedSender<Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let params_array = params.as_array().ok_or("Invalid params")?;
    if params_array.is_empty() {
        return Err("Missing subscription ID parameter".into());
    }

    let subscription_id = params_array[0].as_u64().ok_or("Invalid subscription ID")?;
    
    let success = subscriptions.remove(&subscription_id).is_some();

    // Send response
    let response = json!({
        "jsonrpc": "2.0",
        "result": success,
        "id": id
    });
    let response_msg = serde_json::to_string(&response)?;
    tx.send(Message::Text(response_msg))?;

    Ok(())
}

/// Cleanup all subscriptions
async fn cleanup_subscriptions(subscriptions: &SubscriptionManager) {
    let count = subscriptions.len();
    subscriptions.clear();
    info!("Cleaned up {} subscriptions", count);
}

/// Start the WebSocket server in a background task
pub fn start_websocket_server_task(port: u16, config: Arc<Config>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let server = SolanaWebSocketServer::new(port, config);
        if let Err(e) = server.start().await {
            error!("WebSocket server failed: {}", e);
        }
    })
}