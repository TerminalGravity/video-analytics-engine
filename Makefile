.PHONY: help dev build clean test lint format docker-build docker-up docker-down infra-up infra-down logs

# Default target
help: ## Show this help message
	@echo "Video Analytics Platform - Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development
dev: ## Start all services in development mode
	@echo "Starting development environment..."
	make infra-up
	@sleep 10  # Wait for infrastructure to be ready
	@echo "Starting application services..."
	@cargo watch -x run &
	@cd frontend && npm run dev &
	@cd inference-service && python main.py &
	@wait

dev-frontend: ## Start only frontend development server
	cd frontend && npm run dev

dev-api: ## Start only API gateway in development mode
	cd api-gateway && cargo watch -x run

dev-ingest: ## Start only ingest service in development mode
	cd ingest-service && cargo watch -x run

dev-inference: ## Start only inference service in development mode
	cd inference-service && python main.py

dev-analytics: ## Start only analytics engine in development mode
	cd analytics-engine && cargo watch -x run

# Building
build: ## Build all services
	@echo "Building all services..."
	cd api-gateway && cargo build --release
	cd ingest-service && cargo build --release
	cd analytics-engine && cargo build --release
	cd inference-service && pip install -r requirements.txt
	cd frontend && npm install && npm run build

build-docker: ## Build all Docker images
	@echo "Building Docker images..."
	docker-compose build

# Testing
test: ## Run tests for all services
	@echo "Running tests..."
	cd api-gateway && cargo test
	cd ingest-service && cargo test
	cd analytics-engine && cargo test
	cd inference-service && python -m pytest
	cd frontend && npm test

# Linting and Formatting
lint: ## Run linting for all services
	@echo "Running linting..."
	cd api-gateway && cargo clippy
	cd ingest-service && cargo clippy
	cd analytics-engine && cargo clippy
	cd inference-service && python -m flake8 .
	cd frontend && npm run lint

format: ## Format code for all services
	@echo "Formatting code..."
	cd api-gateway && cargo fmt
	cd ingest-service && cargo fmt
	cd analytics-engine && cargo fmt
	cd inference-service && python -m black .
	cd frontend && npm run format

# Docker operations
docker-up: ## Start all services with Docker Compose
	docker-compose up -d

docker-down: ## Stop all Docker services
	docker-compose down

docker-logs: ## Show logs from all Docker services
	docker-compose logs -f

docker-clean: ## Clean up Docker containers and images
	docker-compose down -v
	docker system prune -f

# Infrastructure only
infra-up: ## Start only infrastructure services (postgres, kafka, redis, minio)
	docker-compose up -d postgres kafka zookeeper redis minio

infra-down: ## Stop infrastructure services
	docker-compose stop postgres kafka zookeeper redis minio

infra-logs: ## Show infrastructure service logs
	docker-compose logs -f postgres kafka redis minio

# Database operations
db-migrate: ## Run database migrations
	cd api-gateway && sqlx migrate run

db-reset: ## Reset database (drop and recreate)
	@echo "Resetting database..."
	docker-compose exec postgres psql -U video_analytics -d video_analytics -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	make db-migrate

# Setup and initialization
setup: ## Initial project setup
	@echo "Setting up project..."
	@echo "Installing Rust dependencies..."
	cd api-gateway && cargo build
	cd ingest-service && cargo build
	cd analytics-engine && cargo build
	@echo "Installing Node.js dependencies..."
	cd frontend && npm install
	@echo "Installing Python dependencies..."
	cd inference-service && pip install -r requirements.txt
	@echo "Setup complete!"

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cd api-gateway && cargo clean
	cd ingest-service && cargo clean
	cd analytics-engine && cargo clean
	cd frontend && rm -rf .next node_modules
	cd inference-service && find . -name "__pycache__" -exec rm -rf {} +

# Production deployment
deploy: ## Deploy to production (build and start all services)
	@echo "Deploying to production..."
	make build-docker
	make docker-up

# Monitoring and logs
logs: ## Show logs from all services
	docker-compose logs -f

logs-api: ## Show API gateway logs
	docker-compose logs -f api-gateway

logs-ingest: ## Show ingest service logs
	docker-compose logs -f ingest-service

logs-inference: ## Show inference service logs
	docker-compose logs -f inference-service

logs-analytics: ## Show analytics engine logs
	docker-compose logs -f analytics-engine

logs-frontend: ## Show frontend logs
	docker-compose logs -f frontend

# Health checks
health: ## Check health of all services
	@echo "Checking service health..."
	@curl -f http://localhost:8080/health || echo "API Gateway: DOWN"
	@curl -f http://localhost:8081/health || echo "Ingest Service: DOWN"
	@curl -f http://localhost:8082/health || echo "Inference Service: DOWN"
	@curl -f http://localhost:8083/health || echo "Analytics Engine: DOWN"
	@curl -f http://localhost:3000 || echo "Frontend: DOWN" 