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

use color_eyre::eyre;
use futures_util::StreamExt;
use serde_json::json;
use socketioxide::SocketIo;
use states::config::Config;
use tokio::signal;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{Level, event};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use words::WordsSocket;

use crate::router::build_router;
use crate::server::setup_server;
use crate::state::ApplicationState;

#[expect(clippy::unnecessary_wraps)]
fn build_configs() -> Result<Config, eyre::Report> {
    let config = Config {
        bind_to: SocketAddr::from(([0, 0, 0, 0], 3000)),
    };

    Ok(config)
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

    let (layer, socket_io) = SocketIo::new_layer();

    let word_socket = {
        let words = include_str!("../../assets/word-list-all.txt");

        WordsSocket::build(socket_io, words).await
    };

    let tasks = TaskTracker::new();

    {
        let token = token.clone();

        let bind_to = application_state.config.bind_to;
        let router = build_router(application_state, layer);

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

                let r = word_socket
                    .get_socket()
                    .emit_with_ack::<_, serde_json::Value>("hup", &json!({ "id": 1, "v": 1 }))
                    .await;

                match r {
                    Ok(act) => {
                        act.for_each_concurrent(Some(10), |(_sid, _ack)| async move {})
                            .await;
                    },
                    Err(e) => {
                        event!(Level::ERROR, ?e, "Failed to broadcast");
                        break;
                    },
                }
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
        r = utils::wait_for_sigterm() => {
            if let Err(e) = r {
                event!(Level::ERROR, message = "Failed to register SIGERM handler, aborting", ?e);
            } else {
                // we completed because ...
                event!(Level::WARN, message = "Sigterm detected, stopping all tasks");
            }
        },
        r = signal::ctrl_c() => {
            if let Err(e) = r {
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

    // announce cancel
    token.cancel();

    // close the tracker, otherwise wait doesn't work
    tasks.close();

    // wait for the task that holds the server to exit gracefully
    // it listens to shutdown_send
    if timeout(Duration::from_millis(10000), tasks.wait())
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

fn init_tracing() -> Result<(), eyre::Report> {
    let main_filter = EnvFilter::builder()
        .parse(env::var(EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| {
            format!("INFO,{}=TRACE", env!("CARGO_PKG_NAME").replace('-', "_"))
        }))?;

    let layers = vec![
        #[cfg(feature = "tokio-console")]
        console_subscriber::ConsoleLayer::builder()
            .with_default_env()
            .spawn()
            .boxed(),
        tracing_subscriber::fmt::layer()
            .with_filter(main_filter)
            .boxed(),
        tracing_error::ErrorLayer::default().boxed(),
    ];

    Ok(tracing_subscriber::registry().with(layers).try_init()?)
}

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::config::HookBuilder::default().install()?;

    init_tracing()?;

    // initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // start service
    let result: Result<(), color_eyre::Report> = rt.block_on(start_tasks());

    result
}
