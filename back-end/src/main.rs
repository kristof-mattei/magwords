mod router;
mod routes;
mod server;
mod state;
mod states;
mod tasks;
mod utils;

use std::net::SocketAddr;
use std::time::Duration;

use states::config::Config;
use tokio::signal;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{event, Level};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::router::build_router;
use crate::server::serve_forever;
use crate::state::ApplicationState;

#[allow(clippy::unnecessary_wraps)]
fn build_configs() -> Result<Config, color_eyre::eyre::Report> {
    let config = Config {};

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

    let router = build_router(application_state);

    let server_task = serve_forever(
        SocketAddr::from(([0, 0, 0, 0], 3000)),
        router,
        token.clone(),
    );

    // now we wait forever for either
    // * sigterm
    // * ctrl + c
    // * a message on the shutdown channel, sent either by the server task or the load_keys task, when they
    // complete (which means they failed)
    tokio::select! {
        _ = utils::wait_for_sigterm() => {
            event!(Level::WARN, message = "Sigterm detected, stopping all tasks");
            token.cancel();
        },
        _ = signal::ctrl_c() => {
            event!(Level::WARN, message = "CTRL+C detected, stopping all tasks");
            token.cancel();
        },
        () = token.cancelled() => {
            event!(Level::ERROR, message = "Underlying task stopped, stopping all others tasks");
        },
    };

    // wait for the task that holds the server to exit gracefully
    // it listens to shutdown_send
    if timeout(Duration::from_millis(10000), server_task)
        .await
        .is_err()
    {
        event!(
            Level::ERROR,
            message = "Server didn't stop within allotted time!"
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

    // set up logger
    // from_env defaults to RUST_LOG
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default())
        .init();

    // initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // start service
    let result: Result<(), color_eyre::Report> = rt.block_on(start_tasks());

    result
}
