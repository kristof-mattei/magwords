use axum::Router;

use crate::state::ApplicationState;

pub(crate) fn build_api_router(state: ApplicationState) -> Router {
    Router::new().with_state(state)
}
