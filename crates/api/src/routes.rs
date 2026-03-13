use axum::{Router, extract::DefaultBodyLimit, http::header};
use tower_http::{set_header::SetResponseHeaderLayer, trace::TraceLayer};

use crate::{AppState, handlers};

pub fn router(state: AppState) -> Router {
    let public = Router::new().merge(handlers::health::router());

    Router::new()
        .nest("/api/v1", Router::new().merge(public))
        // Tracing
        .layer(TraceLayer::new_for_http())
        // Body limit (1 MB)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            header::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            header::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-xss-protection"),
            header::HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("strict-transport-security"),
            header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            header::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("permissions-policy"),
            header::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .with_state(state)
}
