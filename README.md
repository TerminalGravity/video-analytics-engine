# Video Analytics Platform
Imagine this project is a big box of LEGO® pieces for “computer vision”

	1.	Roboflow already supplies the bricks
- Some bricks can spot players and balls in any video.
- Others can follow each player around the field.
- A special brick flattens the camera view so we know where everything is on the court or pitch.
	2.	Our job is to click those bricks into a simple web app
- A coach (or a parent with a phone) uploads a game video or points a live camera at the field.
- The app chews on the footage and pops out instant goodies: heat‑maps, shot charts, who ran how far, auto‑generated highlights.
- Everything shows up on a clean dashboard the same way Google Analytics shows website stats.
	3.	Why use Rust under the hood?
    
Think of Rust as the high‑speed train engine that pulls all the LEGO cars. It keeps the service lightning‑fast (important for live streams) and very safe (no surprise crashes during a match).


A real-time video analytics platform with AI inference capabilities, built with modern microservices architecture.

## Architecture Overview

```
                ┌──────────────────────────────┐
                │   Web / Mobile Front‑end     │
                │  (Next.js + WebSockets)      │
                └────────────┬─────────────────┘
                             │ GraphQL HTTPS
┌───────────┐  gRPC   ┌──────────────────────────┐  Kafka   ┌──────────────────┐
│ IngestSvc │◀──────▶│  API Gateway (axum)      │◀────────▶│ Analytics Engine │
│  (Rust)   │        │  Auth, rate‑limits       │          │  (Rust + polars) │
└───────────┘        └──────────┬───────────────┘          └──────────┬───────┘
     ▲ WebRTC / RTMP            │ gRPC / HTTP                           │
     │                          ▼                                       │
┌───────────┐          ┌─────────────────────┐                Postgres / Timescale
│ Camera /  │   Zero‑  │ InferenceSvc GPU    │   JSON events  │ MinIO media store
│ Uploaders │>  copy   │ (Python or Rust‑ONNX│  ─────────────▶│ Redis  cache
└───────────┘          │ compiled models)    │                └──────────────────┘
```

## Services

### Frontend (`/frontend`)
- **Next.js** web application with real-time WebSocket connections
- GraphQL client for API communication
- Modern, responsive UI for video management and analytics

### API Gateway (`/api-gateway`)
- **Rust/axum** HTTP server with GraphQL endpoint
- Authentication and authorization
- Rate limiting and request validation
- Service orchestration

### Ingest Service (`/ingest-service`)
- **Rust** video ingestion service
- WebRTC and RTMP stream handling
- Zero-copy video processing
- Integration with inference service

### Inference Service (`/inference-service`)
- **Python** or **Rust-ONNX** AI/ML inference engine
- GPU-accelerated model execution
- Real-time video analysis
- Event publishing to Kafka

### Analytics Engine (`/analytics-engine`)
- **Rust + Polars** data processing engine
- Time-series analytics
- Event aggregation and insights
- Data pipeline management

## Infrastructure

- **PostgreSQL/TimescaleDB**: Primary database with time-series capabilities
- **MinIO**: Object storage for media files
- **Redis**: Caching and session management
- **Kafka**: Event streaming and message queuing

## Quick Start

1. **Prerequisites**
   ```bash
   # Install Docker and Docker Compose
   # Install Rust, Node.js, and Python
   ```

2. **Development Setup**
   ```bash
   # Clone and setup
   git clone <repository>
   cd video-analytics-platform
   
   # Start infrastructure services
   docker-compose up -d postgres kafka redis minio
   
   # Start development services
   make dev
   ```

3. **Production Deployment**
   ```bash
   # Build and deploy all services
   docker-compose up -d
   ```

## Development

- **Language Stack**: Rust, TypeScript/JavaScript, Python
- **Databases**: PostgreSQL, TimescaleDB, Redis
- **Message Queue**: Apache Kafka
- **Storage**: MinIO (S3-compatible)
- **Container**: Docker with multi-stage builds

## Security Features

- JWT-based authentication
- Role-based access control
- Rate limiting on all endpoints
- Input validation and sanitization
- Secure headers and CORS policies

## Monitoring & Observability

- Structured logging across all services
- Metrics collection and dashboards
- Distributed tracing
- Health checks and service discovery

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

## License

[License information] 