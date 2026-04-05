mod build_env;
mod cli;
mod router;
mod routes;
mod server;
mod signal_handlers;
mod span;
mod state;
mod states;
mod tasks;
mod utils;
mod words;

use std::env::{self, VarError};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser as _;
use color_eyre::config::HookBuilder;
use color_eyre::eyre;
use states::config::Config;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{Level, event};
use tracing_subscriber::Layer as _;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use words::ServerMessage;

use crate::build_env::get_build_env;
use crate::cli::Cli;
use crate::router::build_router;
use crate::server::setup_server;
use crate::state::ApplicationState;
use crate::states::config::FridgeDimensions;
use crate::utils::flatten_handle;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[expect(clippy::unnecessary_wraps, reason = "We will expand this later")]
fn build_configs(args: &Cli) -> Result<Config, eyre::Report> {
    let config = Config {
        bind_to: SocketAddr::from(([0, 0, 0, 0], 3000)),
        fridge_dimensions: FridgeDimensions {
            fridge_width: args.fridge_width,
            fridge_height: args.fridge_height,
        },
    };

    Ok(config)
}

fn print_header() {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let build_env = get_build_env();

    event!(
        Level::INFO,
        "{} v{} - built for {} ({})",
        NAME,
        VERSION,
        build_env.get_target(),
        build_env.get_target_cpu().unwrap_or("base cpu variant"),
    );
}

/// Starts all the tasks, such as the web server, the key refresh, and
/// ensures all tasks are gracefully shutdown in case of error, ctrl-c or `SIGTERM`.
async fn start_tasks() -> Result<(), eyre::Report> {
    let args = cli::Cli::parse();

    print_header();
    args.print();

    let config = build_configs(&args)?;

    // this channel is used to communicate between
    // tasks and this function, in the case that a task fails, they'll send a message on the shutdown channel
    // after which we'll gracefully terminate other services
    let token = CancellationToken::new();

    let ws_state = {
        let words = include_str!("../../../assets/word-list-all.txt");
        words::build_ws_state(words, config.fridge_dimensions)
    };

    let application_state = ApplicationState::new(config, Arc::clone(&ws_state));

    let tasks = TaskTracker::new();

    {
        let token = token.clone();

        let bind_to = application_state.config.bind_to;
        let router = build_router(application_state);

        tasks.spawn(async move {
            let _guard = token.clone().drop_guard();

            let server = setup_server(bind_to, router, token.clone()).await;

            match server {
                Ok(()) => {
                    event!(Level::INFO, "Server shutting down gracefully");
                },
                Err(error) => {
                    event!(Level::ERROR, ?error, "Server shutting down");
                },
            }
        });
    }

    {
        let token = token.clone();
        let ws_state = Arc::clone(&ws_state);

        tasks.spawn(async move {
            let _guard = token.clone().drop_guard();

            while !token.is_cancelled() {
                sleep(Duration::from_millis(1000)).await;

                ws_state.broadcast(None, ServerMessage::Hup { id: 1, v: 1 });
            }

            ws_state.broadcast(None, ServerMessage::Goodbye {});
        });
    };

    // now we wait forever for either
    // * SIGTERM
    // * ctrl + c (SIGINT)
    // * a message on the shutdown channel, sent either by the server task or
    // another task when they complete (which means they failed)
    tokio::select! {
        result = signal_handlers::wait_for_sigterm() => {
            if let Err(error) = result {
                event!(Level::ERROR, ?error, "Failed to register SIGERM handler, aborting");
            } else {
                // we completed because ...
                event!(Level::WARN, "Sigterm detected, stopping all tasks");
            }
        },
        result = signal_handlers::wait_for_sigint() => {
            if let Err(error) = result {
                event!(Level::ERROR, ?error, "Failed to register CTRL+C handler, aborting");
            } else {
                // we completed because ...
                event!(Level::WARN, "CTRL+C detected, stopping all tasks");
            }
        },
        () = token.cancelled() => {
            event!(Level::WARN, "Underlying task stopped, stopping all others tasks");
        },
    }

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
        event!(Level::ERROR, "Tasks didn't stop within allotted time!");
    }

    event!(Level::INFO, message = "Goodbye");

    Ok(())
}

fn build_default_filter() -> EnvFilter {
    EnvFilter::builder()
        .parse(format!(
            "DEBUG,{}=TRACE,tower_http::trace=TRACE",
            env!("CARGO_CRATE_NAME")
        ))
        .expect("Default filter should always work")
}

fn init_tracing() -> Result<(), eyre::Report> {
    let (filter, filter_parsing_error) = match env::var(EnvFilter::DEFAULT_ENV) {
        Ok(user_directive) => match EnvFilter::builder().parse(user_directive) {
            Ok(filter) => (filter, None),
            Err(error) => (build_default_filter(), Some(eyre::Report::new(error))),
        },
        Err(VarError::NotPresent) => (build_default_filter(), None),
        Err(error @ VarError::NotUnicode(_)) => {
            (build_default_filter(), Some(eyre::Report::new(error)))
        },
    };

    let registry = tracing_subscriber::registry();

    #[cfg(feature = "tokio-console")]
    let registry = registry.with(console_subscriber::ConsoleLayer::builder().spawn());

    registry
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .with(tracing_error::ErrorLayer::default())
        .try_init()?;

    filter_parsing_error.map_or(Ok(()), Err)
}

fn main() -> Result<(), eyre::Report> {
    HookBuilder::default()
        .capture_span_trace_by_default(true)
        .display_env_section(false)
        .install()?;

    init_tracing()?;

    // initialize the runtime
    let result: Result<(), eyre::Report> = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async {
            // explicitly launch everything in a spawned task
            // see https://docs.rs/tokio/latest/tokio/attr.main.html#non-worker-async-function
            let handle = tokio::task::spawn(start_tasks());

            flatten_handle(handle).await
        });

    result
}
