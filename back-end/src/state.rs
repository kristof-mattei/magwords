use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::states::config::Config;

impl FromRef<ApplicationState> for Arc<Config> {
    fn from_ref(input: &ApplicationState) -> Self {
        input.config.clone()
    }
}

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct ApplicationState {
    config: Arc<Config>,
}

impl ApplicationState {
    pub fn new(config: Config) -> Self {
        ApplicationState {
            config: Arc::new(config),
        }
    }
}

impl<S> FromRequestParts<S> for ApplicationState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    // TODO State not found error
    type Rejection = ();

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}
