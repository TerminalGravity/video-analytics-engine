use axum::{
    extract::State,
    http::{HeaderValue, Method},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber;
use serde_json::json;

mod config;
mod database;
mod auth;
mod graphql;
mod error;
mod middleware;
mod models;
mod services;

use config::Config;
use database::Database;
use error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Video Analytics API Gateway");

    // Load configuration
    let config = Config::load().await?;
    tracing::info!("Configuration loaded");

    // Initialize database
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Run migrations
    db.migrate().await?;
    tracing::info!("Database migrations completed");

    // Create application state
    let state = AppState {
        db,
        config: config.clone(),
    };

    // Build our application with routes
    let app = create_app(state).await?;

    // Start server
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", config.port)).await?;
    tracing::info!("Server listening on port {}", config.port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_app(state: AppState) -> Result<Router, AppError> {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    // Create GraphQL schema
    let schema = graphql::create_schema(state.clone()).await?;

    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        
        // GraphQL endpoint
        .route("/graphql", post(graphql::graphql_handler).get(graphql::graphql_playground))
        
        // Authentication endpoints
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/refresh", post(auth::refresh_token))
        
        // WebSocket endpoint for real-time updates
        .route("/ws", get(websocket_handler))
        
        // Add GraphQL schema to state
        .with_state((state, schema))
        
        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors)
                .layer(middleware::rate_limit::RateLimitLayer::new())
        );

    Ok(app)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "video-analytics-api-gateway",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn websocket_handler(
    ws: axum::extract::WebSocketUpgrade,
    State((state, _)): State<(AppState, graphql::Schema)>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| services::websocket::handle_socket(socket, state))
} 