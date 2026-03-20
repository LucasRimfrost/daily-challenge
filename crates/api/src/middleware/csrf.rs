use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};

/// Rejects state-changing requests (POST, PATCH, PUT, DELETE) that lack
/// the `X-Requested-With` header.
///
/// This is a standard anti-CSRF hardening measure: browsers will not attach
/// custom headers to cross-origin simple requests, so a forged form submission
/// from a malicious site will be blocked even on legacy browsers that do not
/// support `SameSite=Strict`.
pub async fn require_csrf_header(request: Request, next: Next) -> Result<Response, StatusCode> {
    let dominated = matches!(
        *request.method(),
        Method::POST | Method::PATCH | Method::PUT | Method::DELETE
    );

    if dominated && !request.headers().contains_key("x-requested-with") {
        tracing::warn!(
            method = %request.method(),
            uri = %request.uri(),
            "CSRF check failed — missing X-Requested-With header"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
