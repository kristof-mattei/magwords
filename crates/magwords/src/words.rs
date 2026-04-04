use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::time::Duration;

use axum::extract::State;
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use rand::RngExt as _;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use tokio::time::Instant;
use tracing::{Level, event};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MoveEventParams {
    id: usize,
    v: usize,
    x: u32,
    y: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WordInfo {
    id: usize,
    word: String,
    x: u32,
    y: u32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum ServerMessage {
    Poets { count: usize },
    Move(MoveEventParams),
    Hup { id: u64, v: u64 },
    Goodbye {},
}

// separate struct to serialize the word list without cloning
#[derive(Serialize)]
struct WireWords<'a> {
    r#type: &'static str,
    data: &'a [WordInfo],
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
enum ClientMessage {
    Move(MoveEventParams),
    Pong {
        #[expect(
            dead_code,
            reason = "deserialized from wire protocol but not read server-side"
        )]
        id: u64,
    },
}

pub struct WsState {
    broadcast_tx: broadcast::Sender<(Option<u64>, ServerMessage)>,
    word_list: RwLock<Vec<WordInfo>>,
    poets: AtomicUsize,
    next_client_id: AtomicU64,
}

impl WsState {
    pub fn broadcast(&self, exclude: Option<u64>, message: ServerMessage) {
        let _r = self.broadcast_tx.send((exclude, message));
    }
}

pub fn build_ws_state(raw_words: &str) -> Arc<WsState> {
    let word_list = build_words(raw_words);
    let (broadcast_tx, _) = broadcast::channel(256);

    Arc::new(WsState {
        broadcast_tx,
        word_list: RwLock::new(word_list),
        poets: AtomicUsize::new(0),
        next_client_id: AtomicU64::new(0),
    })
}

fn build_words(words: &str) -> Vec<WordInfo> {
    let mut rng = rand::rng();

    words
        .lines()
        .enumerate()
        .map(|(i, line)| WordInfo {
            id: i,
            word: line.into(),
            x: rng.random_range(0..1000),
            y: rng.random_range(0..1000),
        })
        .collect::<Vec<_>>()
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(ws_state): State<Arc<WsState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, ws_state, address))
}

// max time to wait for a pong before considering the client stale
const PONG_TIMEOUT: Duration = Duration::from_secs(5);

async fn handle_outbound(
    result: Result<(Option<u64>, ServerMessage), broadcast::error::RecvError>,
    client_id: u64,
    address: SocketAddr,
    socket: &mut WebSocket,
    last_pong: Instant,
) -> ControlFlow<()> {
    match result {
        Ok((exclude, message)) => {
            if exclude == Some(client_id) {
                return ControlFlow::Continue(());
            }

            // on hup, check if the client responded to a previous heartbeat
            if matches!(message, ServerMessage::Hup { .. }) && last_pong.elapsed() > PONG_TIMEOUT {
                event!(Level::TRACE, client_id, %address, "client timed out");
                return ControlFlow::Break(());
            }

            let is_goodbye = matches!(message, ServerMessage::Goodbye { .. });

            let json = match serde_json::to_string(&message) {
                Ok(json) => json,
                Err(error) => {
                    event!(Level::ERROR, ?error, ?message, client_id, %address, "failed to serialize message, this is a bug");

                    return ControlFlow::Break(());
                },
            };

            if let Err(error) = socket.send(Message::text(json)).await {
                event!(Level::TRACE, ?error, client_id, %address, "failed to send message");
                return ControlFlow::Break(());
            }

            if is_goodbye {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
        Err(broadcast::error::RecvError::Lagged(count)) => {
            event!(
                Level::TRACE,
                client_id,
                %address,
                count,
                "client lagged, disconnecting"
            );
            ControlFlow::Break(())
        },
        Err(broadcast::error::RecvError::Closed) => ControlFlow::Break(()),
    }
}

async fn handle_inbound(
    result: Option<Result<Message, axum::Error>>,
    client_id: u64,
    address: SocketAddr,
    state: &WsState,
    last_pong: &mut Instant,
) -> ControlFlow<()> {
    match result {
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Move(move_event)) => {
                    let mut lock = state.word_list.write().await;

                    if let Some(word) = lock.get_mut(move_event.id) {
                        word.x = move_event.x;
                        word.y = move_event.y;

                        state.broadcast(Some(client_id), ServerMessage::Move(move_event));
                    } else {
                        event!(Level::WARN, client_id, %address, id = move_event.id, "invalid word id, disconnecting");
                        return ControlFlow::Break(());
                    }
                },
                Ok(ClientMessage::Pong { .. }) => {
                    *last_pong = Instant::now();
                },
                Err(error) => {
                    event!(Level::TRACE, ?error, client_id, %address, "invalid message received");
                },
            }

            ControlFlow::Continue(())
        },
        Some(Ok(Message::Close(_))) | None => ControlFlow::Break(()),
        Some(Err(error)) => {
            event!(Level::TRACE, ?error, client_id, %address, "websocket read error");
            ControlFlow::Break(())
        },
        Some(Ok(_)) => ControlFlow::Continue(()),
    }
}

async fn handle_socket(mut socket: WebSocket, state: Arc<WsState>, address: SocketAddr) {
    let client_id = state
        .next_client_id
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    event!(Level::DEBUG, client_id, %address, "Client connected");

    // send initial word list to this client
    {
        let json = {
            let words = state.word_list.read().await;
            serde_json::to_string(&WireWords {
                r#type: "words",
                data: &words,
            })
        };

        if let Ok(json) = json {
            if let Err(error) = socket.send(Message::text(json)).await {
                event!(Level::TRACE, ?error, client_id, %address, "failed to send words");
                return;
            }
        }
    }

    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // increment poets and broadcast to all
    {
        let new_count = state
            .poets
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        state.broadcast(None, ServerMessage::Poets { count: new_count });
    }
    let mut last_pong = Instant::now();

    loop {
        let flow = tokio::select! {
            result = broadcast_rx.recv() => handle_outbound(result, client_id, address, &mut socket, last_pong).await,
            result = socket.recv() => handle_inbound(result, client_id, address, &state, &mut last_pong).await,
        };

        if flow.is_break() {
            break;
        }
    }

    // client disconnected, clean up
    let new_count = state
        .poets
        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
        - 1;
    state.broadcast(None, ServerMessage::Poets { count: new_count });

    event!(Level::TRACE, client_id, %address, "Client disconnected");
}
