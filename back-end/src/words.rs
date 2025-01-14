use std::ops::IndexMut;
use std::sync::atomic::AtomicUsize;

use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use socketioxide::extract::{Data, SocketRef, TryData};
use socketioxide::socket::DisconnectReason;
use socketioxide::SocketIo;
use tokio::sync::Mutex;
use tracing::{event, Level};

use crate::MoveEventParams;

static POETS: AtomicUsize = AtomicUsize::new(0);

static WORD_LIST: std::sync::LazyLock<Mutex<Box<[WordInfo]>>> =
    std::sync::LazyLock::new(|| Mutex::new(Box::new([])));

#[derive(Debug, Deserialize, Serialize)]
struct WordInfo {
    id: usize,
    word: String,
    x: u32,
    y: u32,
}

pub(crate) struct WordsSocket {
    io: SocketIo,
}
impl WordsSocket {
    pub(crate) fn get_socket(&self) -> SocketIo {
        self.io.clone()
    }
}

impl Drop for WordsSocket {
    fn drop(&mut self) {
        if let Err(e) = self.io.emit("goodbye", &json!({})) {
            event!(Level::ERROR, ?e, "Failed to announce shutting down");
        }
    }
}

pub async fn setup_socket(raw_words: &str, io: SocketIo) -> WordsSocket {
    let word_list = build_words(raw_words);

    {
        let mut lock = WORD_LIST.lock().await;
        let _old = std::mem::replace(&mut *lock, word_list);
    }

    let socket_clone = io.clone();

    io.ns("/", |socket, data| async move {
        on_connect(socket, data).await;

        if let Err(e) = socket_clone.emit(
            "poets",
            &json!({ "count": POETS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1 }),
        ) {
            event!(Level::ERROR, ?e, "Failed to announce new poet");
        }
    });

    WordsSocket { io }
}

fn build_words(words: &str) -> Box<[WordInfo]> {
    let mut rng = rand::thread_rng();
    words
        .lines()
        .enumerate()
        .map(|(i, l)| WordInfo {
            id: i,
            word: l.into(),
            x: rng.gen_range(0..1000),
            y: rng.gen_range(0..1000),
        })
        .collect::<Vec<_>>()
        .into()
}

#[allow(clippy::needless_pass_by_value)]
async fn on_connect(socket: SocketRef, Data(_data): Data<Value>) {
    event!(
        Level::INFO,
        "Socket.IO connected: {:?} {:?}",
        socket.ns(),
        socket.id
    );

    // register handlers
    socket.on("move", on_move);

    socket.on_disconnect(on_disconnect);

    // register handlers
    // socket.on("ack", on_ack);

    // send client current words
    if let Err(e) = socket.emit("words", &[&*WORD_LIST.lock().await]) {
        event!(Level::TRACE, ?e, "Failed to send words to new client");
    };
}

async fn on_disconnect(socket: SocketRef, reason: DisconnectReason) {
    if let Err(e) = socket.broadcast().emit(
        "poets",
        &json!({ "count": POETS.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1 }),
    ) {
        event!(Level::ERROR, ?e, "Failed to announce poet gone");
    }
    event!(Level::TRACE, ?reason, "Disconnect");
}

async fn on_move(socket: SocketRef, TryData(data): TryData<MoveEventParams>) {
    match data {
        Ok(m) => {
            let mut lock = (WORD_LIST).lock().await;
            let word = lock.index_mut(m.id);

            word.x = m.x;
            word.y = m.y;

            if let Err(e) = socket.broadcast().emit("move", &m) {
                event!(Level::TRACE, ?e, "Failed to broadcast");
            };
        },
        Err(e) => {
            event!(Level::TRACE, ?e, "Invalid move received");
        },
    };
}
