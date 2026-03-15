use axum::{Router, extract::DefaultBodyLimit, http::header, middleware};
use tower_governor::GovernorLayer;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

use crate::{AppState, handlers, middleware::logging, middleware::rate_limit::RateLimiters};

pub fn router(state: AppState) -> Router {
    let limiters = RateLimiters::new();
    limiters.spawn_cleanup();

    let auth_routes = handlers::auth::router().layer(GovernorLayer::new(limiters.auth.clone()));

    let routes = Router::new()
        .merge(handlers::health::router())
        .nest("/auth", auth_routes)
        .nest("/challenge", handlers::challenge::router())
        .nest("/leaderboard", handlers::leaderboard::router());

    Router::new()
        .nest("/api/v1", routes)
        // ── Observability (outermost → innermost) ───────────────────
        .layer(logging::sensitive_headers_layer())
        .layer(logging::trace_layer())
        .layer(middleware::from_fn(logging::request_id))
        .layer(middleware::from_fn(logging::logging))
        // ── Rate limiting ───────────────────────────────────────────
        .layer(GovernorLayer::new(limiters.global.clone()))
        // ── Security / limits ───────────────────────────────────────
        .layer(DefaultBodyLimit::max(1024 * 1024))
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
        .fallback_service(
            ServeDir::new("static").not_found_service(ServeFile::new("static/index.html")),
        )
        .with_state(state)
}
