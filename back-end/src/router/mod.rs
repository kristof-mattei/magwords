#![deny(clippy::unused_async)]

use axum::Router;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
mod api_router;
mod html_router;
use socketioxide::layer::SocketIoLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use self::api_router::build_api_router;
use self::html_router::build_html_router;
use crate::state::ApplicationState;

#[expect(clippy::unused_async)]
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "Hello, world!")
}

pub(crate) fn build_router(state: ApplicationState, websocket_layer: SocketIoLayer) -> Router {
    let api_router = build_api_router(state);
    let html_router = build_html_router();

    html_router
        .nest(
            "/api",
            api_router.fallback_service(handler_404.into_service()),
        )
        .route("/healthz", get(healthz))
        .layer(websocket_layer)
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::DEBUG)),
        )
}
