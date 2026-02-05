use axum::Router;

use crate::state::ApplicationState;

pub fn build_api_router(state: ApplicationState) -> Router {
    Router::new().with_state(state)
}
