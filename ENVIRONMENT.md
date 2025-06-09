# Environment Configuration

This document describes the environment variables needed for the Video Analytics Platform.

## Required Environment Variables

### Database Configuration
```bash
DATABASE_URL=postgresql://video_analytics:dev_password_change_in_production@localhost:5432/video_analytics
```

### Redis Configuration
```bash
REDIS_URL=redis://localhost:6379
```

### Kafka Configuration
```bash
KAFKA_BROKERS=localhost:9092
```

### Authentication & Security
```bash
JWT_SECRET=your-super-secure-jwt-secret-key-change-in-production
BCRYPT_COST=12
JWT_EXPIRY_HOURS=24
REFRESH_TOKEN_EXPIRY_DAYS=30
```

### Server Configuration
```bash
PORT=8080
RUST_LOG=info
```

### CORS Configuration
```bash
CORS_ORIGINS=http://localhost:3000,http://localhost:8080
```

### Rate Limiting
```bash
RATE_LIMIT_RPM=60
RATE_LIMIT_BURST=10
```

## Development Setup

1. Create a `.env` file in the project root with the variables above
2. Start infrastructure services: `make infra-up`
3. Run the API Gateway: `make dev-api`

## Production Considerations

⚠️ **IMPORTANT**: Change all default passwords and secrets before deploying to production!

- Use a strong, randomly generated JWT secret (at least 64 characters)
- Use secure database passwords
- Set appropriate CORS origins
- Configure rate limiting based on your needs
- Use environment-specific database URLs
- Enable SSL/TLS for all connections

## Service-Specific Variables

### API Gateway (Port 8080)
- `DATABASE_URL`
- `REDIS_URL`
- `KAFKA_BROKERS`
- `JWT_SECRET`

### Ingest Service (Port 8081)
- `API_GATEWAY_URL=http://api-gateway:8080`
- `KAFKA_BROKERS`
- `MINIO_ENDPOINT=minio:9000`
- `MINIO_ACCESS_KEY=minioadmin`
- `MINIO_SECRET_KEY=minioadmin123`

### Inference Service (Port 8082)
- `KAFKA_BROKERS`
- `MODEL_PATH=/app/models`
- `GPU_ENABLED=false`

### Analytics Engine (Port 8083)
- `DATABASE_URL`
- `KAFKA_BROKERS`

### Frontend (Port 3000)
- `NEXT_PUBLIC_API_URL=http://localhost:8080`
- `NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws` 