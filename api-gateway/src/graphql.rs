use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, Object, Schema, SimpleObject,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    middleware::auth::{require_auth_context, AuthContext},
    models::{
        Alert, AlertStatus, AnalyticsEvent, CreateVideoStreamInput, EventSeverity,
        InferenceModel, InferenceResult, PaginatedResponse, PaginationInfo, PaginationInput,
        StreamStatus, StreamSourceType, UpdateVideoStreamInput, User, UserRole, VideoSegment,
        VideoStream,
    },
    AppState,
};

pub type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let auth_context = ctx.data::<AuthContext>()
            .map_err(|_| AppError::Authentication("Authentication required".to_string()))?;
        Ok(auth_context.user.clone())
    }

    async fn video_streams(
        &self,
        ctx: &Context<'_>,
        pagination: Option<PaginationInput>,
    ) -> Result<PaginatedResponse<VideoStream>> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let page = pagination.as_ref().and_then(|p| p.page).unwrap_or(1).max(1);
        let per_page = pagination.as_ref().and_then(|p| p.per_page).unwrap_or(20).min(100).max(1);
        let offset = (page - 1) * per_page;

        // Get total count
        let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM video_streams")
            .fetch_one(state.db.pool())
            .await?;

        // Get streams
        let streams = sqlx::query_as::<_, VideoStream>(
            "SELECT * FROM video_streams ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(state.db.pool())
        .await?;

        let total_pages = (total_count as f64 / per_page as f64).ceil() as i32;

        Ok(PaginatedResponse {
            items: streams,
            pagination: PaginationInfo {
                current_page: page,
                per_page,
                total_pages,
                total_count,
                has_next_page: page < total_pages,
                has_prev_page: page > 1,
            },
        })
    }

    async fn video_stream(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<VideoStream>> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let stream = sqlx::query_as::<_, VideoStream>(
            "SELECT * FROM video_streams WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?;

        Ok(stream)
    }

    async fn video_segments(
        &self,
        ctx: &Context<'_>,
        stream_id: Uuid,
        pagination: Option<PaginationInput>,
    ) -> Result<PaginatedResponse<VideoSegment>> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let page = pagination.as_ref().and_then(|p| p.page).unwrap_or(1).max(1);
        let per_page = pagination.as_ref().and_then(|p| p.per_page).unwrap_or(20).min(100).max(1);
        let offset = (page - 1) * per_page;

        // Get total count
        let total_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM video_segments WHERE stream_id = $1"
        )
        .bind(stream_id)
        .fetch_one(state.db.pool())
        .await?;

        // Get segments
        let segments = sqlx::query_as::<_, VideoSegment>(
            "SELECT * FROM video_segments WHERE stream_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3"
        )
        .bind(stream_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(state.db.pool())
        .await?;

        let total_pages = (total_count as f64 / per_page as f64).ceil() as i32;

        Ok(PaginatedResponse {
            items: segments,
            pagination: PaginationInfo {
                current_page: page,
                per_page,
                total_pages,
                total_count,
                has_next_page: page < total_pages,
                has_prev_page: page > 1,
            },
        })
    }

    async fn inference_results(
        &self,
        ctx: &Context<'_>,
        stream_id: Option<Uuid>,
        pagination: Option<PaginationInput>,
    ) -> Result<PaginatedResponse<InferenceResult>> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let page = pagination.as_ref().and_then(|p| p.page).unwrap_or(1).max(1);
        let per_page = pagination.as_ref().and_then(|p| p.per_page).unwrap_or(50).min(200).max(1);
        let offset = (page - 1) * per_page;

        let (count_query, results_query, bind_stream_id) = if let Some(stream_id) = stream_id {
            (
                "SELECT COUNT(*) FROM inference_results WHERE stream_id = $1",
                "SELECT * FROM inference_results WHERE stream_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
                Some(stream_id),
            )
        } else {
            (
                "SELECT COUNT(*) FROM inference_results",
                "SELECT * FROM inference_results ORDER BY timestamp DESC LIMIT $1 OFFSET $2",
                None,
            )
        };

        // Get total count
        let total_count: i64 = if let Some(stream_id) = bind_stream_id {
            sqlx::query_scalar(count_query)
                .bind(stream_id)
                .fetch_one(state.db.pool())
                .await?
        } else {
            sqlx::query_scalar(count_query)
                .fetch_one(state.db.pool())
                .await?
        };

        // Get results
        let results = if let Some(stream_id) = bind_stream_id {
            sqlx::query_as::<_, InferenceResult>(results_query)
                .bind(stream_id)
                .bind(per_page)
                .bind(offset)
                .fetch_all(state.db.pool())
                .await?
        } else {
            sqlx::query_as::<_, InferenceResult>(results_query)
                .bind(per_page)
                .bind(offset)
                .fetch_all(state.db.pool())
                .await?
        };

        let total_pages = (total_count as f64 / per_page as f64).ceil() as i32;

        Ok(PaginatedResponse {
            items: results,
            pagination: PaginationInfo {
                current_page: page,
                per_page,
                total_pages,
                total_count,
                has_next_page: page < total_pages,
                has_prev_page: page > 1,
            },
        })
    }

    async fn alerts(
        &self,
        ctx: &Context<'_>,
        stream_id: Option<Uuid>,
        status: Option<AlertStatus>,
        pagination: Option<PaginationInput>,
    ) -> Result<PaginatedResponse<Alert>> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let page = pagination.as_ref().and_then(|p| p.page).unwrap_or(1).max(1);
        let per_page = pagination.as_ref().and_then(|p| p.per_page).unwrap_or(20).min(100).max(1);
        let offset = (page - 1) * per_page;

        // Build dynamic query based on filters
        let mut where_conditions = Vec::new();
        let mut bind_values: Vec<&dyn sqlx::Encode<sqlx::Postgres>> = Vec::new();
        let mut param_count = 0;

        if let Some(stream_id) = stream_id {
            param_count += 1;
            where_conditions.push(format!("stream_id = ${}", param_count));
        }

        if let Some(status) = status {
            param_count += 1;
            where_conditions.push(format!("status = ${}", param_count));
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Get total count (simplified for this example)
        let total_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM alerts {}", where_clause
        ))
        .fetch_one(state.db.pool())
        .await?;

        // Get alerts (simplified query)
        let alerts = sqlx::query_as::<_, Alert>(&format!(
            "SELECT * FROM alerts {} ORDER BY triggered_at DESC LIMIT {} OFFSET {}",
            where_clause, per_page, offset
        ))
        .fetch_all(state.db.pool())
        .await?;

        let total_pages = (total_count as f64 / per_page as f64).ceil() as i32;

        Ok(PaginatedResponse {
            items: alerts,
            pagination: PaginationInfo {
                current_page: page,
                per_page,
                total_pages,
                total_count,
                has_next_page: page < total_pages,
                has_prev_page: page > 1,
            },
        })
    }

    async fn inference_models(&self, ctx: &Context<'_>) -> Result<Vec<InferenceModel>> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let models = sqlx::query_as::<_, InferenceModel>(
            "SELECT * FROM inference_models ORDER BY created_at DESC"
        )
        .fetch_all(state.db.pool())
        .await?;

        Ok(models)
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_video_stream(
        &self,
        ctx: &Context<'_>,
        input: CreateVideoStreamInput,
    ) -> Result<VideoStream> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;
        
        let auth_context = ctx.data::<AuthContext>()
            .map_err(|_| AppError::Authentication("Authentication required".to_string()))?;

        let stream_id = Uuid::new_v4();

        let stream = sqlx::query_as::<_, VideoStream>(
            r#"
            INSERT INTO video_streams (id, name, description, source_url, source_type, status, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(stream_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.source_url)
        .bind(input.source_type)
        .bind(StreamStatus::Inactive)
        .bind(auth_context.user.id)
        .fetch_one(state.db.pool())
        .await?;

        tracing::info!("Video stream created: {} by {}", stream.name, auth_context.user.email);

        Ok(stream)
    }

    async fn update_video_stream(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: UpdateVideoStreamInput,
    ) -> Result<VideoStream> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let auth_context = ctx.data::<AuthContext>()
            .map_err(|_| AppError::Authentication("Authentication required".to_string()))?;

        // Check if stream exists and user has permission
        let existing_stream = sqlx::query_as::<_, VideoStream>(
            "SELECT * FROM video_streams WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Video stream not found".to_string()))?;

        // Check permission (owner or admin)
        if existing_stream.created_by != Some(auth_context.user.id) 
            && auth_context.user.role != UserRole::Admin {
            return Err(AppError::Authorization("Permission denied".to_string()));
        }

        // Build update query dynamically
        let mut set_clauses = vec!["updated_at = NOW()"];
        let mut bind_count = 1;

        if input.name.is_some() {
            bind_count += 1;
            set_clauses.push(&format!("name = ${}", bind_count));
        }
        if input.description.is_some() {
            bind_count += 1;
            set_clauses.push(&format!("description = ${}", bind_count));
        }
        if input.source_url.is_some() {
            bind_count += 1;
            set_clauses.push(&format!("source_url = ${}", bind_count));
        }
        if input.status.is_some() {
            bind_count += 1;
            set_clauses.push(&format!("status = ${}", bind_count));
        }

        let query = format!(
            "UPDATE video_streams SET {} WHERE id = $1 RETURNING *",
            set_clauses.join(", ")
        );

        // This is a simplified version - in a real implementation you'd use a query builder
        let stream = sqlx::query_as::<_, VideoStream>(&query)
            .bind(id)
            .fetch_one(state.db.pool())
            .await?;

        Ok(stream)
    }

    async fn delete_video_stream(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let auth_context = ctx.data::<AuthContext>()
            .map_err(|_| AppError::Authentication("Authentication required".to_string()))?;

        // Check if stream exists and user has permission
        let existing_stream = sqlx::query_as::<_, VideoStream>(
            "SELECT * FROM video_streams WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Video stream not found".to_string()))?;

        // Check permission (owner or admin)
        if existing_stream.created_by != Some(auth_context.user.id) 
            && auth_context.user.role != UserRole::Admin {
            return Err(AppError::Authorization("Permission denied".to_string()));
        }

        let result = sqlx::query("DELETE FROM video_streams WHERE id = $1")
            .bind(id)
            .execute(state.db.pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn acknowledge_alert(&self, ctx: &Context<'_>, id: Uuid) -> Result<Alert> {
        let state = ctx.data::<AppState>()
            .map_err(|_| AppError::Internal("Failed to get app state".to_string()))?;

        let auth_context = ctx.data::<AuthContext>()
            .map_err(|_| AppError::Authentication("Authentication required".to_string()))?;

        let alert = sqlx::query_as::<_, Alert>(
            r#"
            UPDATE alerts 
            SET status = $1, acknowledged_at = NOW(), acknowledged_by = $2 
            WHERE id = $3 
            RETURNING *
            "#,
        )
        .bind(AlertStatus::Acknowledged)
        .bind(auth_context.user.id)
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Alert not found".to_string()))?;

        Ok(alert)
    }
}

pub async fn create_schema(state: AppState) -> Result<Schema> {
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(state)
        .finish();

    Ok(schema)
}

pub async fn graphql_handler(
    State((state, schema)): State<(AppState, Schema)>,
    req: async_graphql_axum::GraphQLRequest,
) -> Result<impl IntoResponse> {
    let response = schema.execute(req.into_inner()).await;
    Ok(async_graphql_axum::GraphQLResponse::from(response))
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
} 