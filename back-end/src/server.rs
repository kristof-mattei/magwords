use std::net::SocketAddr;

use axum::Router;
use color_eyre::eyre::{self, Context};
use tokio_util::sync::CancellationToken;
use tracing::{Level, event};

/// Set up server on socket, with a router, and a cancellation token for graceful shutdown
///
/// # Errors
/// * Couldn't bind to address
/// * Server failure
pub(crate) async fn setup_server(
    bind_to: SocketAddr,
    router: Router,
    token: CancellationToken,
) -> Result<(), eyre::Report> {
    event!(Level::INFO, ?bind_to, "Trying to bind");

    let listener = tokio::net::TcpListener::bind(bind_to)
        .await
        .wrap_err("Failed to bind Webserver to port")?;

    event!(Level::INFO, ?bind_to, "Webserver bound successfully");

    axum::serve(listener, router)
        .with_graceful_shutdown(token.cancelled_owned())
        .await
        .map_err(Into::into)
}
