use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{Level, event};

pub fn build_html_router() -> Router {
    if let Ok(front_end_proxy) = std::env::var("FRONT_END_PROXY").as_deref() {
        event!(Level::INFO, "Serving website via proxy");

        let vite_proxy_service_builder = tower_proxy::builder_http(front_end_proxy).unwrap();

        let svc: tower_proxy::ReusedService<
            tower_proxy::Identity,
            tower_proxy::client::HttpConnector,
            axum::body::Body,
        > = vite_proxy_service_builder.build(tower_proxy::rewrite::Identity {});

        Router::new().fallback_service(svc)
    } else {
        event!(Level::INFO, "Serving website from dist");
        Router::new()
            .fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))
    }
}
