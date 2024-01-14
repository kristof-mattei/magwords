use std::convert::Into;
use std::future::IntoFuture;
use std::net::SocketAddr;

use axum::Router;
use color_eyre::eyre::Context;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{event, Level};

pub(crate) async fn serve_forever(
    bind_to: SocketAddr,
    router: Router,
    token: CancellationToken,
) -> JoinHandle<()> {
    let token = token.clone();

    tokio::spawn(async move {
        let server = set_up_server(bind_to, router, token.clone()).await;
        match &server {
            Err(e) => {
                event!(Level::ERROR, message = "Server shutting down", ?e);
            },
            Ok(()) => {
                event!(Level::INFO, "Server shutting down gracefully");
            },
        }

        // in the case that the server stops signal the rest
        token.cancel();
    })
}

pub(crate) async fn set_up_server(
    bind_to: SocketAddr,
    router: Router,
    token: CancellationToken,
) -> Result<(), color_eyre::Report> {
    let listener = tokio::net::TcpListener::bind(bind_to)
        .await
        .wrap_err("Failed to bind server to port")?;

    event!(Level::INFO, "Server is bound");

    tokio::select! {
        r = axum::serve(listener, router).into_future() => { Ok(r.map_err(Into::<std::io::Error>::into)?) }
        () = token.cancelled() => { Ok(()) }
    }
}
