use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub kafka_brokers: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub rate_limit: RateLimitConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub bcrypt_cost: u32,
}

impl Config {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let config = Config {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| {
                    "postgresql://video_analytics:dev_password_change_in_production@localhost:5432/video_analytics".to_string()
                }),
            
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            
            kafka_brokers: env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secure-jwt-secret-key-change-in-production".to_string()),
            
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            
            rate_limit: RateLimitConfig {
                requests_per_minute: env::var("RATE_LIMIT_RPM")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                burst_size: env::var("RATE_LIMIT_BURST")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
            
            auth: AuthConfig {
                jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
                refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                bcrypt_cost: env::var("BCRYPT_COST")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
            },
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters long".into());
        }

        if self.port == 0 {
            return Err("Port must be greater than 0".into());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }

        if self.redis_url.is_empty() {
            return Err("Redis URL cannot be empty".into());
        }

        if self.kafka_brokers.is_empty() {
            return Err("Kafka brokers cannot be empty".into());
        }

        Ok(())
    }
} 