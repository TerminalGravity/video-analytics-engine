use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{
    clock::{DefaultClock, QuantaClock},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    num::NonZeroU32,
    sync::{Arc, Mutex},
    time::Duration,
};
use tower::{Layer, Service};

use crate::error::AppError;

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;
type IpRateLimiters = Arc<Mutex<HashMap<String, SharedRateLimiter>>>;

#[derive(Clone)]
pub struct RateLimitLayer {
    quota: Quota,
    ip_limiters: IpRateLimiters,
}

impl RateLimitLayer {
    pub fn new() -> Self {
        Self::with_quota(Quota::per_minute(NonZeroU32::new(60).unwrap()))
    }

    pub fn with_quota(quota: Quota) -> Self {
        Self {
            quota,
            ip_limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn per_minute(limit: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(limit).unwrap_or(NonZeroU32::new(1).unwrap()));
        Self::with_quota(quota)
    }

    pub fn per_second(limit: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(limit).unwrap_or(NonZeroU32::new(1).unwrap()));
        Self::with_quota(quota)
    }

    fn get_or_create_limiter(&self, ip: String) -> SharedRateLimiter {
        let mut limiters = self.ip_limiters.lock().unwrap();
        
        limiters.entry(ip).or_insert_with(|| {
            Arc::new(RateLimiter::direct(self.quota))
        }).clone()
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            layer: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    layer: RateLimitLayer,
}

impl<S> Service<Request> for RateLimitService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let layer = self.layer.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract IP address
            let ip = extract_ip(&request).unwrap_or_else(|| "unknown".to_string());
            
            // Get rate limiter for this IP
            let limiter = layer.get_or_create_limiter(ip);

            // Check rate limit
            match limiter.check() {
                Ok(_) => {
                    // Request allowed, proceed
                    inner.call(request).await.map_err(Into::into)
                }
                Err(_) => {
                    // Rate limit exceeded
                    let response = AppError::RateLimited.into_response();
                    Ok(response)
                }
            }
        })
    }
}

fn extract_ip(request: &Request) -> Option<String> {
    // First try to get IP from X-Forwarded-For header (for proxies)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }

    // Then try X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return Some(real_ip_str.to_string());
        }
    }

    // Finally, try to get IP from connection info
    request.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // This is a simple implementation, in production you'd want to use
    // a more sophisticated rate limiter with Redis backing
    next.run(request).await.map_err(|e| {
        tracing::error!("Rate limit middleware error: {:?}", e);
        AppError::Internal("Rate limit error".to_string())
    })
} 