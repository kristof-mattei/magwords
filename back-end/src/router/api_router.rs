use axum::Router;
use axum::response::IntoResponse;
use axum::routing::method_routing::get;
use http::StatusCode;
use tracing::{Level, event, instrument};

use crate::state::ApplicationState;

pub fn build_api_router(state: ApplicationState) -> Router {
    Router::new()
        .route("/resource", get(resource))
        .with_state(state)
}

async fn resource() -> impl IntoResponse {
    expensive_call().await;

    (StatusCode::OK, "Resource")
}

#[instrument]
async fn expensive_call() {
    event!(Level::DEBUG, "DEBUG");
}
