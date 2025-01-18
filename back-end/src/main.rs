mod router;
mod routes;
mod server;
mod state;
mod states;
mod tasks;
mod utils;
mod words;

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;
use socketioxide::SocketIo;
use states::config::Config;
use tokio::signal;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;
use tracing::{event, Level};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::router::build_router;
use crate::server::setup_server;
use crate::state::ApplicationState;
use crate::words::setup_socket;

#[allow(clippy::unnecessary_wraps)]
fn build_configs() -> Result<Config, color_eyre::eyre::Report> {
    let config = Config {};

    Ok(config)
}

#[derive(Debug, Deserialize, Serialize)]
struct MoveEventParams {
    id: usize,
    v: usize,
    x: u32,
    y: u32,
}

/// starts all the tasks, such as the web server, the key refresh, ...
/// ensures all tasks are gracefully shutdown in case of error, ctrl+c or sigterm
async fn start_tasks() -> Result<(), color_eyre::Report> {
    let config = build_configs()?;

    // this channel is used to communicate between
    // tasks and this function, in the case that a task fails, they'll send a message on the shutdown channel
    // after which we'll gracefully terminate other services
    let token = CancellationToken::new();

    let application_state = ApplicationState::new(config);

    let (layer, io) = SocketIo::new_layer();

    let router = build_router(application_state, layer);

    let bind_to = SocketAddr::from(([0, 0, 0, 0], 3000));

    let word_socket = {
        let words = include_str!("../../assets/word-list.txt");

        setup_socket(words, io).await
    };

    let mut tasks = JoinSet::new();

    {
        let token = token.clone();

        tasks.spawn(async move {
            let _guard = token.clone().drop_guard();

            let server = setup_server(bind_to, router, token.clone()).await;

            match server {
                Err(e) => {
                    event!(Level::ERROR, message = "Server shutting down", ?e);
                },
                Ok(()) => {
                    event!(Level::INFO, "Server shutting down gracefully");
                },
            }
        });
    }

    {
        let token = token.clone();

        tasks.spawn(async move {
            let _guard = token.clone().drop_guard();

            while !token.is_cancelled() {
                sleep(Duration::from_millis(1000)).await;

                if let Err(e) = word_socket
                    .get_socket()
                    .emit("hup", &json!({ "id":1, "v":1 }))
                    .await
                {
                    event!(Level::ERROR, ?e, "Failed to broadcast");
                    break;
                }
                // let r = word_socket
                //     .get_socket()
                //     .emit_with_ack::<serde_json::Value>("hup", json!({ "id":1, "v":1 }));
                // match r {
                //     Ok(o) => {
                //         o.for_each(|(sid, ack)| async move {
                //             event!(Level::INFO, ?sid, ?ack, "Ack!");
                //         })
                //         .await;
                //     },
                //     Err(e) => {
                //         event!(Level::ERROR, ?e, "Failed to broadcast");
                //         break;
                //     },
                // }
            }

            drop(word_socket);
        });
    };

    // now we wait forever for either
    // * sigterm
    // * ctrl + c
    // * a message on the shutdown channel, sent either by the server task or
    // another task when they complete (which means they failed)
    tokio::select! {
        h = utils::wait_for_sigterm() => {
            if let Err(e) = h {
                event!(Level::ERROR, message = "Failed to register SIGERM handler, aborting", ?e);
            } else {
                // we completed because ...
                event!(Level::WARN, message = "Sigterm detected, stopping all tasks");
            }
        },
        h = signal::ctrl_c() => {
            if let Err(e) = h {
                event!(Level::ERROR, message = "Failed to register CTRL+C handler, aborting", ?e);
            } else {
                // we completed because ...
                event!(Level::WARN, message = "CTRL+C detected, stopping all tasks");
            }
        },
        () = token.cancelled() => {
            event!(Level::ERROR, message = "Underlying task stopped, stopping all others tasks");
        },
    };

    token.cancel();

    // wait for the task that holds the server to exit gracefully
    // it listens to shutdown_send
    if timeout(Duration::from_millis(10000), tasks.shutdown())
        .await
        .is_err()
    {
        event!(
            Level::ERROR,
            message = "Tasks didn't stop within allotted time!"
        );
    }

    event!(Level::INFO, message = "Goodbye");

    Ok(())
}

fn main() -> Result<(), color_eyre::Report> {
    // set up .env
    // dotenv().expect(".env file not found");

    color_eyre::config::HookBuilder::default()
        .capture_span_trace_by_default(false)
        .install()?;

    let rust_log_value = env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| format!("INFO,{}=TRACE", env!("CARGO_PKG_NAME").replace('-', "_")));

    // set up logger
    // from_env defaults to RUST_LOG
    tracing_subscriber::registry()
        .with(EnvFilter::builder().parse(rust_log_value).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default())
        .init();

    // initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // start service
    let result: Result<(), color_eyre::Report> = rt.block_on(start_tasks());

    result
}
