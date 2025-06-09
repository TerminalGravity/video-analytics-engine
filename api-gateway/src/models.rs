use async_graphql::{SimpleObject, InputObject, Enum};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// User models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[graphql(skip)]
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum UserRole {
    Admin,
    User,
    Viewer,
}

#[derive(Debug, Deserialize, Validate, InputObject)]
pub struct CreateUserInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize, Validate, InputObject)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

// Video Stream models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct VideoStream {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub source_type: StreamSourceType,
    pub status: StreamStatus,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum StreamSourceType {
    Rtmp,
    Webrtc,
    File,
    Camera,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum StreamStatus {
    Active,
    Inactive,
    Error,
}

#[derive(Debug, Deserialize, Validate, InputObject)]
pub struct CreateVideoStreamInput {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub source_type: StreamSourceType,
}

#[derive(Debug, Deserialize, Validate, InputObject)]
pub struct UpdateVideoStreamInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub status: Option<StreamStatus>,
}

// Video Segment models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct VideoSegment {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub file_path: String,
    pub duration_seconds: f32,
    pub size_bytes: i64,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Inference models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct InferenceModel {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub model_type: String,
    pub file_path: String,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct InferenceResult {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub model_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub frame_number: i64,
    pub confidence: f32,
    pub bounding_box: Option<serde_json::Value>,
    pub detected_class: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// Analytics models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub severity: EventSeverity,
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

// Alert models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Alert {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub alert_type: String,
    pub severity: EventSeverity,
    pub status: AlertStatus,
    pub metadata: Option<serde_json::Value>,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum AlertStatus {
    Open,
    Acknowledged,
    Closed,
}

// Authentication models
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}

// Session models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// API Key models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[graphql(skip)]
    pub key_hash: String,
    pub permissions: serde_json::Value,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Pagination
#[derive(Debug, InputObject)]
pub struct PaginationInput {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, SimpleObject)]
pub struct PaginationInfo {
    pub current_page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub total_count: i64,
    pub has_next_page: bool,
    pub has_prev_page: bool,
}

// Generic paginated response
#[derive(Debug, SimpleObject)]
#[graphql(concrete(name = "PaginatedUsers", params(User)))]
#[graphql(concrete(name = "PaginatedVideoStreams", params(VideoStream)))]
#[graphql(concrete(name = "PaginatedVideoSegments", params(VideoSegment)))]
#[graphql(concrete(name = "PaginatedInferenceResults", params(InferenceResult)))]
#[graphql(concrete(name = "PaginatedAlerts", params(Alert)))]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationInfo,
} 