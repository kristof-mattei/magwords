use axum::Router;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
mod api_router;
mod html_router;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
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

// #[expect(clippy::unused_async)]
// async fn root() -> impl IntoResponse {
//     (
//         StatusCode::OK,
//         "These are not the droids you're looking for!",
//     )
// }

pub(crate) fn build_router(state: ApplicationState) -> Router {
    let api_router = build_api_router(state);

    let html_router = build_html_router();

    api_router
        .merge(html_router)
        .route("/healthz", get(healthz))
        .fallback_service(handler_404.into_service())
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}
