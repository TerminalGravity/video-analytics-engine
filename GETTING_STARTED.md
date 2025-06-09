# Getting Started - Video Analytics Platform

This guide will help you get the Video Analytics Platform up and running on your local machine.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Docker & Docker Compose** (for infrastructure services)
- **Rust** (latest stable version) - [Install Rust](https://rustup.rs/)
- **Node.js** (v18 or later) - [Install Node.js](https://nodejs.org/)
- **Python** (3.9 or later) - [Install Python](https://www.python.org/)

### Optional Tools
- **cargo-watch** for auto-recompilation: `cargo install cargo-watch`
- **sqlx-cli** for database migrations: `cargo install sqlx-cli`

## Quick Start

### 1. Clone and Setup

```bash
git clone <your-repository>
cd video-analytics-platform
```

### 2. Configure Environment

Create a `.env` file in the project root:

```bash
# Copy the example configuration
cp ENVIRONMENT.md .env

# Edit the .env file with your preferred settings
# The defaults will work for local development
```

### 3. Start Infrastructure Services

```bash
# Start PostgreSQL, Kafka, Redis, and MinIO
make infra-up

# Wait a few seconds for services to initialize
sleep 10
```

### 4. Setup and Build Services

```bash
# Install dependencies and build all services
make setup
```

### 5. Start the Platform

You have two options:

#### Option A: Start All Services at Once
```bash
make dev
```

#### Option B: Start Services Individually (for development)
```bash
# Terminal 1: API Gateway
make dev-api

# Terminal 2: Ingest Service (once implemented)
make dev-ingest

# Terminal 3: Inference Service (once implemented)
make dev-inference

# Terminal 4: Analytics Engine (once implemented)
make dev-analytics

# Terminal 5: Frontend (once implemented)
make dev-frontend
```

## What's Available Now

### ✅ API Gateway (Fully Implemented)
- **GraphQL API**: http://localhost:8080/graphql
- **GraphQL Playground**: http://localhost:8080/graphql (GET request)
- **Health Check**: http://localhost:8080/health
- **WebSocket**: ws://localhost:8080/ws

### 🔧 Infrastructure Services
- **PostgreSQL**: localhost:5432 (with TimescaleDB)
- **Redis**: localhost:6379
- **Kafka**: localhost:9092
- **MinIO**: http://localhost:9001 (admin: minioadmin/minioadmin123)

### 📝 Database Schema
The database is automatically initialized with:
- User management tables
- Video stream tables
- Inference results (time-series)
- Analytics events (time-series)
- Alerts and notifications
- Default admin user: `admin@example.com` / `admin123`

## Testing the API

### 1. Register a User
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "role": "User"
  }'
```

### 2. Login
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

### 3. Use GraphQL
Visit http://localhost:8080/graphql in your browser to access the GraphQL Playground.

Example query:
```graphql
query {
  videoStreams {
    items {
      id
      name
      status
      createdAt
    }
    pagination {
      totalCount
      currentPage
    }
  }
}
```

## Available Make Commands

```bash
make help              # Show all available commands
make dev               # Start all services in development mode
make build             # Build all services
make test              # Run tests for all services
make lint              # Run linting for all services
make format            # Format code for all services
make infra-up          # Start infrastructure services only
make infra-down        # Stop infrastructure services
make docker-up         # Start all services with Docker
make docker-down       # Stop all Docker services
make logs              # Show logs from all services
make health            # Check health of all services
make clean             # Clean build artifacts
```

## Next Steps

The foundation is now ready! Here's what you can do next:

### 1. Complete the Remaining Services
- **Ingest Service**: Video ingestion via WebRTC/RTMP
- **Inference Service**: AI/ML video analysis
- **Analytics Engine**: Data processing and insights
- **Frontend**: React/Next.js dashboard

### 2. Add Features
- Real-time video streaming
- AI model management
- Advanced analytics dashboards
- Alert management system
- User roles and permissions

### 3. Production Deployment
- Configure environment variables for production
- Set up reverse proxy (nginx)
- Configure SSL certificates
- Set up monitoring and logging
- Implement backup strategies

## Troubleshooting

### Common Issues

**Database connection failed**
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check logs
make infra-logs
```

**Port already in use**
```bash
# Check what's using the port
lsof -i :8080

# Kill the process or change the port in your .env file
```

**Rust compilation errors**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
make clean
make build
```

**Docker issues**
```bash
# Reset Docker environment
make docker-clean
make infra-up
```

## Architecture Overview

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Frontend  │    │ API Gateway │    │   Services  │
│ (Next.js)   │◄──►│   (Rust)    │◄──►│ (Rust/Py)  │
│   :3000     │    │    :8080    │    │ :8081-8083  │
└─────────────┘    └─────────────┘    └─────────────┘
                           │
                           ▼
              ┌─────────────────────────┐
              │    Infrastructure       │
              │  PostgreSQL + Kafka     │
              │  Redis + MinIO         │
              └─────────────────────────┘
```

## Support

If you encounter any issues:

1. Check the logs: `make logs`
2. Verify services are healthy: `make health`
3. Review the environment configuration: `ENVIRONMENT.md`
4. Check the troubleshooting section above

Happy coding! 🚀 