mod api_router;
mod html_router;

use axum::Router;
use axum::handler::HandlerWithoutStateExt as _;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::router::api_router::build_api_router;
use crate::router::html_router::build_html_router;
use crate::state::ApplicationState;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "Hello, world!")
}

pub fn build_router(state: ApplicationState) -> Router {
    let api_router = build_api_router(state);
    let html_router = build_html_router();

    // we can move the `/healthz` layer beneath the `TraceLayer` to prevent it from being logged

    html_router
        .nest(
            "/api",
            api_router.fallback_service(handler_404.into_service()),
        )
        .route("/healthz", get(healthz))
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}
