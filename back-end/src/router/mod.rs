#![deny(clippy::unused_async)]

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
mod api_router;
mod html_router;
use socketioxide::layer::SocketIoLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use self::api_router::build_api_router;
use self::html_router::build_html_router;
use crate::state::ApplicationState;

// #[allow(clippy::unused_async)]
// async fn handler_404() -> impl IntoResponse {
//     (StatusCode::NOT_FOUND, "nothing to see here")
// }

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "Hello, world!")
}

// #[allow(clippy::unused_async)]
// async fn root() -> impl IntoResponse {
//     (
//         StatusCode::OK,
//         "These are not the droids you're looking for!",
//     )
// }

pub(crate) fn build_router(state: ApplicationState, websocket_layer: SocketIoLayer) -> Router {
    let api_router = build_api_router(state);

    let html_router = build_html_router();

    // .fallback_service(handler_404.into_service())
    api_router
        .merge(html_router)
        .layer(websocket_layer)
        .route("/healthz", get(healthz))
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}
