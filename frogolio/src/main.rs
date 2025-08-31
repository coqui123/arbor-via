use axum::{
    routing::get,
    Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod errors;
mod routes;
mod state;
mod services;
mod repo;
mod middleware;
mod handler;
mod models;

use crate::routes::frogol::frogol_routes;
use crate::routes::auth::auth_routes;
use crate::routes::dashboard::dashboard_routes;
use crate::routes::avatar::avatar_routes;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Load environment variables from .env if present
    let _ = dotenvy::dotenv();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "frogolio=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set for production");

    let app_state = AppState::new(pool, jwt_secret);

    use tower_http::services::ServeDir;
    use tower_cookies::CookieManagerLayer;
    use crate::middleware::compression::create_compression_layer;

    let app = Router::new()
        .route("/", get(|| async { axum::response::Redirect::to("/login") }))
        .merge(frogol_routes())
        .merge(auth_routes())
        .merge(dashboard_routes())
        .merge(routes::lead::lead_routes())
        .merge(avatar_routes())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state.clone())
        .layer(CookieManagerLayer::new())
        .layer(create_compression_layer());

    // CSRF middleware is available but not globally wired to avoid breaking behavior.
    // HTMX is already configured to include X-CSRF-Token in requests; wire middleware per-route later if needed.

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Frogolio server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Failed to start server");
}
