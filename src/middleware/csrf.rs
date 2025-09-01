use axum::{
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};

// Deprecated variant kept for reference; not used in router wiring
// pub async fn csrf_middleware(cookies: Cookies, req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> { ... }

// Axum 0.7-friendly CSRF verifier for use with `axum::middleware::from_fn`
pub async fn csrf_verify(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = req.method();

    // Parse csrf_token from Cookie header if present
    let cookie_header = req.headers().get(header::COOKIE).and_then(|h| h.to_str().ok());
    let mut cookie_token: Option<String> = None;
    if let Some(cookies) = cookie_header {
        for part in cookies.split(';') {
            let trimmed = part.trim();
            if let Some(rest) = trimmed.strip_prefix("csrf_token=") {
                cookie_token = Some(rest.to_string());
                break;
            }
        }
    }

    // Safe methods: ensure token cookie exists
    if method.is_safe() {
        let mut res = next.run(req).await;
        if cookie_token.is_none() {
            let token = uuid::Uuid::new_v4().to_string();
            let set_cookie = format!("csrf_token={}; Path=/; SameSite=Lax", token);
            res.headers_mut()
                .append(header::SET_COOKIE, HeaderValue::from_str(&set_cookie).expect("Valid cookie string should be convertible to HeaderValue"));
        }
        return Ok(res);
    }

    // Unsafe methods: require header token and match cookie token
    let header_token = req
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if cookie_token.as_deref() != Some(header_token) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(req).await)
}

// Note: custom Header implementation removed; we directly read string headers
