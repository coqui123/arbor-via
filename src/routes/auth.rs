use axum::{
    extract::{State, Form},
    response::Redirect,
    routing::{get, post},
    Router,
};
use askama::Template;
use askama_axum::IntoResponse;
use serde::Deserialize;
use validator::Validate;
use crate::state::AppState;
use crate::errors::AppError;
use tower_cookies::{Cookies, Cookie};
use time::Duration;

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate {
    error_msg: String,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
struct RegisterTemplate {
    error_msg: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginForm {
    #[validate(email)]
    email: String,
    password: String,
}

#[derive(Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(email)]
    email: String,
    #[validate(length(min = 6))]
    password: String,
}

#[derive(serde::Serialize)]
pub struct AuthResponse {
    success: bool,
    message: String,
}

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(show_login).post(login))
        .route("/register", get(show_register).post(register))
        .route("/logout", post(logout))
}

async fn show_login() -> impl IntoResponse {
    LoginTemplate {
        error_msg: "".to_string(),
    }
}

async fn show_register() -> impl IntoResponse {
    RegisterTemplate {
        error_msg: "".to_string(),
    }
}

async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate form
    if let Err(errors) = form.validate() {
        return Err(AppError::InvalidInput(format!("Validation errors: {:?}", errors)));
    }

    // Attempt login
    let session = state.services.auth.login(&form.email, &form.password).await?;

    // Set auth token cookie
    let mut cookie = Cookie::new("auth_token", session.token);
    cookie.set_http_only(true);
    cookie.set_path("/");
    // In production, consider setting Secure and SameSite to strengthen CSRF defenses
    #[cfg(not(debug_assertions))]
    {
        cookie.set_secure(true);
        // Use time::cookie for SameSite if available in tower_cookies; otherwise skip to avoid breaking build
        #[allow(unused_imports)]
        use tower_cookies::cookie::SameSite;
        #[cfg(any())]
        cookie.set_same_site(SameSite::Strict);
    }
    cookies.add(cookie);

    // In a real app, you'd set a cookie here
    // For now, we'll just redirect to dashboard
    Ok(Redirect::to("/dashboard"))
}

async fn register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate form
    if let Err(errors) = form.validate() {
        return Err(AppError::InvalidInput(format!("Validation errors: {:?}", errors)));
    }

    // Attempt registration
    let _user = state.services.auth.register(&form.email, &form.password).await?;

    // Redirect to login
    Ok(Redirect::to("/login"))
}

async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    if let Some(c) = cookies.get("auth_token") {
        let token = c.value().to_string();
        // best-effort delete on server
        let _ = state.services.auth.logout(&token).await;
        // remove client cookie
        let mut expired = Cookie::from("auth_token");
        expired.set_max_age(Duration::seconds(0));
        expired.set_path("/");
        cookies.add(expired);
    }
    Ok(Redirect::to("/login"))
} 