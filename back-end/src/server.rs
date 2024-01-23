use std::convert::Into;
use std::future::IntoFuture;
use std::net::SocketAddr;

use axum::Router;
use color_eyre::eyre::Context;
use tokio_util::sync::CancellationToken;
use tracing::{event, Level};

pub(crate) async fn setup_server(
    bind_to: SocketAddr,
    router: Router,
    token: CancellationToken,
) -> Result<(), color_eyre::Report> {
    event!(Level::INFO, ?bind_to, "Trying to bind");

    let listener = tokio::net::TcpListener::bind(bind_to)
        .await
        .wrap_err("Failed to bind server to port")?;

    event!(Level::INFO, ?bind_to, "Server bound successfully");

    tokio::select! {
        r = axum::serve(listener, router).into_future() => { Ok(r.map_err(Into::<std::io::Error>::into)?) }
        () = token.cancelled() => { Ok(()) }
    }
}
